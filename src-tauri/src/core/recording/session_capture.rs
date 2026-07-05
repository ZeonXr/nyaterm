struct FileRecording {
    writer: BufWriter<File>,
    file_path: PathBuf,
    include_io_labels: bool,
    include_timestamps: bool,
}

impl FileRecording {
    fn new(
        file: File,
        file_path: PathBuf,
        include_io_labels: bool,
        include_timestamps: bool,
    ) -> Self {
        Self {
            writer: BufWriter::new(file),
            file_path,
            include_io_labels,
            include_timestamps,
        }
    }

    fn write_record(&mut self, record: &TranscriptRecord) {
        let _ = self.writer.write_all(
            record
                .format(self.include_io_labels, self.include_timestamps)
                .as_bytes(),
        );
    }

    fn finish(&mut self) {
        let _ = self.writer.flush();
    }
}

struct SessionCaptureState {
    recording: Option<FileRecording>,
    records: VecDeque<TranscriptRecord>,
    record_bytes: usize,
    memory_limit_bytes: usize,
    input_buffer: String,
    output_buffer: String,
    live_echo_buffer: String,
    submitted_line_echo: Option<String>,
    suppress_next_newline: bool,
    next_line_id: u64,
}

impl SessionCaptureState {
    fn new(memory_limit_bytes: usize) -> Self {
        Self {
            recording: None,
            records: VecDeque::new(),
            record_bytes: 0,
            memory_limit_bytes,
            input_buffer: String::new(),
            output_buffer: String::new(),
            live_echo_buffer: String::new(),
            submitted_line_echo: None,
            suppress_next_newline: false,
            next_line_id: 1,
        }
    }

    fn set_memory_limit(&mut self, memory_limit_bytes: usize) {
        self.memory_limit_bytes = memory_limit_bytes;
        self.trim_records();
    }

    fn start_recording(
        &mut self,
        file: File,
        file_path: PathBuf,
        include_io_labels: bool,
        include_timestamps: bool,
    ) -> AppResult<()> {
        if self.recording.is_some() {
            return Err(AppError::Config("Recording is already active".to_string()));
        }
        self.flush_output_lines(true);
        self.recording = Some(FileRecording::new(
            file,
            file_path,
            include_io_labels,
            include_timestamps,
        ));
        Ok(())
    }

    fn stop_recording(&mut self) -> AppResult<String> {
        if self.recording.is_none() {
            return Err(AppError::Config("No active recording".to_string()));
        }
        self.commit_partial_input();
        self.flush_output_lines(true);
        let mut recording = self
            .recording
            .take()
            .ok_or_else(|| AppError::Config("No active recording".to_string()))?;
        recording.finish();
        Ok(recording.file_path.to_string_lossy().to_string())
    }

    fn write_input(&mut self, data: &[u8]) {
        let text = String::from_utf8_lossy(data);

        for ch in text.chars() {
            match ch {
                '\r' | '\n' => self.commit_input_line(),
                '\u{8}' | '\u{7f}' => self.handle_backspace(),
                '\t' => {
                    self.input_buffer.push('\t');
                    self.live_echo_buffer.push('\t');
                }
                c if !c.is_control() => {
                    self.input_buffer.push(c);
                    self.live_echo_buffer.push(c);
                }
                _ => {}
            }
        }
    }

    fn write_output(&mut self, data: &str) {
        let mut sanitized = strip_terminal_control_sequences(data);
        if sanitized.is_empty() {
            return;
        }

        if self.suppress_next_newline {
            sanitized = strip_one_leading_newline(&sanitized).to_string();
            self.suppress_next_newline = false;
            if sanitized.is_empty() {
                return;
            }
        }

        sanitized = self.consume_live_echo(&sanitized);
        if sanitized.is_empty() {
            return;
        }

        let (mut sanitized, consumed_submitted_echo) = self.consume_submitted_echo(&sanitized);
        if sanitized.is_empty() {
            return;
        }

        if !consumed_submitted_echo && self.submitted_line_echo.is_some() {
            sanitized = strip_one_leading_newline(&sanitized).to_string();
            self.submitted_line_echo = None;
            if sanitized.is_empty() {
                return;
            }
        }

        self.output_buffer.push_str(&sanitized);
        self.flush_output_lines(false);
    }

    fn finish(&mut self) {
        self.commit_partial_input();
        self.flush_output_lines(true);
        if let Some(recording) = self.recording.as_mut() {
            recording.finish();
        }
        self.recording = None;
    }

    fn snapshot_records(&mut self) -> Vec<TranscriptRecord> {
        self.flush_output_lines(true);
        self.records.iter().cloned().collect()
    }

    fn append_record(&mut self, label: &'static str, data: String) {
        if data.is_empty() {
            return;
        }

        let line_id = self.next_line_id;
        self.next_line_id = self.next_line_id.saturating_add(1);
        let record = TranscriptRecord::new(line_id, label, data);
        if let Some(recording) = self.recording.as_mut() {
            recording.write_record(&record);
        }

        self.record_bytes += record.size_bytes;
        self.records.push_back(record);
        self.trim_records();
    }

    fn trim_records(&mut self) {
        while self.records.len() > 1 && self.record_bytes > self.memory_limit_bytes {
            if let Some(record) = self.records.pop_front() {
                self.record_bytes = self.record_bytes.saturating_sub(record.size_bytes);
            }
        }
    }

    fn handle_backspace(&mut self) {
        if let Some(removed) = self.input_buffer.pop() {
            if self.live_echo_buffer.ends_with(removed) {
                self.live_echo_buffer.pop();
            }
        }
    }

    fn commit_input_line(&mut self) {
        self.flush_output_lines(true);
        let line = mem::take(&mut self.input_buffer);
        self.live_echo_buffer.clear();

        if line.trim().is_empty() {
            self.submitted_line_echo = None;
            return;
        }

        self.append_record("INPUT", line.clone());
        self.submitted_line_echo = Some(line);
    }

    fn commit_partial_input(&mut self) {
        self.flush_output_lines(true);
        let line = mem::take(&mut self.input_buffer);
        self.live_echo_buffer.clear();
        self.submitted_line_echo = None;

        if line.trim().is_empty() {
            return;
        }

        self.append_record("INPUT", line);
    }

    fn consume_live_echo(&mut self, text: &str) -> String {
        let consumed = consume_matching_prefix(&mut self.live_echo_buffer, text);
        text[consumed..].to_string()
    }

    fn consume_submitted_echo(&mut self, text: &str) -> (String, bool) {
        let Some(line) = self.submitted_line_echo.as_ref() else {
            return (text.to_string(), false);
        };

        if !text.starts_with(line) {
            return (text.to_string(), false);
        }

        let mut remaining = text[line.len()..].to_string();
        self.submitted_line_echo = None;

        let stripped = strip_one_leading_newline(&remaining);
        if stripped.len() != remaining.len() {
            remaining = stripped.to_string();
        } else {
            self.suppress_next_newline = true;
        }

        (remaining, true)
    }

    fn flush_output_lines(&mut self, flush_partial: bool) {
        while let Some(pos) = self.output_buffer.find('\n') {
            let line = self.output_buffer[..pos].to_string();
            self.output_buffer.drain(..=pos);
            self.append_record("OUTPUT", line);
        }

        if flush_partial && !self.output_buffer.is_empty() {
            let tail = mem::take(&mut self.output_buffer);
            self.append_record("OUTPUT", tail);
        }
    }
}
