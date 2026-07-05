#[derive(Clone, Serialize)]
pub struct FileModifiedPayload {
    pub session_id: String,
    pub local_path: String,
    pub remote_path: String,
}

struct WatchState {
    _watcher: Option<RecommendedWatcher>,
}

static ACTIVE_WATCHERS: LazyLock<Arc<Mutex<HashMap<String, WatchState>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

const CONTENT_HASH_LIMIT_BYTES: u64 = 64 * 1024 * 1024;
const STARTUP_SUPPRESSION_WINDOW: Duration = Duration::from_secs(2);
const WATCH_DEBOUNCE: Duration = Duration::from_millis(500);

