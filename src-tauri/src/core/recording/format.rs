fn prepare_output_file_path(file_path: &str) -> AppResult<PathBuf> {
    let path = PathBuf::from(file_path);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Config(format!("Failed to create directory: {e}")))?;
        }
    }
    Ok(path)
}

fn format_record_parts(
    timestamp: &str,
    label: &str,
    data: &str,
    include_io_labels: bool,
    include_timestamps: bool,
) -> String {
    match (include_timestamps, include_io_labels) {
        (true, true) => format!("[{timestamp}] [{label}] {data}\n"),
        (true, false) => format!("[{timestamp}] {data}\n"),
        (false, true) => format!("[{label}] {data}\n"),
        (false, false) => format!("{data}\n"),
    }
}

fn chrono_timestamp() -> String {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    now.format(time::macros::format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
    ))
    .unwrap_or_else(|_| "1970-01-01 00:00:00.000".to_string())
}

fn consume_matching_prefix(prefix_buffer: &mut String, text: &str) -> usize {
    let mut prefix_idx = 0;
    let mut text_idx = 0;

    while prefix_idx < prefix_buffer.len() && text_idx < text.len() {
        let prefix_char = prefix_buffer[prefix_idx..].chars().next();
        let text_char = text[text_idx..].chars().next();

        match (prefix_char, text_char) {
            (Some(left), Some(right)) if left == right => {
                prefix_idx += left.len_utf8();
                text_idx += right.len_utf8();
            }
            _ => break,
        }
    }

    if prefix_idx > 0 {
        prefix_buffer.drain(..prefix_idx);
    }

    text_idx
}

fn strip_one_leading_newline(text: &str) -> &str {
    text.strip_prefix('\n').unwrap_or(text)
}

fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
