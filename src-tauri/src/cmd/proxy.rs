use crate::config::{self, ProxyConfig};
use crate::error::AppResult;
use crate::utils::crypto;

fn schedule_cloud_sync_notify(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        crate::core::cloud_sync::notify_config_changed(&app).await;
    });
}

#[tauri::command]
pub fn get_proxies(app: tauri::AppHandle) -> AppResult<Vec<ProxyConfig>> {
    let mut proxies = config::load_proxies(&app)?;
    for p in &mut proxies {
        p.password = None;
    }
    Ok(proxies)
}

#[tauri::command]
pub fn get_proxy_groups(app: tauri::AppHandle) -> AppResult<Vec<config::ProxyGroup>> {
    config::load_proxy_groups(&app)
}

#[tauri::command]
pub fn save_proxy(app: tauri::AppHandle, mut proxy: ProxyConfig) -> AppResult<String> {
    let mut proxies = config::load_proxies(&app)?;

    if proxy.id.is_empty() {
        proxy.id = uuid::Uuid::new_v4().to_string();
    }
    let target_id = proxy.id.clone();
    let existing = proxies.iter().find(|p| p.id == target_id);

    proxy.password = match proxy.password.as_deref() {
        Some(plain) if !plain.is_empty() => Some(crypto::encrypt(plain)?),
        Some("") => None,
        _ => existing.and_then(|e| e.password.clone()),
    };

    if let Some(ex) = proxies.iter_mut().find(|p| p.id == target_id) {
        *ex = proxy;
    } else {
        proxies.push(proxy);
    }

    config::save_proxies(&app, &proxies)?;
    schedule_cloud_sync_notify(app.clone());
    Ok(target_id)
}

#[tauri::command]
pub fn save_proxy_group(
    app: tauri::AppHandle,
    mut group: config::ProxyGroup,
) -> AppResult<String> {
    let mut groups = config::load_proxy_groups(&app)?;

    if group.id.is_empty() {
        group.id = uuid::Uuid::new_v4().to_string();
    }
    let target_id = group.id.clone();
    if let Some(existing) = groups.iter_mut().find(|item| item.id == target_id) {
        *existing = group;
    } else {
        groups.push(group);
    }

    config::save_proxy_groups(&app, &groups)?;
    schedule_cloud_sync_notify(app.clone());
    Ok(target_id)
}

#[tauri::command]
pub fn set_proxy_group(
    app: tauri::AppHandle,
    proxy_id: String,
    group_id: Option<String>,
) -> AppResult<()> {
    let mut proxies = config::load_proxies(&app)?;
    if let Some(proxy) = proxies.iter_mut().find(|proxy| proxy.id == proxy_id) {
        proxy.group_id = group_id;
    }
    config::save_proxies(&app, &proxies)?;
    schedule_cloud_sync_notify(app.clone());
    Ok(())
}

#[tauri::command]
pub fn delete_proxy(app: tauri::AppHandle, proxy_id: String) -> AppResult<()> {
    let mut proxies = config::load_proxies(&app)?;
    proxies.retain(|p| p.id != proxy_id);
    config::save_proxies(&app, &proxies)?;
    schedule_cloud_sync_notify(app.clone());
    Ok(())
}

#[tauri::command]
pub fn delete_proxy_group(app: tauri::AppHandle, group_id: String) -> AppResult<()> {
    let mut groups = config::load_proxy_groups(&app)?;
    groups.retain(|group| group.id != group_id);
    config::save_proxy_groups(&app, &groups)?;

    let mut proxies = config::load_proxies(&app)?;
    proxies.retain(|proxy| proxy.group_id.as_deref() != Some(group_id.as_str()));
    config::save_proxies(&app, &proxies)?;

    schedule_cloud_sync_notify(app.clone());
    Ok(())
}

#[tauri::command]
pub fn get_proxy_password(app: tauri::AppHandle, proxy_id: String) -> AppResult<Option<String>> {
    let proxies = config::load_proxies(&app)?;
    let proxy = proxies.into_iter().find(|p| p.id == proxy_id);
    match proxy.and_then(|p| p.password) {
        Some(ct) => Ok(crypto::decrypt(&ct).ok()),
        None => Ok(None),
    }
}
