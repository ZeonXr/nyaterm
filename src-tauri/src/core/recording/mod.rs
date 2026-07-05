use crate::error::{AppError, AppResult};
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::mem;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;
use time::OffsetDateTime;

pub const DEFAULT_MEMORY_LIMIT_BYTES: usize = 5 * 1024 * 1024;
pub const DEFAULT_HISTORY_SEARCH_LINES: usize = 30_000;
pub const MAX_HISTORY_SEARCH_LINES: usize = 100_000;
pub const DEFAULT_HISTORY_SEARCH_LIMIT: usize = 100;

include!("transcript.rs");
include!("search_types.rs");
include!("session_capture.rs");
include!("manager.rs");
include!("format.rs");
include!("search.rs");
include!("tests.rs");
