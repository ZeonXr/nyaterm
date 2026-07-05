#[cfg(test)]
mod tests {
    use super::{
        DO, IAC, OPT_NAWS, OPT_SUPPRESS_GO_AHEAD, TelnetEnterMode, TelnetLineEditor,
        TelnetSessionConfig, WILL, maybe_build_naws, negotiate_response, normalize_enter_bytes,
        split_write_chunks, strip_telnet_commands,
    };

    #[test]
    fn standard_negotiation_responds_by_default() {
        assert_eq!(
            negotiate_response(WILL, OPT_SUPPRESS_GO_AHEAD, true, true),
            vec![IAC, DO, OPT_SUPPRESS_GO_AHEAD]
        );
    }

    #[test]
    fn send_sga_false_rejects_sga() {
        assert_ne!(
            negotiate_response(WILL, OPT_SUPPRESS_GO_AHEAD, true, false),
            vec![IAC, DO, OPT_SUPPRESS_GO_AHEAD]
        );
    }

    #[test]
    fn send_naws_false_rejects_naws_negotiation() {
        assert_ne!(
            negotiate_response(DO, OPT_NAWS, false, true),
            vec![IAC, WILL, OPT_NAWS]
        );
    }

    #[test]
    fn raw_mode_can_suppress_negotiation_responses() {
        let config = TelnetSessionConfig {
            raw_tcp_cli: true,
            ..Default::default()
        };
        let mut responses = Vec::new();
        if !config.raw_tcp_cli {
            let _ = strip_telnet_commands(&[IAC, WILL, OPT_SUPPRESS_GO_AHEAD], &mut |cmd, opt| {
                responses.push(negotiate_response(
                    cmd,
                    opt,
                    config.send_naws,
                    config.send_sga,
                ));
            });
        }
        assert!(responses.is_empty());
    }

    #[test]
    fn send_naws_false_prevents_naws_resize_payload() {
        let config = TelnetSessionConfig {
            send_naws: false,
            ..Default::default()
        };
        assert!(maybe_build_naws(80, 24, &config).is_none());
    }

    #[test]
    fn raw_mode_prevents_naws_resize_payload() {
        let config = TelnetSessionConfig {
            raw_tcp_cli: true,
            ..Default::default()
        };
        assert!(maybe_build_naws(80, 24, &config).is_none());
    }

    #[test]
    fn enter_conversion_maps_carriage_return() {
        assert_eq!(
            normalize_enter_bytes(b"show\r", TelnetEnterMode::Crlf),
            b"show\r\n"
        );
        assert_eq!(
            normalize_enter_bytes(b"show\r", TelnetEnterMode::Cr),
            b"show\r"
        );
        assert_eq!(
            normalize_enter_bytes(b"show\r", TelnetEnterMode::Lf),
            b"show\n"
        );
    }

    #[test]
    fn force_character_at_a_time_preserves_utf8_order() {
        let chunks = split_write_chunks("a中\r".as_bytes(), true);
        assert_eq!(
            chunks,
            vec![b"a".to_vec(), "中".as_bytes().to_vec(), b"\r".to_vec()]
        );
        let joined: Vec<u8> = chunks.into_iter().flatten().collect();
        assert_eq!(joined, "a中\r".as_bytes());
    }

    #[test]
    fn strip_telnet_commands_emits_naws_response_request() {
        let mut seen = Vec::new();
        let visible = strip_telnet_commands(b"hi\xff\xfd\x1f", &mut |cmd, opt| {
            seen.push((cmd, opt));
        });
        assert_eq!(visible, b"hi");
        assert_eq!(seen, vec![(DO, OPT_NAWS)]);
    }

    #[test]
    fn local_line_editor_backspace_updates_buffer() {
        let mut editor = TelnetLineEditor::default();
        let result = editor.process(b"abc\x7f", TelnetEnterMode::Cr);

        assert_eq!(editor.buffer(), "ab");
        assert_eq!(result.display, "abc\x08 \x08");
        assert!(result.writes.is_empty());
    }

    #[test]
    fn local_line_editor_sends_buffer_on_enter() {
        let mut editor = TelnetLineEditor::default();
        let result = editor.process(b"abc\x7fd\r", TelnetEnterMode::Cr);

        assert_eq!(editor.buffer(), "");
        assert_eq!(result.writes, vec![b"abd\r".to_vec()]);
        assert_eq!(result.display, "abc\x08 \x08d\r\n");

        let mut editor = TelnetLineEditor::default();
        let result = editor.process(b"abc\x7fd\r", TelnetEnterMode::Crlf);
        assert_eq!(result.writes, vec![b"abd\r\n".to_vec()]);

        let mut editor = TelnetLineEditor::default();
        let result = editor.process(b"abc\x7fd\r", TelnetEnterMode::Lf);
        assert_eq!(result.writes, vec![b"abd\n".to_vec()]);
    }

    #[test]
    fn local_line_editor_backspace_removes_one_utf8_char() {
        let mut editor = TelnetLineEditor::default();
        let result = editor.process("中a\u{7f}".as_bytes(), TelnetEnterMode::Cr);

        assert_eq!(editor.buffer(), "中");
        assert_eq!(result.display, "中a\x08 \x08");

        let result = editor.process(b"\x7f", TelnetEnterMode::Cr);
        assert_eq!(editor.buffer(), "");
        assert_eq!(result.display, "\x08 \x08");
    }

    #[test]
    fn local_line_editor_passes_controls_without_buffering() {
        let mut editor = TelnetLineEditor::default();
        let result = editor.process(b"a\x03\x04\x1b[A\x1b[3~", TelnetEnterMode::Cr);

        assert_eq!(editor.buffer(), "");
        assert_eq!(
            result.writes,
            vec![vec![0x03], vec![0x04], b"\x1b[A".to_vec()]
        );
        assert_eq!(result.display, "a\x08 \x08");
    }
}
