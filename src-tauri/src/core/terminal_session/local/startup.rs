struct LocalStartupScript {
    script: Option<String>,
    shell_integration_active: bool,
}

fn build_local_startup_script(_shell_name: &str, _ready_marker: &str) -> LocalStartupScript {
    build_local_startup_script_for_platform(
        _shell_name,
        _ready_marker,
        cfg!(not(target_os = "windows")),
    )
}

fn build_local_startup_script_for_platform(
    _shell_name: &str,
    _ready_marker: &str,
    _allow_unix_prelude: bool,
) -> LocalStartupScript {
    LocalStartupScript {
        script: None,
        shell_integration_active: false,
    }
}

fn should_emit_visible_output(suppress_visible: &mut bool, ready: bool) -> bool {
    if !*suppress_visible {
        return true;
    }

    if !ready {
        return false;
    }

    *suppress_visible = false;
    true
}

fn write_to_pty(writer: &mut dyn Write, data: &[u8]) -> std::io::Result<()> {
    writer.write_all(data)?;
    writer.flush()
}

