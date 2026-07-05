
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_JSON: &str = r#"
{
  "version": 1,
  "passwords": [
    { "ref": "prod-root-password", "name": "Prod root password", "password": "replace-me" }
  ],
  "ssh_keys": [
    {
      "ref": "ops-ed25519",
      "name": "Ops ED25519",
      "private_key": "-----BEGIN OPENSSH PRIVATE KEY-----\n...\n-----END OPENSSH PRIVATE KEY-----",
      "passphrase": "optional-passphrase"
    }
  ],
  "groups": [
    { "path": ["Production"] },
    { "path": ["Production", "Web"] },
    { "path": ["Lab"] }
  ],
  "sessions": [
    {
      "name": "Prod web direct password",
      "type": "ssh",
      "group_path": ["Production", "Web"],
      "host": "web-01.example.com",
      "port": 22,
      "username": "deploy",
      "auth": { "mode": "password", "password": "replace-me" }
    },
    {
      "name": "Prod db saved password",
      "type": "ssh",
      "group_path": ["Production", "Database"],
      "host": "db-01.example.com",
      "username": "root",
      "auth": { "mode": "password", "password_ref": "prod-root-password" }
    },
    {
      "name": "Bastion saved key",
      "type": "ssh",
      "group_path": ["Production"],
      "host": "bastion.example.com",
      "username": "ops",
      "auth": { "mode": "key", "key_ref": "ops-ed25519" }
    },
    {
      "name": "Lab router",
      "type": "telnet",
      "group_path": ["Lab"],
      "host": "192.168.10.1",
      "port": 23,
      "backspace_mode": "del"
    },
    {
      "name": "USB console",
      "type": "serial",
      "group_path": ["Lab"],
      "port_name": "COM3",
      "baud_rate": 115200,
      "data_bits": 8,
      "parity": "none",
      "stop_bits": "1",
      "backspace_mode": "ctrl_h"
    },
    {
      "name": "Local PowerShell",
      "type": "local_terminal",
      "shell_path": "pwsh.exe",
      "shell_args": "-NoLogo",
      "working_dir": "C:\\Users\\me"
    }
  ]
}
"#;

    #[test]
    fn windterm_import_splits_user_at_host_targets() {
        let sessions = parse_windterm_content(
            r#"
[
  {
    "session.protocol": "SSH",
    "session.target": "deploy@192.168.1.10",
    "session.label": "Prod web",
    "session.port": 2222
  }
]
"#,
        )
        .expect("parse windterm sessions");

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "Prod web");
        assert_eq!(sessions[0].host, "192.168.1.10");
        assert_eq!(sessions[0].username, "deploy");
        assert_eq!(sessions[0].port, 2222);
    }

    #[test]
    fn windterm_import_defaults_username_when_target_has_no_user() {
        let sessions = parse_windterm_content(
            r#"
[
  {
    "session.protocol": "SSH",
    "session.target": "192.168.1.10"
  }
]
"#,
        )
        .expect("parse windterm sessions");

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].host, "192.168.1.10");
        assert_eq!(sessions[0].username, "root");
    }

    #[test]
    fn windterm_target_rejects_empty_user_or_host_splits() {
        assert_eq!(
            parse_windterm_target("@192.168.1.10"),
            ("@192.168.1.10".to_string(), "root".to_string())
        );
        assert_eq!(
            parse_windterm_target("deploy@"),
            ("deploy@".to_string(), "root".to_string())
        );
    }

    #[test]
    fn windterm_target_splits_on_last_at_symbol() {
        assert_eq!(
            parse_windterm_target("ops@team@example.com"),
            ("example.com".to_string(), "ops@team".to_string())
        );
    }

    #[test]
    fn nyaterm_json_sample_import_prepares_supported_shapes() {
        crate::utils::crypto::set_master_password(None);

        let prepared = parse_nyaterm_json_content(SAMPLE_JSON).expect("parse sample");

        assert_eq!(prepared.groups.len(), 3);
        assert_eq!(prepared.passwords.len(), 1);
        assert_eq!(prepared.ssh_keys.len(), 1);
        assert_eq!(prepared.connections.len(), 6);
        assert_ne!(
            prepared.passwords[0].password.as_deref(),
            Some("replace-me")
        );
        assert_ne!(
            prepared.ssh_keys[0].key.as_deref(),
            Some("-----BEGIN OPENSSH PRIVATE KEY-----\n...\n-----END OPENSSH PRIVATE KEY-----")
        );

        let direct_auth = prepared.connections[0].auth.as_ref().expect("direct auth");
        assert_eq!(direct_auth.mode, "password");
        assert!(direct_auth.password_id.is_none());
        assert_ne!(direct_auth.password.as_deref(), Some("replace-me"));

        let saved_password_auth = prepared.connections[1]
            .auth
            .as_ref()
            .expect("saved password auth");
        assert_eq!(saved_password_auth.mode, "password");
        assert!(saved_password_auth.password_id.is_some());
        assert!(saved_password_auth.password.is_none());

        let key_auth = prepared.connections[2].auth.as_ref().expect("key auth");
        assert_eq!(key_auth.mode, "key");
        assert!(key_auth.key_id.is_some());

        let local_config = &prepared.connections[5].config;
        assert!(matches!(
            local_config,
            ConnectionType::LocalTerminal {
                shell_path,
                shell_args,
                ..
            } if shell_path == "pwsh.exe" && shell_args == "-NoLogo"
        ));
    }

    #[test]
    fn nyaterm_json_rejects_duplicate_password_refs() {
        let json = r#"
{
  "version": 1,
  "passwords": [
    { "ref": "dup", "name": "One", "password": "a" },
    { "ref": "dup", "name": "Two", "password": "b" }
  ],
  "sessions": []
}
"#;

        let error = parse_nyaterm_json_content(json).unwrap_err();
        assert!(error.to_string().contains("Duplicate password ref"));
    }

    #[test]
    fn nyaterm_json_rejects_missing_password_refs() {
        let json = r#"
{
  "version": 1,
  "sessions": [
    {
      "name": "Missing password",
      "type": "ssh",
      "host": "example.com",
      "username": "root",
      "auth": { "mode": "password", "password_ref": "missing" }
    }
  ]
}
"#;

        let error = parse_nyaterm_json_content(json).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("password_ref 'missing' was not found")
        );
    }

    #[test]
    fn nyaterm_json_rejects_invalid_ports() {
        let json = r#"
{
  "version": 1,
  "sessions": [
    {
      "name": "Bad port",
      "type": "ssh",
      "host": "example.com",
      "port": 0,
      "username": "root",
      "auth": { "mode": "none" }
    }
  ]
}
"#;

        let error = parse_nyaterm_json_content(json).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("port must be between 1 and 65535")
        );
    }
}
