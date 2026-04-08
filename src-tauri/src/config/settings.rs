use super::ui::UiConfig;
use super::{default_false, default_true, get_config_dir, load_json, save_json};
use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    #[serde(default = "default_true")]
    pub startup_restore: bool,
    #[serde(default = "default_shell")]
    pub default_local_shell: String,
    #[serde(default = "default_false")]
    pub minimize_to_tray: bool,
    #[serde(default)]
    pub boss_key: Option<String>,
    #[serde(default = "default_true")]
    pub confirm_on_close: bool,
}

fn default_shell() -> String {
    if cfg!(windows) {
        "powershell.exe".to_string()
    } else {
        "bash".to_string()
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            startup_restore: false,
            default_local_shell: default_shell(),
            minimize_to_tray: false,
            boss_key: None,
            confirm_on_close: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    #[serde(default = "default_app_theme")]
    pub theme: String,
    #[serde(default = "default_font")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: f64,
    #[serde(default = "default_false")]
    pub ligatures: bool,
    #[serde(default = "default_opacity")]
    pub background_opacity: f64,
    #[serde(default = "default_cursor_style")]
    pub cursor_style: String,
    #[serde(default = "default_true")]
    pub cursor_blink: bool,
    #[serde(default = "default_ui_font_size")]
    pub ui_font_size: f64,
    #[serde(default)]
    pub terminal_theme: Option<String>,
}

fn default_app_theme() -> String {
    "github-dark".to_string()
}
fn default_font() -> String {
    "JetBrains Mono, 'Noto Sans SC Variable', Consolas, monospace".to_string()
}
fn default_font_size() -> f64 {
    16.0
}
fn default_opacity() -> f64 {
    1.0
}
fn default_cursor_style() -> String {
    "block".to_string()
}
fn default_ui_font_size() -> f64 {
    16.0
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: default_app_theme(),
            font_family: default_font(),
            font_size: default_font_size(),
            ligatures: false,
            background_opacity: default_opacity(),
            cursor_style: default_cursor_style(),
            cursor_blink: true,
            ui_font_size: default_ui_font_size(),
            terminal_theme: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    #[serde(default = "default_false")]
    pub enabled: bool,
    #[serde(default = "default_proxy_protocol")]
    pub protocol: String,
    #[serde(default = "default_proxy_host")]
    pub host: String,
    #[serde(default = "default_proxy_port")]
    pub port: u16,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default, skip_serializing)]
    pub password: Option<String>,
}

fn default_proxy_protocol() -> String {
    "socks5".to_string()
}

fn default_proxy_host() -> String {
    "127.0.0.1".to_string()
}

fn default_proxy_port() -> u16 {
    1080
}

impl Default for ProxySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            protocol: default_proxy_protocol(),
            host: default_proxy_host(),
            port: default_proxy_port(),
            username: None,
            password: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchEngine {
    pub name: String,
    pub url_template: String,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSettings {
    #[serde(default = "default_custom_engines")]
    pub custom_engines: Vec<SearchEngine>,
}

fn default_custom_engines() -> Vec<SearchEngine> {
    vec![
        SearchEngine {
            name: "Google".to_string(),
            url_template: "https://www.google.com/search?q=%s".to_string(),
            icon: Some("google".to_string()),
        },
        SearchEngine {
            name: "Bing".to_string(),
            url_template: "https://www.bing.com/search?q=%s".to_string(),
            icon: Some("bing".to_string()),
        },
        SearchEngine {
            name: "DuckDuckGo".to_string(),
            url_template: "https://duckduckgo.com/?q=%s".to_string(),
            icon: Some("duckduckgo".to_string()),
        },
    ]
}

impl Default for SearchSettings {
    fn default() -> Self {
        Self {
            custom_engines: default_custom_engines(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationSettings {
    #[serde(default = "default_target_language")]
    pub target_language: String,
    #[serde(default)]
    pub deepl_api_key: String,
    #[serde(default)]
    pub baidu_app_id: String,
    #[serde(default)]
    pub baidu_app_key: String,
    #[serde(default)]
    pub ali_app_id: String,
    #[serde(default)]
    pub ali_app_key: String,
    #[serde(default)]
    pub youdao_app_id: String,
    #[serde(default)]
    pub youdao_app_key: String,
}

fn default_target_language() -> String {
    "zh-CN".to_string()
}

impl Default for TranslationSettings {
    fn default() -> Self {
        Self {
            target_language: default_target_language(),
            deepl_api_key: String::new(),
            baidu_app_id: String::new(),
            baidu_app_key: String::new(),
            ali_app_id: String::new(),
            ali_app_key: String::new(),
            youdao_app_id: String::new(),
            youdao_app_key: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    #[serde(default = "default_true")]
    pub use_os_keyring: bool,
    #[serde(default = "default_false")]
    pub require_master_password: bool,
    #[serde(default = "default_false")]
    pub enable_screen_lock: bool,
    #[serde(default)]
    pub idle_lock_minutes: u32,
    #[serde(default)]
    pub lock_password: Option<String>,
    #[serde(default = "default_host_key_policy")]
    pub host_key_policy: String,
}

fn default_host_key_policy() -> String {
    "prompt".to_string()
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            use_os_keyring: true,
            require_master_password: false,
            enable_screen_lock: false,
            idle_lock_minutes: 0,
            lock_password: None,
            host_key_policy: default_host_key_policy(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeywordHighlightRule {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default = "default_highlight_color_dark")]
    pub color_dark: String,
    #[serde(default = "default_highlight_color_light")]
    pub color_light: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_highlight_color_dark() -> String {
    "#79c0ff".to_string()
}
fn default_highlight_color_light() -> String {
    "#0969da".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionLinksMatcherSettings {
    #[serde(default = "default_true")]
    pub ipv4: bool,
    #[serde(default = "default_true")]
    pub archive: bool,
    #[serde(default = "default_true")]
    pub host_port: bool,
}

impl Default for ActionLinksMatcherSettings {
    fn default() -> Self {
        Self {
            ipv4: true,
            archive: true,
            host_port: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSettings {
    #[serde(default = "default_scrollback")]
    pub scrollback_lines: u32,
    #[serde(default = "default_keep_alive")]
    pub keep_alive_interval: u32,
    #[serde(default = "default_false")]
    pub hardware_acceleration: bool,
    #[serde(default = "default_true")]
    pub keyword_highlights_enabled: bool,
    #[serde(default = "default_false")]
    pub keyword_highlights_across_wrapped_lines: bool,
    #[serde(default)]
    pub keyword_highlights: Vec<KeywordHighlightRule>,
    #[serde(default = "default_true")]
    pub action_links_enabled: bool,
    #[serde(default)]
    pub action_links_matchers: ActionLinksMatcherSettings,
}

fn default_scrollback() -> u32 {
    10000
}
fn default_keep_alive() -> u32 {
    60
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            scrollback_lines: default_scrollback(),
            keep_alive_interval: default_keep_alive(),
            hardware_acceleration: false,
            keyword_highlights_enabled: true,
            keyword_highlights_across_wrapped_lines: false,
            keyword_highlights: Vec::new(),
            action_links_enabled: true,
            action_links_matchers: ActionLinksMatcherSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferSettings {
    #[serde(default = "default_transfer_threads")]
    pub download_threads: u32,
    #[serde(default = "default_transfer_threads")]
    pub upload_threads: u32,
    #[serde(default = "default_duplicate_strategy")]
    pub duplicate_strategy: String,
    #[serde(default = "default_true")]
    pub preserve_timestamps: bool,
    #[serde(default = "default_true")]
    pub resume_broken_transfer: bool,
    #[serde(default = "default_file_permissions")]
    pub default_file_permissions: String,
    #[serde(default = "default_max_retries")]
    pub max_transfer_retries: u32,
    #[serde(default = "default_buffer_size")]
    pub transfer_buffer_size: u32,
    #[serde(default)]
    pub download_path: String,
    #[serde(default = "default_false")]
    pub ask_save_location: bool,
    #[serde(default)]
    pub default_editor: String,
    #[serde(default)]
    pub recording_path: String,
}

fn default_transfer_threads() -> u32 {
    3
}
fn default_duplicate_strategy() -> String {
    "overwrite".to_string()
}
fn default_file_permissions() -> String {
    "644".to_string()
}
fn default_max_retries() -> u32 {
    2
}
fn default_buffer_size() -> u32 {
    32
}

impl Default for TransferSettings {
    fn default() -> Self {
        Self {
            download_threads: default_transfer_threads(),
            upload_threads: default_transfer_threads(),
            duplicate_strategy: default_duplicate_strategy(),
            preserve_timestamps: true,
            resume_broken_transfer: true,
            default_file_permissions: default_file_permissions(),
            max_transfer_retries: default_max_retries(),
            transfer_buffer_size: default_buffer_size(),
            download_path: String::new(),
            ask_save_location: false,
            default_editor: String::new(),
            recording_path: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionSettings {
    #[serde(default = "default_true")]
    pub copy_on_select: bool,
    #[serde(default = "default_true")]
    pub right_click_paste: bool,
    #[serde(default = "default_word_separators")]
    pub word_separators: String,
    #[serde(default = "default_encoding")]
    pub default_encoding: String,
}

fn default_word_separators() -> String {
    " ()[]{}\"':=,;|&<>".to_string()
}
fn default_encoding() -> String {
    "UTF-8".to_string()
}

impl Default for InteractionSettings {
    fn default() -> Self {
        Self {
            copy_on_select: false,
            right_click_paste: false,
            word_separators: default_word_separators(),
            default_encoding: default_encoding(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    #[serde(default)]
    pub general: GeneralSettings,
    #[serde(default)]
    pub appearance: AppearanceSettings,
    #[serde(default)]
    pub proxy: ProxySettings,
    #[serde(default)]
    pub search: SearchSettings,
    #[serde(default)]
    pub translation: TranslationSettings,
    #[serde(default)]
    pub security: SecuritySettings,
    #[serde(default)]
    pub terminal: TerminalSettings,
    #[serde(default)]
    pub interaction: InteractionSettings,
    #[serde(default)]
    pub transfer: TransferSettings,
    #[serde(default)]
    pub ui: UiConfig,
}

pub fn load_app_settings(app: &AppHandle) -> AppResult<AppSettings> {
    let dir = get_config_dir(app)?;
    let mut settings: AppSettings = load_json(&dir.join("settings.json"))?;

    let mut migrated = false;

    // Migrate "keyManagement" → "securityAuth" in activity bar layout
    for list in [
        &mut settings.ui.activity_bar_layout.left_top,
        &mut settings.ui.activity_bar_layout.left_bottom,
        &mut settings.ui.activity_bar_layout.right_top,
        &mut settings.ui.activity_bar_layout.right_bottom,
    ] {
        for item in list.iter_mut() {
            if item == "keyManagement" {
                *item = "securityAuth".to_string();
                migrated = true;
            }
        }
    }
    if let Some(ref mut panel) = settings.ui.active_left_panel {
        if panel == "keyManagement" {
            *panel = "securityAuth".to_string();
            migrated = true;
        }
    }

    // Migrate "fileTransfer" out of activity bar (now embedded below file explorer)
    for list in [
        &mut settings.ui.activity_bar_layout.left_top,
        &mut settings.ui.activity_bar_layout.left_bottom,
        &mut settings.ui.activity_bar_layout.right_top,
        &mut settings.ui.activity_bar_layout.right_bottom,
    ] {
        let before = list.len();
        list.retain(|id| id != "fileTransfer");
        if list.len() != before {
            migrated = true;
        }
    }
    if settings.ui.active_left_panel.as_deref() == Some("fileTransfer") {
        settings.ui.active_left_panel = Some("fileExplorer".to_string());
        migrated = true;
    }
    if settings.ui.active_right_panel.as_deref() == Some("fileTransfer") {
        settings.ui.active_right_panel = Some("savedConnections".to_string());
        migrated = true;
    }

    // Ensure "network" is in left_top if not already in any zone
    {
        let all_ids: Vec<&str> = settings
            .ui
            .activity_bar_layout
            .left_top
            .iter()
            .chain(&settings.ui.activity_bar_layout.left_bottom)
            .chain(&settings.ui.activity_bar_layout.right_top)
            .chain(&settings.ui.activity_bar_layout.right_bottom)
            .map(|s| s.as_str())
            .collect();
        if !all_ids.contains(&"network") {
            settings
                .ui
                .activity_bar_layout
                .left_top
                .push("network".to_string());
            migrated = true;
        }
    }

    // Ensure "recording" is in right_bottom if not already in any zone
    {
        let all_ids: Vec<&str> = settings
            .ui
            .activity_bar_layout
            .left_top
            .iter()
            .chain(&settings.ui.activity_bar_layout.left_bottom)
            .chain(&settings.ui.activity_bar_layout.right_top)
            .chain(&settings.ui.activity_bar_layout.right_bottom)
            .map(|s| s.as_str())
            .collect();
        if !all_ids.contains(&"recording") {
            settings
                .ui
                .activity_bar_layout
                .right_bottom
                .insert(1, "recording".to_string());
            migrated = true;
        }
    }

    if migrated {
        let _ = save_app_settings(app, &settings);
    }

    Ok(settings)
}

pub fn save_app_settings(app: &AppHandle, config: &AppSettings) -> AppResult<()> {
    let dir = get_config_dir(app)?;
    save_json(&dir.join("settings.json"), config)
}
