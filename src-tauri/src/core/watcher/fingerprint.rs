#[derive(Debug, Clone, PartialEq, Eq)]
struct FileFingerprint {
    len: u64,
    modified: Option<SystemTime>,
    content_hash: Option<[u8; 32]>,
}

#[derive(Debug, PartialEq, Eq)]
enum FingerprintChange {
    Unchanged,
    BaselineOnly,
    ContentChanged,
}

impl FileFingerprint {
    fn from_path(path: &Path) -> io::Result<Self> {
        Self::from_path_with_hash_limit(path, CONTENT_HASH_LIMIT_BYTES)
    }

    fn from_path_with_hash_limit(path: &Path, hash_limit_bytes: u64) -> io::Result<Self> {
        let metadata = fs::metadata(path)?;
        let len = metadata.len();
        let modified = metadata.modified().ok();
        let content_hash = if metadata.is_file() && len <= hash_limit_bytes {
            Some(hash_file(path)?)
        } else {
            None
        };

        Ok(Self {
            len,
            modified,
            content_hash,
        })
    }
}

fn hash_file(path: &Path) -> io::Result<[u8; 32]> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let digest = hasher.finalize();
    let mut hash = [0_u8; 32];
    hash.copy_from_slice(&digest);
    Ok(hash)
}

fn is_content_change_candidate(kind: EventKind) -> bool {
    matches!(
        kind,
        EventKind::Modify(
            ModifyKind::Data(_) | ModifyKind::Any | ModifyKind::Name(_) | ModifyKind::Other,
        )
    )
}

fn classify_fingerprint_change(
    previous: &FileFingerprint,
    current: &FileFingerprint,
    within_startup_window: bool,
) -> FingerprintChange {
    if let (Some(previous_hash), Some(current_hash)) =
        (&previous.content_hash, &current.content_hash)
    {
        return if previous_hash != current_hash {
            FingerprintChange::ContentChanged
        } else if previous != current {
            FingerprintChange::BaselineOnly
        } else {
            FingerprintChange::Unchanged
        };
    }

    if previous.len != current.len {
        return FingerprintChange::ContentChanged;
    }

    if previous.modified != current.modified {
        return if within_startup_window {
            FingerprintChange::BaselineOnly
        } else {
            FingerprintChange::ContentChanged
        };
    }

    FingerprintChange::Unchanged
}

fn should_emit_for_fingerprint(
    baseline: &mut Option<FileFingerprint>,
    current: FileFingerprint,
    within_startup_window: bool,
) -> bool {
    let Some(previous) = baseline.as_ref() else {
        *baseline = Some(current);
        return false;
    };

    match classify_fingerprint_change(previous, &current, within_startup_window) {
        FingerprintChange::ContentChanged => {
            *baseline = Some(current);
            true
        }
        FingerprintChange::BaselineOnly => {
            *baseline = Some(current);
            false
        }
        FingerprintChange::Unchanged => false,
    }
}

