#[cfg(test)]
mod tests {
    use super::{
        RecordingManager, consume_matching_prefix, strip_one_leading_newline,
        strip_terminal_control_sequences,
    };
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_path(name: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir()
            .join(format!("nyaterm-recording-{name}-{nanos}.log"))
            .to_string_lossy()
            .to_string()
    }

    #[test]
    fn strips_terminal_escape_sequences_from_output() {
        let raw = concat!(
            "\x1b[?2004l",
            "app.log  \x1b[0m\x1b[01;34mgo\x1b[0m\n",
            "\x1b]7;file://ubuntu/root\x07",
            "\x1b[?2004h\x1b[0m\x1b[1;33m[root\x1b[1;37m@\x1b[1;36mubuntu ",
            "\x1b[1;32m~\x1b[1;35m]\x1b[1;31m\n\n# \x1b[0m"
        );

        let cleaned = strip_terminal_control_sequences(raw);
        assert_eq!(cleaned, "app.log  go\n[root@ubuntu ~]\n\n# ");
    }

    #[test]
    fn strips_unknown_escape_with_multibyte_replacement_without_panicking() {
        let raw = format!("before\x1b{}after\n", char::REPLACEMENT_CHARACTER);

        let cleaned = strip_terminal_control_sequences(&raw);

        assert_eq!(cleaned, "beforeafter\n");
    }

    #[test]
    fn consumes_matching_echo_prefix() {
        let mut prefix = "ps -ef".to_string();
        let consumed = consume_matching_prefix(&mut prefix, "ps -ef\nUID");
        assert_eq!(consumed, "ps -ef".len());
        assert!(prefix.is_empty());
    }

    #[test]
    fn strips_only_one_leading_newline() {
        assert_eq!(strip_one_leading_newline("\nhello"), "hello");
        assert_eq!(strip_one_leading_newline("hello"), "hello");
        assert_eq!(strip_one_leading_newline("\n\nhello"), "\nhello");
    }

    #[test]
    fn writes_recording_with_and_without_io_labels() {
        let manager = RecordingManager::new();
        let labeled_path = unique_path("labels");
        manager.start("s1", &labeled_path, true, true).unwrap();
        manager.write_input("s1", b"echo hi\r");
        manager.write_output("s1", "echo hi\r\nhi\n");
        manager.stop("s1").unwrap();

        let labeled = fs::read_to_string(&labeled_path).unwrap();
        assert!(labeled.contains("[INPUT] echo hi"));
        assert!(labeled.contains("[OUTPUT] hi"));

        let plain_path = unique_path("plain");
        manager.start("s1", &plain_path, false, true).unwrap();
        manager.write_output("s1", "done\n");
        manager.stop("s1").unwrap();

        let plain = fs::read_to_string(&plain_path).unwrap();
        assert!(!plain.contains("[INPUT]"));
        assert!(!plain.contains("[OUTPUT]"));
        assert!(plain.contains("done"));

        let _ = fs::remove_file(labeled_path);
        let _ = fs::remove_file(plain_path);
    }

    #[test]
    fn writes_recording_without_timestamps() {
        let manager = RecordingManager::new();

        let labeled_path = unique_path("no-timestamp-labels");
        manager.start("s1", &labeled_path, true, false).unwrap();
        manager.write_output("s1", "done\n");
        manager.stop("s1").unwrap();

        let labeled = fs::read_to_string(&labeled_path).unwrap();
        assert_eq!(labeled, "[OUTPUT] done\n");

        let plain_path = unique_path("no-timestamp-plain");
        manager.start("s1", &plain_path, false, false).unwrap();
        manager.write_output("s1", "plain\n");
        manager.stop("s1").unwrap();

        let plain = fs::read_to_string(&plain_path).unwrap();
        assert_eq!(plain, "plain\n");

        let _ = fs::remove_file(labeled_path);
        let _ = fs::remove_file(plain_path);
    }

    #[test]
    fn saves_memory_transcript_and_trims_old_records() {
        let manager = RecordingManager::new();
        manager.set_memory_limit(90);
        manager.write_output("s1", "first line\n");
        manager.write_output("s1", "second line\n");
        manager.write_output("s1", "third line\n");

        let path = unique_path("memory");
        manager.save_transcript("s1", &path, true, true).unwrap();
        let saved = fs::read_to_string(&path).unwrap();

        assert!(!saved.contains("first line"));
        assert!(saved.contains("third line"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn saves_transcript_after_binary_like_output() {
        let manager = RecordingManager::new();
        let output = format!("ready\x1b{}done\n", char::REPLACEMENT_CHARACTER);

        manager.write_output("s1", &output);

        let path = unique_path("binary-like");
        manager.save_transcript("s1", &path, true, true).unwrap();
        let saved = fs::read_to_string(&path).unwrap();

        assert!(saved.contains("readydone"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn terminal_history_search_finds_literal_matches() {
        let manager = RecordingManager::new();
        manager.write_output("s1", "alpha\nbeta install\nbeta done\n");

        let result = manager
            .search_history(super::TerminalHistorySearchRequest {
                session_id: "s1".to_string(),
                query: "beta".to_string(),
                case_sensitive: false,
                regex: false,
                whole_word: false,
                limit: Some(100),
                context_before: Some(1),
                context_after: Some(1),
                max_lines: None,
            })
            .unwrap();

        assert_eq!(result.total, 2);
        assert_eq!(result.results.len(), 2);
        assert_eq!(result.results[0].line_number, 2);
        assert_eq!(result.results[0].before, vec!["alpha"]);
        assert_eq!(result.results[0].after, vec!["beta done"]);
        assert_eq!(result.results[0].source, "output");
    }

    #[test]
    fn terminal_history_search_honors_case_and_whole_word() {
        let manager = RecordingManager::new();
        manager.write_output("s1", "install\nInstall\ninstaller\n");

        let case_sensitive = manager
            .search_history(super::TerminalHistorySearchRequest {
                session_id: "s1".to_string(),
                query: "Install".to_string(),
                case_sensitive: true,
                regex: false,
                whole_word: false,
                limit: Some(100),
                context_before: Some(0),
                context_after: Some(0),
                max_lines: None,
            })
            .unwrap();
        assert_eq!(case_sensitive.total, 1);
        assert_eq!(case_sensitive.results[0].preview, "Install");

        let whole_word = manager
            .search_history(super::TerminalHistorySearchRequest {
                session_id: "s1".to_string(),
                query: "install".to_string(),
                case_sensitive: false,
                regex: false,
                whole_word: true,
                limit: Some(100),
                context_before: Some(0),
                context_after: Some(0),
                max_lines: None,
            })
            .unwrap();
        assert_eq!(whole_word.total, 2);
    }

    #[test]
    fn terminal_history_search_supports_regex_limit_and_truncation() {
        let manager = RecordingManager::new();
        manager.write_output("s1", "error 100\nerror 200\nok\n");

        let result = manager
            .search_history(super::TerminalHistorySearchRequest {
                session_id: "s1".to_string(),
                query: r"error \d+".to_string(),
                case_sensitive: false,
                regex: true,
                whole_word: false,
                limit: Some(1),
                context_before: Some(0),
                context_after: Some(0),
                max_lines: None,
            })
            .unwrap();

        assert_eq!(result.total, 2);
        assert_eq!(result.results.len(), 1);
        assert!(result.truncated);
        assert_eq!(result.results[0].preview, "error 100");
    }

    #[test]
    fn recording_does_not_backfill_existing_memory() {
        let manager = RecordingManager::new();
        manager.write_output("s1", "before\n");

        let path = unique_path("no-backfill");
        manager.start("s1", &path, true, true).unwrap();
        manager.write_output("s1", "after\n");
        manager.stop("s1").unwrap();

        let recorded = fs::read_to_string(&path).unwrap();
        assert!(!recorded.contains("before"));
        assert!(recorded.contains("after"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn recording_does_not_backfill_partial_output_buffer() {
        let manager = RecordingManager::new();
        manager.write_output("s1", "prompt without newline");

        let path = unique_path("no-partial-backfill");
        manager.start("s1", &path, true, true).unwrap();
        manager.write_output("s1", "\nafter\n");
        manager.stop("s1").unwrap();

        let recorded = fs::read_to_string(&path).unwrap();
        assert!(!recorded.contains("prompt without newline"));
        assert!(recorded.contains("after"));

        let _ = fs::remove_file(path);
    }
}
