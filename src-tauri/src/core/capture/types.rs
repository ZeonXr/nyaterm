pub struct CapturedOutput {
    pub output: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
}

const MARKER_PREFIX: &str = "__DF_CMD_";
