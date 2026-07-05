#[derive(Clone, Debug)]
struct TranscriptRecord {
    line_id: u64,
    timestamp: String,
    label: &'static str,
    data: String,
    size_bytes: usize,
}

impl TranscriptRecord {
    fn new(line_id: u64, label: &'static str, data: String) -> Self {
        let timestamp = chrono_timestamp();
        let size_bytes = format_record_parts(&timestamp, label, &data, true, true).len();
        Self {
            line_id,
            timestamp,
            label,
            data,
            size_bytes,
        }
    }

    fn format(&self, include_io_labels: bool, include_timestamps: bool) -> String {
        format_record_parts(
            &self.timestamp,
            self.label,
            &self.data,
            include_io_labels,
            include_timestamps,
        )
    }
}

