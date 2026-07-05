#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalHistorySearchRequest {
    pub session_id: String,
    pub query: String,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default)]
    pub regex: bool,
    #[serde(default)]
    pub whole_word: bool,
    pub limit: Option<usize>,
    pub context_before: Option<usize>,
    pub context_after: Option<usize>,
    pub max_lines: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalHistorySearchResponse {
    pub total: usize,
    pub elapsed_ms: u128,
    pub truncated: bool,
    pub results: Vec<TerminalHistorySearchResult>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalHistorySearchResult {
    pub line_id: u64,
    pub line_number: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub preview: String,
    pub before: Vec<String>,
    pub after: Vec<String>,
    pub source: String,
}
