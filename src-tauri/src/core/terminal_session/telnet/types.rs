#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelnetEnterMode {
    Crlf,
    Cr,
    Lf,
}

impl Default for TelnetEnterMode {
    fn default() -> Self {
        Self::Cr
    }
}

impl TelnetEnterMode {
    pub fn from_config_value(value: &str) -> Self {
        match value {
            "crlf" => Self::Crlf,
            "lf" => Self::Lf,
            _ => Self::Cr,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TelnetSessionConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub backspace_mode: String,
    pub raw_tcp_cli: bool,
    pub enter_mode: TelnetEnterMode,
    pub local_echo: bool,
    pub local_line_edit: bool,
    pub force_character_at_a_time: bool,
    pub send_naws: bool,
    pub send_sga: bool,
}

impl Default for TelnetSessionConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 23,
            name: "Telnet".to_string(),
            backspace_mode: "del".to_string(),
            raw_tcp_cli: false,
            enter_mode: TelnetEnterMode::Cr,
            local_echo: false,
            local_line_edit: false,
            force_character_at_a_time: false,
            send_naws: true,
            send_sga: true,
        }
    }
}

