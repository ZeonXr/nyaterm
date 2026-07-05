pub async fn start_file_watch(
    app: AppHandle,
    session_id: String,
    local_path: String,
    remote_path: String,
) -> AppResult<()> {
    // Generate a unique key for this watch instance
    let watch_key = format!("{session_id}:{local_path}");
    let mut watchers = ACTIVE_WATCHERS.lock().unwrap();

    // If we are already watching this file, don't start another watcher
    if watchers.contains_key(&watch_key) {
        return Ok(());
    }

    let (tx, rx) = channel();

    // Create the watcher (this has to happen synchronously or via std thread)
    let mut watcher = notify::recommended_watcher(tx)
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;

    watcher
        .watch(Path::new(&local_path), RecursiveMode::NonRecursive)
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;

    // Store the watcher to keep it alive
    watchers.insert(
        watch_key.clone(),
        WatchState {
            _watcher: Some(watcher),
        },
    );

    let app_clone = app.clone();
    let session_id_clone = session_id.clone();
    let local_path_clone = local_path.clone();
    let remote_path_clone = remote_path.clone();
    let watched_path = PathBuf::from(local_path_clone.clone());

    log_event(StructuredLog {
        level: StructuredLogLevel::Info,
        domain: "watcher.sync".to_string(),
        event: "watch.start".to_string(),
        message: "Starting file watch".to_string(),
        ids: Some(serde_json::json!({ "session_id": session_id })),
        data: Some(serde_json::json!({
            "local_path": local_path,
            "remote_path": remote_path,
        })),
        error: None,
        client_timestamp: None,
    });

    // Spawn a blocking thread to listen for notify events
    std::thread::spawn(move || {
        let watch_started = Instant::now();
        let mut baseline = match FileFingerprint::from_path(&watched_path) {
            Ok(fingerprint) => Some(fingerprint),
            Err(e) => {
                log_rate_limited(StructuredLog {
                    level: StructuredLogLevel::Error,
                    domain: "watcher.sync".to_string(),
                    event: "watch.baseline_failed".to_string(),
                    message: "Failed to read file watch baseline".to_string(),
                    ids: Some(serde_json::json!({ "session_id": session_id_clone.clone() })),
                    data: Some(serde_json::json!({
                        "local_path": local_path_clone.clone(),
                        "remote_path": remote_path_clone.clone(),
                    })),
                    error: Some(serde_json::json!({ "message": e.to_string() })),
                    client_timestamp: None,
                });
                None
            }
        };
        let now = Instant::now();
        let mut last_check = now.checked_sub(WATCH_DEBOUNCE).unwrap_or(now);
        for res in rx {
            match res {
                Ok(event) => {
                    tracing::debug!(kind = ?event.kind, "Notify event received");

                    // Most text editors do atomic saves (save to temp file, then rename/move)
                    // We only emit when the watched file's content fingerprint actually changes.
                    if is_content_change_candidate(event.kind) {
                        tracing::debug!(paths = ?event.paths, "Detected candidate content event");
                        // Debounce: prevent checking multiple times for a single save operation (common in editors)
                        if last_check.elapsed() > WATCH_DEBOUNCE {
                            last_check = Instant::now();
                            match FileFingerprint::from_path(&watched_path) {
                                Ok(current) => {
                                    let should_emit = should_emit_for_fingerprint(
                                        &mut baseline,
                                        current,
                                        watch_started.elapsed() <= STARTUP_SUPPRESSION_WINDOW,
                                    );
                                    if !should_emit {
                                        tracing::debug!(
                                            "File fingerprint unchanged, skipping file-modified emit"
                                        );
                                        continue;
                                    }
                                }
                                Err(e) => {
                                    log_rate_limited(StructuredLog {
                                        level: StructuredLogLevel::Error,
                                        domain: "watcher.sync".to_string(),
                                        event: "watch.fingerprint_failed".to_string(),
                                        message: "Failed to read file watch fingerprint"
                                            .to_string(),
                                        ids: Some(serde_json::json!({
                                            "session_id": session_id_clone.clone()
                                        })),
                                        data: Some(serde_json::json!({
                                            "local_path": local_path_clone.clone(),
                                            "remote_path": remote_path_clone.clone(),
                                        })),
                                        error: Some(serde_json::json!({
                                            "message": e.to_string()
                                        })),
                                        client_timestamp: None,
                                    });
                                    continue;
                                }
                            }

                            tracing::debug!("File content changed, emitting file-modified payload");
                            let payload = FileModifiedPayload {
                                session_id: session_id_clone.clone(),
                                local_path: local_path_clone.clone(),
                                remote_path: remote_path_clone.clone(),
                            };
                            if let Err(e) = app_clone.emit("file-modified", payload) {
                                log_rate_limited(StructuredLog {
                                    level: StructuredLogLevel::Error,
                                    domain: "watcher.sync".to_string(),
                                    event: "watch.emit_failed".to_string(),
                                    message: "Failed to emit file-modified event".to_string(),
                                    ids: Some(
                                        serde_json::json!({ "session_id": session_id_clone.clone() }),
                                    ),
                                    data: Some(serde_json::json!({
                                        "local_path": local_path_clone.clone(),
                                        "remote_path": remote_path_clone.clone(),
                                    })),
                                    error: Some(serde_json::json!({ "message": e.to_string() })),
                                    client_timestamp: None,
                                });
                            }
                        } else {
                            tracing::debug!("Watcher event debounced");
                        }
                    }
                }
                Err(e) => {
                    log_rate_limited(StructuredLog {
                        level: StructuredLogLevel::Error,
                        domain: "watcher.sync".to_string(),
                        event: "watch.backend_error".to_string(),
                        message: "File watcher backend error".to_string(),
                        ids: Some(serde_json::json!({ "session_id": session_id_clone.clone() })),
                        data: Some(serde_json::json!({
                            "local_path": local_path_clone.clone(),
                            "remote_path": remote_path_clone.clone(),
                        })),
                        error: Some(serde_json::json!({ "message": e.to_string() })),
                        client_timestamp: None,
                    });
                    break;
                }
            }
        }
    });

    Ok(())
}

#[allow(clippy::unused_async)]
pub async fn stop_file_watch(session_id: String, local_path: String) -> AppResult<()> {
    let watch_key = format!("{session_id}:{local_path}");
    let mut watchers = ACTIVE_WATCHERS.lock().unwrap();
    watchers.remove(&watch_key);
    Ok(())
}
