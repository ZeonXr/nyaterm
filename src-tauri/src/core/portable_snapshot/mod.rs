use crate::config::{
    self, ActivityBarLayout, AppSettings, DiagnosticsSettings, InteractionSettings, SearchSettings,
    TerminalSettings, TransferSettings, TranslationSettings,
};
use crate::error::{AppError, AppResult};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use zip::write::SimpleFileOptions;

use super::{QuickCommandsStore, SessionManager};

const PORTABLE_SNAPSHOT_SCHEMA_VERSION: u32 = 3;
const SNAPSHOT_META_KEY: &str = "meta";
const SNAPSHOT_JSON_PORTABLE_SETTINGS: &str = "portable-settings";
const SNAPSHOT_ZIP_MANIFEST_NAME: &str = "manifest.json";
const SNAPSHOT_ZIP_PAYLOAD_NAME: &str = "snapshot.redb";
const MAX_COMPRESSED_SNAPSHOT_PAYLOAD_BYTES: u64 = 50 * 1024 * 1024;

const SNAPSHOT_META_TABLE: TableDefinition<&str, &str> = TableDefinition::new("snapshot_meta");
const SNAPSHOT_ENTITIES_TABLE: TableDefinition<&str, &str> = TableDefinition::new("entity_docs");
const SNAPSHOT_V2_JSON_DOCS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("json_docs");
const SNAPSHOT_V2_TEXT_DOCS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("text_docs");

include!("types.rs");
include!("build.rs");
include!("codec.rs");
include!("apply.rs");
include!("hash.rs");
include!("legacy.rs");
include!("redb_codec.rs");
include!("temp_redb.rs");
include!("tests.rs");
