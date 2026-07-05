fn sanitize_filename(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let sanitized: String = base
        .chars()
        .map(|c| {
            if matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*') || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();

    if sanitized.is_empty() {
        "zmodem_file".to_string()
    } else {
        sanitized
    }
}

/// Build a 5×CAN + 5×BS abort/cancel sequence per ZMODEM spec.
fn cancel_sequence() -> Vec<u8> {
    let mut seq = vec![ZDLE; CANCEL_SEQ_LEN];
    seq.extend([0x08; CANCEL_SEQ_LEN]); // backspace to clean up display
    seq
}
