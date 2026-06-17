use crate::config;
use crate::core::ssh::TunnelManager;
use crate::error::{AppError, AppResult};
use std::sync::Arc;
use tauri::Manager;

fn schedule_cloud_sync_notify(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        crate::core::cloud_sync::notify_config_changed(&app).await;
    });
}

#[tauri::command]
pub async fn get_tunnels(app: tauri::AppHandle) -> AppResult<Vec<config::TunnelConfig>> {
    let mut tunnels = config::load_tunnels(&app)?;
    let tunnel_mgr = app.state::<Arc<TunnelManager>>();
    for tunnel in &mut tunnels {
        tunnel.is_open = tunnel_mgr.is_open(&tunnel.id).await;
    }
    Ok(tunnels)
}

#[tauri::command]
pub fn get_tunnel_groups(app: tauri::AppHandle) -> AppResult<Vec<config::TunnelGroup>> {
    config::load_tunnel_groups(&app)
}

#[tauri::command]
pub async fn save_tunnel(
    app: tauri::AppHandle,
    tunnel_mgr: tauri::State<'_, Arc<TunnelManager>>,
    tunnel: config::TunnelConfig,
) -> AppResult<()> {
    if tunnel_mgr.is_open(&tunnel.id).await {
        tunnel_mgr.close(&tunnel.id).await;
    }
    let mut tunnels = config::load_tunnels(&app)?;
    if let Some(existing) = tunnels.iter_mut().find(|t| t.id == tunnel.id) {
        *existing = tunnel;
    } else {
        tunnels.push(tunnel);
    }
    config::save_tunnels(&app, &tunnels)?;
    schedule_cloud_sync_notify(app.clone());
    Ok(())
}

#[tauri::command]
pub fn save_tunnel_group(
    app: tauri::AppHandle,
    mut group: config::TunnelGroup,
) -> AppResult<String> {
    let mut groups = config::load_tunnel_groups(&app)?;

    if group.id.is_empty() {
        group.id = uuid::Uuid::new_v4().to_string();
    }
    let target_id = group.id.clone();
    if let Some(existing) = groups.iter_mut().find(|item| item.id == target_id) {
        *existing = group;
    } else {
        groups.push(group);
    }

    config::save_tunnel_groups(&app, &groups)?;
    schedule_cloud_sync_notify(app.clone());
    Ok(target_id)
}

#[tauri::command]
pub fn set_tunnel_group(
    app: tauri::AppHandle,
    tunnel_id: String,
    group_id: Option<String>,
) -> AppResult<()> {
    let mut tunnels = config::load_tunnels(&app)?;
    if let Some(tunnel) = tunnels.iter_mut().find(|tunnel| tunnel.id == tunnel_id) {
        tunnel.group_id = group_id;
    }
    config::save_tunnels(&app, &tunnels)?;
    schedule_cloud_sync_notify(app.clone());
    Ok(())
}

#[tauri::command]
pub async fn delete_tunnel(
    app: tauri::AppHandle,
    tunnel_mgr: tauri::State<'_, Arc<TunnelManager>>,
    tunnel_id: String,
) -> AppResult<()> {
    if tunnel_mgr.is_open(&tunnel_id).await {
        tunnel_mgr.close(&tunnel_id).await;
    }
    let mut tunnels = config::load_tunnels(&app)?;
    tunnels.retain(|t| t.id != tunnel_id);
    config::save_tunnels(&app, &tunnels)?;
    schedule_cloud_sync_notify(app.clone());
    Ok(())
}

#[tauri::command]
pub async fn delete_tunnel_group(
    app: tauri::AppHandle,
    tunnel_mgr: tauri::State<'_, Arc<TunnelManager>>,
    group_id: String,
) -> AppResult<()> {
    let mut groups = config::load_tunnel_groups(&app)?;
    groups.retain(|group| group.id != group_id);
    config::save_tunnel_groups(&app, &groups)?;

    let mut tunnels = config::load_tunnels(&app)?;
    for tunnel in &tunnels {
        if tunnel.group_id.as_deref() == Some(group_id.as_str()) {
            tunnel_mgr.close(&tunnel.id).await;
        }
    }
    tunnels.retain(|tunnel| tunnel.group_id.as_deref() != Some(group_id.as_str()));
    config::save_tunnels(&app, &tunnels)?;

    schedule_cloud_sync_notify(app.clone());
    Ok(())
}

#[tauri::command]
pub async fn open_tunnel(
    app: tauri::AppHandle,
    tunnel_mgr: tauri::State<'_, Arc<TunnelManager>>,
    tunnel_id: String,
) -> AppResult<()> {
    let tunnels = config::load_tunnels(&app)?;
    let tunnel = tunnels
        .iter()
        .find(|t| t.id == tunnel_id)
        .ok_or_else(|| AppError::Config(format!("Tunnel '{}' not found", tunnel_id)))?;
    tunnel_mgr.open(tunnel, &app).await
}

#[tauri::command]
pub async fn close_tunnel(
    tunnel_mgr: tauri::State<'_, Arc<TunnelManager>>,
    tunnel_id: String,
) -> AppResult<()> {
    tunnel_mgr.close(&tunnel_id).await;
    Ok(())
}
