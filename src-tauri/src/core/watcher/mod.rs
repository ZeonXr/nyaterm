use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tauri::{AppHandle, Emitter};

use crate::error::{AppError, AppResult};
use crate::observability::{StructuredLog, StructuredLogLevel, log_event, log_rate_limited};

include!("types.rs");
include!("fingerprint.rs");
include!("manager.rs");
include!("tests.rs");
