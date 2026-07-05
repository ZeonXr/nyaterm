//! Import sessions from Xshell (.xts), MobaXterm (.mxtsessions), WindTerm (.sessions),
//! and NyaTerm JSON files.

use crate::config::{
    self, AiExecutionProfile, ConnectionAuth, ConnectionType, Group, SavedConnection,
};
use crate::error::{AppError, AppResult};
use crate::utils::crypto;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use tauri::Emitter;

include!("types.rs");
include!("text.rs");
include!("common.rs");
include!("xshell.rs");
include!("mobaxterm.rs");
include!("windterm.rs");
include!("nyaterm_json.rs");
include!("merge.rs");
include!("tests.rs");
