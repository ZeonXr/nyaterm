use crate::core::SessionManager;
use crate::core::monitoring::docker::{
    DOCKER_COMPOSE_PROJECTS_SCRIPT, DOCKER_IMAGES_SCRIPT, DOCKER_NETWORKS_SCRIPT,
    DOCKER_OVERVIEW_SCRIPT, DOCKER_VOLUMES_SCRIPT, DockerComposeProject, DockerComposeService,
    DockerContainerDetails, DockerContainerStats, DockerImage, DockerNetwork, DockerVolume,
    RemoteDockerOverview, docker_container_details_script, parse_compose_projects,
    parse_compose_services_output, parse_docker_container_details_output,
    parse_docker_images_output, parse_docker_networks_output, parse_docker_overview_output,
    parse_docker_stats_output, parse_docker_volumes_output,
};
use crate::core::remote_exec::{
    RemoteCommandOutput, ensure_success, exec_ssh_session_command, sh_quote,
};
use crate::error::{AppError, AppResult};
use std::sync::Arc;
use std::time::Duration;

const DOCKER_TIMEOUT: Duration = Duration::from_secs(20);
const DOCKER_LOG_TIMEOUT: Duration = Duration::from_secs(30);
const COMPOSE_PS_JSON_BEGIN: &str = "COMPOSE_PS_JSON_BEGIN";
const COMPOSE_PS_JSON_END: &str = "COMPOSE_PS_JSON_END";

#[tauri::command]
pub async fn get_remote_docker_overview(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
) -> AppResult<RemoteDockerOverview> {
    let output = exec_ssh_session_command(
        state.inner(),
        &session_id,
        DOCKER_OVERVIEW_SCRIPT.as_bytes(),
        DOCKER_TIMEOUT,
    )
    .await?;

    Ok(parse_docker_overview_output(&output.stdout))
}

#[tauri::command]
pub async fn get_remote_docker_images(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
) -> AppResult<Vec<DockerImage>> {
    let output = exec_ssh_session_command(
        state.inner(),
        &session_id,
        DOCKER_IMAGES_SCRIPT.as_bytes(),
        DOCKER_TIMEOUT,
    )
    .await?;
    let output = ensure_success(output, "Docker images failed")?;

    Ok(parse_docker_images_output(&output.stdout))
}

#[tauri::command]
pub async fn get_remote_docker_volumes(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
) -> AppResult<Vec<DockerVolume>> {
    let output = exec_ssh_session_command(
        state.inner(),
        &session_id,
        DOCKER_VOLUMES_SCRIPT.as_bytes(),
        DOCKER_TIMEOUT,
    )
    .await?;
    let output = ensure_success(output, "Docker volumes failed")?;

    Ok(parse_docker_volumes_output(&output.stdout))
}

#[tauri::command]
pub async fn get_remote_docker_networks(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
) -> AppResult<Vec<DockerNetwork>> {
    let output = exec_ssh_session_command(
        state.inner(),
        &session_id,
        DOCKER_NETWORKS_SCRIPT.as_bytes(),
        DOCKER_TIMEOUT,
    )
    .await?;
    let output = ensure_success(output, "Docker networks failed")?;

    Ok(parse_docker_networks_output(&output.stdout))
}

#[tauri::command]
pub async fn get_remote_docker_compose_projects(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
) -> AppResult<Vec<DockerComposeProject>> {
    let output = exec_ssh_session_command(
        state.inner(),
        &session_id,
        DOCKER_COMPOSE_PROJECTS_SCRIPT.as_bytes(),
        DOCKER_TIMEOUT,
    )
    .await?;
    let output = ensure_success(output, "Docker compose projects failed")?;

    Ok(parse_compose_projects(&output.stdout))
}

#[tauri::command]
pub async fn get_docker_container_details(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
    container_id: String,
) -> AppResult<DockerContainerDetails> {
    let command = docker_container_details_script(&container_id);
    let output = exec_ssh_session_command(
        state.inner(),
        &session_id,
        command.as_bytes(),
        DOCKER_TIMEOUT,
    )
    .await?;
    let output = ensure_success(output, "Docker container details failed")?;

    Ok(parse_docker_container_details_output(&output.stdout))
}

#[tauri::command]
pub async fn get_docker_container_stats(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
    container_id: String,
) -> AppResult<Option<DockerContainerStats>> {
    let command = format!(
        "docker stats --no-stream --no-trunc --format \"CONTAINER_STATS\\t{{{{.ID}}}}\\t{{{{.CPUPerc}}}}\\t{{{{.MemUsage}}}}\\t{{{{.MemPerc}}}}\\t{{{{.NetIO}}}}\\t{{{{.BlockIO}}}}\\t{{{{.PIDs}}}}\" {}",
        sh_quote(&container_id)
    );
    let output = exec_ssh_session_command(
        state.inner(),
        &session_id,
        command.as_bytes(),
        DOCKER_TIMEOUT,
    )
    .await?;
    let output = ensure_success(output, "Docker container stats failed")?;

    Ok(parse_docker_stats_output(&output.stdout).into_iter().next())
}

#[tauri::command]
pub async fn docker_container_action(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
    container_id: String,
    action: String,
) -> AppResult<RemoteCommandOutput> {
    let action = normalize_container_action(&action)?;
    let command = format!("docker {action} {}", sh_quote(&container_id));
    run_docker_action(
        state.inner(),
        &session_id,
        &command,
        "Docker container action failed",
    )
    .await
}

#[tauri::command]
pub async fn docker_image_remove(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
    image_id: String,
    force: bool,
) -> AppResult<RemoteCommandOutput> {
    let force_arg = if force { " -f" } else { "" };
    let command = format!("docker image rm{force_arg} {}", sh_quote(&image_id));
    run_docker_action(
        state.inner(),
        &session_id,
        &command,
        "Docker image remove failed",
    )
    .await
}

#[tauri::command]
pub async fn docker_volume_remove(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
    volume_name: String,
    force: bool,
) -> AppResult<RemoteCommandOutput> {
    let force_arg = if force { " -f" } else { "" };
    let command = format!("docker volume rm{force_arg} {}", sh_quote(&volume_name));
    run_docker_action(
        state.inner(),
        &session_id,
        &command,
        "Docker volume remove failed",
    )
    .await
}

#[tauri::command]
pub async fn docker_network_remove(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
    network_id: String,
) -> AppResult<RemoteCommandOutput> {
    let command = format!("docker network rm {}", sh_quote(&network_id));
    run_docker_action(
        state.inner(),
        &session_id,
        &command,
        "Docker network remove failed",
    )
    .await
}

#[tauri::command]
pub async fn docker_system_prune(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
    volumes: bool,
) -> AppResult<RemoteCommandOutput> {
    let volumes_arg = if volumes { " --volumes" } else { "" };
    let command = format!("docker system prune -f{volumes_arg}");
    run_docker_action(
        state.inner(),
        &session_id,
        &command,
        "Docker system prune failed",
    )
    .await
}

#[tauri::command]
pub async fn get_docker_container_logs(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
    container_id: String,
    tail: u32,
) -> AppResult<RemoteCommandOutput> {
    let tail = tail.clamp(10, 2000);
    let command = format!("docker logs --tail {tail} {}", sh_quote(&container_id));
    exec_ssh_session_command(
        state.inner(),
        &session_id,
        command.as_bytes(),
        DOCKER_LOG_TIMEOUT,
    )
    .await
}

#[tauri::command]
pub async fn docker_compose_action(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
    project_name: String,
    config_files: Option<String>,
    action: String,
) -> AppResult<RemoteCommandOutput> {
    let action = normalize_compose_action(&action)?;
    let mut command = build_compose_base_command(&project_name, config_files.as_deref());
    command.push(' ');
    command.push_str(action);
    if action == "up" {
        command.push_str(" -d");
    }

    run_docker_action(
        state.inner(),
        &session_id,
        &command,
        "Docker compose action failed",
    )
    .await
}

#[tauri::command]
pub async fn get_docker_compose_services(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
    project_name: String,
    config_files: Option<String>,
) -> AppResult<Vec<DockerComposeService>> {
    let base = build_compose_base_command(&project_name, config_files.as_deref());
    let command = format!(
        "services_output=$({base} config --services) || exit $?; \
         printf '%s\\n' \"$services_output\"; \
         printf '\\n{COMPOSE_PS_JSON_BEGIN}\\n'; \
         {base} ps --all --format json 2>/dev/null || true; \
         printf '\\n{COMPOSE_PS_JSON_END}\\n'"
    );
    let output = exec_ssh_session_command(
        state.inner(),
        &session_id,
        command.as_bytes(),
        DOCKER_TIMEOUT,
    )
    .await?;
    let output = ensure_success(output, "Docker compose services failed")?;
    let (services_raw, ps_json_raw) = split_compose_services_output(&output.stdout);

    Ok(parse_compose_services_output(&services_raw, &ps_json_raw))
}

#[tauri::command]
pub async fn docker_compose_service_action(
    state: tauri::State<'_, Arc<SessionManager>>,
    session_id: String,
    project_name: String,
    config_files: Option<String>,
    service_name: String,
    action: String,
) -> AppResult<RemoteCommandOutput> {
    let action = normalize_compose_service_action(&action)?;
    let mut command = build_compose_base_command(&project_name, config_files.as_deref());
    command.push(' ');
    command.push_str(action);
    if action == "up" {
        command.push_str(" -d");
    }
    command.push(' ');
    command.push_str(&sh_quote(&service_name));

    run_docker_action(
        state.inner(),
        &session_id,
        &command,
        "Docker compose service action failed",
    )
    .await
}

async fn run_docker_action(
    manager: &Arc<SessionManager>,
    session_id: &str,
    command: &str,
    context: &str,
) -> AppResult<RemoteCommandOutput> {
    let output =
        exec_ssh_session_command(manager, session_id, command.as_bytes(), DOCKER_TIMEOUT).await?;
    ensure_success(output, context)
}

fn build_compose_base_command(project_name: &str, config_files: Option<&str>) -> String {
    let mut command = String::from("docker compose");

    if let Some(config_files) = config_files.filter(|value| !value.trim().is_empty()) {
        for file in config_files
            .split(',')
            .map(str::trim)
            .filter(|file| !file.is_empty())
        {
            command.push_str(" -f ");
            command.push_str(&sh_quote(file));
        }
    }

    command.push_str(" -p ");
    command.push_str(&sh_quote(project_name));
    command
}

fn split_compose_services_output(output: &str) -> (String, String) {
    let mut services = String::new();
    let mut ps_json = String::new();
    let mut in_ps_json = false;

    for line in output.lines() {
        if line == COMPOSE_PS_JSON_BEGIN {
            in_ps_json = true;
            continue;
        }
        if line == COMPOSE_PS_JSON_END {
            in_ps_json = false;
            continue;
        }
        if in_ps_json {
            ps_json.push_str(line);
            ps_json.push('\n');
        } else {
            services.push_str(line);
            services.push('\n');
        }
    }

    (services, ps_json)
}

fn normalize_container_action(action: &str) -> AppResult<&'static str> {
    match action.trim().to_ascii_lowercase().as_str() {
        "start" => Ok("start"),
        "stop" => Ok("stop"),
        "restart" => Ok("restart"),
        "kill" => Ok("kill"),
        "remove" | "rm" => Ok("rm"),
        _ => Err(AppError::Config(
            "Unsupported Docker container action".to_string(),
        )),
    }
}

fn normalize_compose_action(action: &str) -> AppResult<&'static str> {
    match action.trim().to_ascii_lowercase().as_str() {
        "up" => Ok("up"),
        "down" => Ok("down"),
        "restart" => Ok("restart"),
        _ => Err(AppError::Config(
            "Unsupported Docker compose action".to_string(),
        )),
    }
}

fn normalize_compose_service_action(action: &str) -> AppResult<&'static str> {
    match action.trim().to_ascii_lowercase().as_str() {
        "up" => Ok("up"),
        "stop" => Ok("stop"),
        "restart" => Ok("restart"),
        _ => Err(AppError::Config(
            "Unsupported Docker compose service action".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_container_actions() {
        assert_eq!(normalize_container_action("remove").unwrap(), "rm");
        assert!(normalize_container_action("exec").is_err());
    }

    #[test]
    fn validates_compose_actions() {
        assert_eq!(normalize_compose_action("up").unwrap(), "up");
        assert!(normalize_compose_action("pull").is_err());
    }

    #[test]
    fn validates_compose_service_actions() {
        assert_eq!(normalize_compose_service_action("up").unwrap(), "up");
        assert_eq!(
            normalize_compose_service_action("restart").unwrap(),
            "restart"
        );
        assert!(normalize_compose_service_action("down").is_err());
    }
}
