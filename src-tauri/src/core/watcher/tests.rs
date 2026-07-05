#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{DataChange, MetadataKind};
    use std::time::UNIX_EPOCH;

    fn fingerprint(
        len: u64,
        modified_secs: u64,
        content_hash: Option<[u8; 32]>,
    ) -> FileFingerprint {
        FileFingerprint {
            len,
            modified: Some(UNIX_EPOCH + Duration::from_secs(modified_secs)),
            content_hash,
        }
    }

    #[test]
    fn metadata_only_event_is_not_content_change_candidate() {
        assert!(!is_content_change_candidate(EventKind::Modify(
            ModifyKind::Metadata(MetadataKind::Any),
        )));
    }

    #[test]
    fn data_event_is_content_change_candidate() {
        assert!(is_content_change_candidate(EventKind::Modify(
            ModifyKind::Data(DataChange::Content),
        )));
    }

    #[test]
    fn same_content_hash_with_changed_mtime_updates_baseline_without_emit() {
        let hash = [7_u8; 32];
        let mut baseline = Some(fingerprint(10, 1, Some(hash)));
        let current = fingerprint(10, 2, Some(hash));

        assert!(!should_emit_for_fingerprint(
            &mut baseline,
            current.clone(),
            false
        ));
        assert_eq!(baseline, Some(current));
    }

    #[test]
    fn changed_content_hash_with_same_size_emits() {
        let mut baseline = Some(fingerprint(10, 1, Some([1_u8; 32])));
        let current = fingerprint(10, 1, Some([2_u8; 32]));

        assert!(should_emit_for_fingerprint(
            &mut baseline,
            current.clone(),
            false
        ));
        assert_eq!(baseline, Some(current));
    }

    #[test]
    fn changed_size_emits_even_without_hash() {
        let mut baseline = Some(fingerprint(10, 1, None));
        let current = fingerprint(11, 1, None);

        assert!(should_emit_for_fingerprint(
            &mut baseline,
            current.clone(),
            true
        ));
        assert_eq!(baseline, Some(current));
    }

    #[test]
    fn oversized_fingerprint_uses_metadata_fallback_after_startup() {
        let mut baseline = Some(fingerprint(10, 1, None));
        let current = fingerprint(10, 2, None);

        assert!(should_emit_for_fingerprint(
            &mut baseline,
            current.clone(),
            false
        ));
        assert_eq!(baseline, Some(current));
    }

    #[test]
    fn oversized_fingerprint_suppresses_same_size_startup_residue() {
        let mut baseline = Some(fingerprint(10, 1, None));
        let current = fingerprint(10, 2, None);

        assert!(!should_emit_for_fingerprint(
            &mut baseline,
            current.clone(),
            true
        ));
        assert_eq!(baseline, Some(current));
    }

    #[test]
    fn repeated_same_fingerprint_does_not_emit_after_baseline_update() {
        let mut baseline = Some(fingerprint(10, 1, Some([1_u8; 32])));
        let current = fingerprint(10, 1, Some([2_u8; 32]));

        assert!(should_emit_for_fingerprint(
            &mut baseline,
            current.clone(),
            false
        ));
        assert!(!should_emit_for_fingerprint(&mut baseline, current, false));
    }

    #[test]
    fn file_hash_is_skipped_above_limit() {
        let path = std::env::temp_dir().join(format!(
            "nyaterm-watcher-test-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::write(&path, b"ab").unwrap();

        let fingerprint = FileFingerprint::from_path_with_hash_limit(&path, 1).unwrap();

        assert_eq!(fingerprint.len, 2);
        assert_eq!(fingerprint.content_hash, None);

        fs::remove_file(path).unwrap();
    }
}
