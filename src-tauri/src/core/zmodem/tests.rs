#[cfg(test)]
mod tests {
    use super::{
        ProgressThrottle, ZMODEM_PROGRESS_BYTES, ZMODEM_PROGRESS_INTERVAL, ZmodemDetectResult,
        ZmodemDetector, ZmodemDirection,
    };
    use std::time::{Duration, Instant};

    fn detected_direction(result: ZmodemDetectResult) -> ZmodemDirection {
        match result {
            ZmodemDetectResult::Detected { direction, .. } => direction,
            ZmodemDetectResult::NoMatch { .. } => panic!("expected ZMODEM detection"),
        }
    }

    #[test]
    fn detects_complete_zhex_download_header() {
        let mut detector = ZmodemDetector::new();
        let result = detector.feed(b"ready\r\n**\x18B00");

        match result {
            ZmodemDetectResult::Detected {
                direction,
                passthrough,
                initial_bytes,
            } => {
                assert_eq!(direction, ZmodemDirection::Download);
                assert_eq!(passthrough, b"ready\r\n");
                assert_eq!(initial_bytes, b"**\x18B00");
            }
            ZmodemDetectResult::NoMatch { .. } => panic!("expected ZMODEM detection"),
        }
    }

    #[test]
    fn detects_zbin_upload_header_across_chunks() {
        let mut detector = ZmodemDetector::new();
        match detector.feed(b"prefix\r\n**\x18") {
            ZmodemDetectResult::NoMatch { passthrough } => {
                assert_eq!(passthrough, b"prefix\r\n")
            }
            ZmodemDetectResult::Detected { .. } => panic!("unexpected early detection"),
        }

        let result = detector.feed(b"A\x01payload");
        match result {
            ZmodemDetectResult::Detected {
                direction,
                passthrough,
                initial_bytes,
            } => {
                assert_eq!(direction, ZmodemDirection::Upload);
                assert!(passthrough.is_empty());
                assert_eq!(initial_bytes, b"**\x18A\x01payload");
            }
            ZmodemDetectResult::NoMatch { .. } => panic!("expected ZMODEM detection"),
        }
    }

    #[test]
    fn detects_rz_zhex_upload_header_after_prompt_text() {
        let mut detector = ZmodemDetector::new();
        let result = detector.feed(b"\x18z waiting to receive.**\x18B01");

        match result {
            ZmodemDetectResult::Detected {
                direction,
                passthrough,
                initial_bytes,
            } => {
                assert_eq!(direction, ZmodemDirection::Upload);
                assert_eq!(passthrough, b"\x18z waiting to receive.");
                assert_eq!(initial_bytes, b"**\x18B01");
            }
            ZmodemDetectResult::NoMatch { .. } => panic!("expected ZMODEM detection"),
        }
    }

    #[test]
    fn detects_rz_zhex_upload_header_split_after_prompt_text() {
        let mut detector = ZmodemDetector::new();
        match detector.feed(b"\x18z waiting to receive.**") {
            ZmodemDetectResult::NoMatch { passthrough } => {
                assert_eq!(passthrough, b"\x18z waiting to receive.")
            }
            ZmodemDetectResult::Detected { .. } => panic!("unexpected early detection"),
        }

        assert_eq!(
            detected_direction(detector.feed(b"\x18B01")),
            ZmodemDirection::Upload
        );
    }

    #[test]
    fn detects_zhex_frame_type_split_after_first_hex_digit() {
        let mut detector = ZmodemDetector::new();
        match detector.feed(b"**\x18B0") {
            ZmodemDetectResult::NoMatch { passthrough } => assert!(passthrough.is_empty()),
            ZmodemDetectResult::Detected { .. } => panic!("unexpected early detection"),
        }

        assert_eq!(
            detected_direction(detector.feed(b"1rest")),
            ZmodemDirection::Upload
        );
    }

    #[test]
    fn passthroughs_interactive_asterisks_immediately() {
        let mut detector = ZmodemDetector::new();
        let chunks: [&[u8]; 4] = [b"docker", b"*", b"*", b"*"];
        let mut visible = Vec::new();

        for chunk in chunks {
            match detector.feed(chunk) {
                ZmodemDetectResult::NoMatch { passthrough } => {
                    visible.extend_from_slice(&passthrough);
                }
                ZmodemDetectResult::Detected { .. } => panic!("unexpected ZMODEM detection"),
            }
        }

        assert_eq!(visible, b"docker***");
    }

    #[test]
    fn waits_for_zmodem_header_after_zdle_even_inside_text() {
        let mut detector = ZmodemDetector::new();
        match detector.feed(b"prefix**\x18") {
            ZmodemDetectResult::NoMatch { passthrough } => {
                assert_eq!(passthrough, b"prefix")
            }
            ZmodemDetectResult::Detected { .. } => panic!("unexpected ZMODEM detection"),
        }

        assert_eq!(
            detected_direction(detector.feed(b"A\x01payload")),
            ZmodemDirection::Upload
        );
    }

    #[test]
    fn progress_throttle_emits_first_time() {
        let mut throttle = ProgressThrottle::new();
        assert!(throttle.should_emit_at(0, false, Instant::now()));
    }

    #[test]
    fn progress_throttle_respects_time_and_byte_thresholds() {
        let mut throttle = ProgressThrottle::new();
        let start = Instant::now();
        assert!(throttle.should_emit_at(0, false, start));
        assert!(!throttle.should_emit_at(1, false, start + Duration::from_millis(10)));
        assert!(throttle.should_emit_at(
            ZMODEM_PROGRESS_BYTES,
            false,
            start + Duration::from_millis(20)
        ));
        assert!(throttle.should_emit_at(
            ZMODEM_PROGRESS_BYTES + 1,
            false,
            start + ZMODEM_PROGRESS_INTERVAL + Duration::from_millis(30)
        ));
    }

    #[test]
    fn progress_throttle_force_emits_completion() {
        let mut throttle = ProgressThrottle::new();
        let start = Instant::now();
        assert!(throttle.should_emit_at(128, false, start));
        assert!(throttle.should_emit_at(129, true, start + Duration::from_millis(1)));
    }
}
