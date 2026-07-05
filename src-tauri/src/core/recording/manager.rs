pub struct RecordingManager {
    sessions: Mutex<HashMap<String, SessionCaptureState>>,
    memory_limit_bytes: Mutex<usize>,
}

impl RecordingManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            memory_limit_bytes: Mutex::new(DEFAULT_MEMORY_LIMIT_BYTES),
        }
    }

    pub fn start(
        &self,
        session_id: &str,
        file_path: &str,
        include_io_labels: bool,
        include_timestamps: bool,
    ) -> AppResult<()> {
        let path = prepare_output_file_path(file_path)?;
        let file = File::create(&path)
            .map_err(|e| AppError::Config(format!("Failed to create recording file: {e}")))?;
        let memory_limit_bytes = *lock_recover(&self.memory_limit_bytes);

        let mut sessions = lock_recover(&self.sessions);
        let state = sessions
            .entry(session_id.to_string())
            .or_insert_with(|| SessionCaptureState::new(memory_limit_bytes));
        state.set_memory_limit(memory_limit_bytes);
        state.start_recording(file, path, include_io_labels, include_timestamps)
    }

    pub fn stop(&self, session_id: &str) -> AppResult<String> {
        let mut sessions = lock_recover(&self.sessions);
        let state = sessions
            .get_mut(session_id)
            .ok_or_else(|| AppError::Config("No active recording".to_string()))?;
        state.stop_recording()
    }

    pub fn save_transcript(
        &self,
        session_id: &str,
        file_path: &str,
        include_io_labels: bool,
        include_timestamps: bool,
    ) -> AppResult<String> {
        let path = prepare_output_file_path(file_path)?;
        let records = {
            let mut sessions = lock_recover(&self.sessions);
            sessions
                .get_mut(session_id)
                .map(SessionCaptureState::snapshot_records)
                .unwrap_or_default()
        };

        let mut writer = BufWriter::new(
            File::create(&path)
                .map_err(|e| AppError::Config(format!("Failed to create transcript file: {e}")))?,
        );
        for record in &records {
            writer
                .write_all(
                    record
                        .format(include_io_labels, include_timestamps)
                        .as_bytes(),
                )
                .map_err(|e| AppError::Config(format!("Failed to write transcript file: {e}")))?;
        }
        writer
            .flush()
            .map_err(|e| AppError::Config(format!("Failed to flush transcript file: {e}")))?;
        Ok(path.to_string_lossy().to_string())
    }

    pub fn search_history(
        &self,
        request: TerminalHistorySearchRequest,
    ) -> AppResult<TerminalHistorySearchResponse> {
        let started = Instant::now();
        let query = request.query;
        if query.is_empty() {
            return Ok(TerminalHistorySearchResponse {
                total: 0,
                elapsed_ms: started.elapsed().as_millis(),
                truncated: false,
                results: Vec::new(),
            });
        }

        let limit = request.limit.unwrap_or(DEFAULT_HISTORY_SEARCH_LIMIT).max(1);
        let context_before = request.context_before.unwrap_or(0).min(20);
        let context_after = request.context_after.unwrap_or(0).min(20);
        let max_lines = request
            .max_lines
            .unwrap_or(DEFAULT_HISTORY_SEARCH_LINES)
            .clamp(1, MAX_HISTORY_SEARCH_LINES);
        let records = {
            let mut sessions = lock_recover(&self.sessions);
            sessions
                .get_mut(&request.session_id)
                .map(SessionCaptureState::snapshot_records)
                .unwrap_or_default()
        };
        let start_index = records.len().saturating_sub(max_lines);
        let searched_records = &records[start_index..];
        let matcher = HistoryMatcher::new(
            &query,
            request.case_sensitive,
            request.regex,
            request.whole_word,
        )?;
        let mut total = 0usize;
        let mut results = Vec::new();

        for (relative_index, record) in searched_records.iter().enumerate() {
            if let Some((column_start, column_end)) = matcher.find(&record.data) {
                total += 1;
                if results.len() < limit {
                    let absolute_index = start_index + relative_index;
                    results.push(TerminalHistorySearchResult {
                        line_id: record.line_id,
                        line_number: absolute_index + 1,
                        column_start,
                        column_end,
                        preview: record.data.clone(),
                        before: context_records(&records, absolute_index, context_before, true),
                        after: context_records(&records, absolute_index, context_after, false),
                        source: record.label.to_ascii_lowercase(),
                    });
                }
            }
        }

        Ok(TerminalHistorySearchResponse {
            total,
            elapsed_ms: started.elapsed().as_millis(),
            truncated: total > results.len() || records.len() > max_lines,
            results,
        })
    }

    pub fn set_memory_limit(&self, max_bytes: usize) {
        let bounded = max_bytes.max(1);
        *lock_recover(&self.memory_limit_bytes) = bounded;

        let mut sessions = lock_recover(&self.sessions);
        for state in sessions.values_mut() {
            state.set_memory_limit(bounded);
        }
    }

    pub fn is_recording(&self, session_id: &str) -> bool {
        self.sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(session_id)
            .is_some_and(|state| state.recording.is_some())
    }

    pub fn list_recording_sessions(&self) -> Vec<String> {
        self.sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .filter_map(|(id, state)| state.recording.as_ref().map(|_| id.clone()))
            .collect()
    }

    pub fn write_output(&self, session_id: &str, data: &str) {
        let memory_limit_bytes = *lock_recover(&self.memory_limit_bytes);
        let mut sessions = lock_recover(&self.sessions);
        let state = sessions
            .entry(session_id.to_string())
            .or_insert_with(|| SessionCaptureState::new(memory_limit_bytes));
        state.set_memory_limit(memory_limit_bytes);
        state.write_output(data);
    }

    pub fn write_input(&self, session_id: &str, data: &[u8]) {
        let memory_limit_bytes = *lock_recover(&self.memory_limit_bytes);
        let mut sessions = lock_recover(&self.sessions);
        let state = sessions
            .entry(session_id.to_string())
            .or_insert_with(|| SessionCaptureState::new(memory_limit_bytes));
        state.set_memory_limit(memory_limit_bytes);
        state.write_input(data);
    }

    pub fn cleanup_session(&self, session_id: &str) {
        let removed = {
            let mut sessions = lock_recover(&self.sessions);
            sessions.remove(session_id)
        };
        if let Some(mut state) = removed {
            state.finish();
        }
    }
}
