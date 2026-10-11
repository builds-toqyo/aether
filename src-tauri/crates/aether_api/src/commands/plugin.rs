use crate::state::AppState;
use log::{debug, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::State;

/// Metadata describing a loaded plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub path: String,
    pub loaded: bool,
    pub hooks: Vec<PluginHook>,
    pub installed_version: Option<String>,
    pub latest_version: Option<String>,
    pub update_available: bool,
}

/// Plugin metadata from remote repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemotePluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub download_url: String,
    pub checksum: String,
    pub size: u64,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub downloads: u64,
    pub rating: f32,
}

/// Plugin repository index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRepository {
    pub name: String,
    pub url: String,
    pub version: String,
    pub plugins: Vec<RemotePluginInfo>,
}

/// Search filter for plugin repository
#[derive(Debug, Clone, Deserialize)]
pub struct PluginSearchFilter {
    pub query: Option<String>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
    pub min_rating: Option<f32>,
    pub sort_by: Option<String>, // "name", "downloads", "rating", "updated"
    pub sort_order: Option<String>, // "asc", "desc"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginHook {
    OnProjectLoad,
    OnProjectSave,
    OnRenderStart,
    OnRenderComplete,
    ProcessFrame,
    ProcessAudio,
    CustomEffect,
}

/// In-memory plugin registry
#[derive(Debug, Default)]
pub struct PluginRegistry {
    pub plugins: HashMap<String, PluginInfo>,
    pub hooks: HashMap<PluginHook, Vec<String>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            hooks: HashMap::new(),
        }
    }

    pub fn register(&mut self, info: PluginInfo) {
        for hook in &info.hooks {
            self.hooks.entry(*hook).or_default().push(info.id.clone());
        }
        self.plugins.insert(info.id.clone(), info);
    }

    pub fn unregister(&mut self, plugin_id: &str) {
        if let Some(info) = self.plugins.remove(plugin_id) {
            for hook in &info.hooks {
                if let Some(list) = self.hooks.get_mut(hook) {
                    list.retain(|id| id != plugin_id);
                }
            }
        }
    }

    pub fn plugins_for_hook(&self, hook: PluginHook) -> Vec<&PluginInfo> {
        self.hooks
            .get(&hook)
            .map(|ids| ids.iter().filter_map(|id| self.plugins.get(id)).collect())
            .unwrap_or_default()
    }

    /// Invoke all plugins registered for a given hook
    pub fn invoke_hook(&self, hook: PluginHook, context: &serde_json::Value) {
        let plugins = self.plugins_for_hook(hook);
        if plugins.is_empty() {
            return;
        }

        info!("Invoking hook '{:?}' for {} plugin(s)", hook, plugins.len());
        for plugin in plugins {
            debug!(
                "Would call plugin '{}' for hook '{:?}' with context: {}",
                plugin.id, hook, context
            );
            // TODO: Actual plugin invocation via dynamic library call
            // For MVP, plugins are tracked but not dynamically executed
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PluginResponse {
    pub success: bool,
    pub message: String,
    pub plugin: Option<PluginInfo>,
}

#[derive(Debug, Deserialize)]
pub struct LoadPluginRequest {
    pub path: String,
}

#[tauri::command]
pub async fn plugin_load(
    path: String,
    state: State<'_, AppState>,
) -> Result<PluginResponse, String> {
    debug!("Loading plugin from: {}", path);

    let path = PathBuf::from(&path);
    if !path.exists() {
        return Err(format!("Plugin file not found: {}", path.display()));
    }

    // For MVP: simulate loading by reading a companion .json manifest
    let manifest_path = path.with_extension("json");
    let info = if manifest_path.exists() {
        let manifest_json = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read plugin manifest: {}", e))?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_json)
            .map_err(|e| format!("Failed to parse plugin manifest: {}", e))?;

        PluginInfo {
            id: manifest
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            name: manifest
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown Plugin")
                .to_string(),
            version: manifest
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string(),
            author: manifest
                .get("author")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            description: manifest
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            path: path.to_string_lossy().to_string(),
            loaded: true,
            hooks: manifest
                .get("hooks")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .filter_map(|s| match s {
                            "on_project_load" => Some(PluginHook::OnProjectLoad),
                            "on_project_save" => Some(PluginHook::OnProjectSave),
                            "on_render_start" => Some(PluginHook::OnRenderStart),
                            "on_render_complete" => Some(PluginHook::OnRenderComplete),
                            "process_frame" => Some(PluginHook::ProcessFrame),
                            "process_audio" => Some(PluginHook::ProcessAudio),
                            "custom_effect" => Some(PluginHook::CustomEffect),
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            installed_version: Some(
                manifest
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0.0.0")
                    .to_string(),
            ),
            latest_version: None,
            update_available: false,
        }
    } else {
        // No manifest: infer from filename
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        PluginInfo {
            id: name.clone(),
            name: name.clone(),
            version: "0.0.0".to_string(),
            author: "".to_string(),
            description: format!("Plugin loaded from {}", path.display()),
            path: path.to_string_lossy().to_string(),
            loaded: true,
            hooks: Vec::new(),
            installed_version: Some("0.0.0".to_string()),
            latest_version: None,
            update_available: false,
        }
    };

    let mut registry = state
        .plugin_registry
        .lock()
        .map_err(|e| format!("Failed to lock plugin registry: {}", e))?;

    if registry.plugins.contains_key(&info.id) {
        warn!("Plugin '{}' already loaded, replacing", info.id);
        registry.unregister(&info.id);
    }

    registry.register(info.clone());
    info!(
        "Loaded plugin '{}' v{} from {}",
        info.name,
        info.version,
        path.display()
    );

    Ok(PluginResponse {
        success: true,
        message: format!("Loaded plugin '{}' v{}", info.name, info.version),
        plugin: Some(info),
    })
}

#[tauri::command]
pub async fn plugin_unload(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<PluginResponse, String> {
    let mut registry = state
        .plugin_registry
        .lock()
        .map_err(|e| format!("Failed to lock plugin registry: {}", e))?;

    let info = registry.plugins.get(&plugin_id).cloned();
    registry.unregister(&plugin_id);

    info!("Unloaded plugin '{}'", plugin_id);

    Ok(PluginResponse {
        success: true,
        message: format!("Unloaded plugin {}", plugin_id),
        plugin: info,
    })
}

#[tauri::command]
pub async fn plugin_list(state: State<'_, AppState>) -> Result<Vec<PluginInfo>, String> {
    let registry = state
        .plugin_registry
        .lock()
        .map_err(|e| format!("Failed to lock plugin registry: {}", e))?;

    Ok(registry.plugins.values().cloned().collect())
}

#[tauri::command]
pub async fn plugin_get_info(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<PluginInfo, String> {
    let registry = state
        .plugin_registry
        .lock()
        .map_err(|e| format!("Failed to lock plugin registry: {}", e))?;

    registry
        .plugins
        .get(&plugin_id)
        .cloned()
        .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))
}

#[tauri::command]
pub async fn plugin_scan_directory(
    directory: String,
    state: State<'_, AppState>,
) -> Result<Vec<PluginInfo>, String> {
    debug!("Scanning plugin directory: {}", directory);
    let dir = PathBuf::from(&directory);
    if !dir.exists() || !dir.is_dir() {
        return Err(format!("Directory not found: {}", directory));
    }

    let mut loaded = Vec::new();
    let mut registry = state
        .plugin_registry
        .lock()
        .map_err(|e| format!("Failed to lock plugin registry: {}", e))?;

    let extensions = ["dylib", "so", "dll"];
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str())
                && extensions.contains(&ext) {
                    let manifest_path = path.with_extension("json");
                    let info = if manifest_path.exists() {
                        if let Ok(manifest_json) = fs::read_to_string(&manifest_path) {
                            if let Ok(manifest) =
                                serde_json::from_str::<serde_json::Value>(&manifest_json)
                            {
                                PluginInfo {
                                    id: manifest
                                        .get("id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown")
                                        .to_string(),
                                    name: manifest
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("Unknown")
                                        .to_string(),
                                    version: manifest
                                        .get("version")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("0.0.0")
                                        .to_string(),
                                    author: manifest
                                        .get("author")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    description: manifest
                                        .get("description")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    path: path.to_string_lossy().to_string(),
                                    loaded: true,
                                    hooks: manifest
                                        .get("hooks")
                                        .and_then(|v| v.as_array())
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|v| v.as_str())
                                                .filter_map(|s| match s {
                                                    "on_project_load" => {
                                                        Some(PluginHook::OnProjectLoad)
                                                    }
                                                    "on_project_save" => {
                                                        Some(PluginHook::OnProjectSave)
                                                    }
                                                    "on_render_start" => {
                                                        Some(PluginHook::OnRenderStart)
                                                    }
                                                    "on_render_complete" => {
                                                        Some(PluginHook::OnRenderComplete)
                                                    }
                                                    "process_frame" => {
                                                        Some(PluginHook::ProcessFrame)
                                                    }
                                                    "process_audio" => {
                                                        Some(PluginHook::ProcessAudio)
                                                    }
                                                    "custom_effect" => {
                                                        Some(PluginHook::CustomEffect)
                                                    }
                                                    _ => None,
                                                })
                                                .collect()
                                        })
                                        .unwrap_or_default(),
                                    installed_version: Some(
                                        manifest
                                            .get("version")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("0.0.0")
                                            .to_string(),
                                    ),
                                    latest_version: None,
                                    update_available: false,
                                }
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    } else {
                        continue; // skip plugins without manifests for now
                    };

                    if !registry.plugins.contains_key(&info.id) {
                        registry.register(info.clone());
                        loaded.push(info);
                    }
                }
        }
    }

    info!(
        "Scanned '{}': loaded {} new plugin(s)",
        directory,
        loaded.len()
    );
    Ok(loaded)
}

#[derive(Debug, Serialize)]
pub struct PluginRepositoryResponse {
    pub success: bool,
    pub message: String,
    pub repository: Option<PluginRepository>,
}

#[derive(Debug, Deserialize)]
pub struct FetchRepositoryRequest {
    pub url: String,
}

#[tauri::command]
pub async fn plugin_fetch_repository(url: String) -> Result<PluginRepositoryResponse, String> {
    debug!("Fetching plugin repository from: {}", url);

    let client = Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch repository: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Repository request failed with status: {}",
            response.status()
        ));
    }

    let repository: PluginRepository = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse repository data: {}", e))?;

    info!(
        "Fetched repository '{}' with {} plugins",
        repository.name,
        repository.plugins.len()
    );

    Ok(PluginRepositoryResponse {
        success: true,
        message: format!(
            "Fetched repository '{}' with {} plugins",
            repository.name,
            repository.plugins.len()
        ),
        repository: Some(repository),
    })
}

#[derive(Debug, Serialize)]
pub struct PluginSearchResponse {
    pub success: bool,
    pub message: String,
    pub plugins: Vec<RemotePluginInfo>,
}

#[tauri::command]
pub async fn plugin_search_repository(
    repository_url: String,
    filter: Option<PluginSearchFilter>,
) -> Result<PluginSearchResponse, String> {
    debug!("Searching repository: {}", repository_url);

    let client = Client::new();
    let response = client
        .get(&repository_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch repository: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Repository request failed with status: {}",
            response.status()
        ));
    }

    let repository: PluginRepository = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse repository data: {}", e))?;

    let mut plugins = repository.plugins;

    // Apply filters
    if let Some(filter) = filter {
        // Filter by query
        if let Some(query) = filter.query {
            let query_lower = query.to_lowercase();
            plugins.retain(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
                    || p.id.to_lowercase().contains(&query_lower)
            });
        }

        // Filter by tags
        if let Some(tags) = filter.tags
            && !tags.is_empty() {
                plugins.retain(|p| {
                    tags.iter()
                        .any(|tag| p.tags.iter().any(|pt| pt.eq_ignore_ascii_case(tag)))
                });
            }

        // Filter by author
        if let Some(author) = filter.author {
            plugins.retain(|p| p.author.eq_ignore_ascii_case(&author));
        }

        // Filter by minimum rating
        if let Some(min_rating) = filter.min_rating {
            plugins.retain(|p| p.rating >= min_rating);
        }

        // Sort results
        let sort_by = filter.sort_by.as_deref().unwrap_or("name");
        let sort_order = filter.sort_order.as_deref().unwrap_or("asc");
        let reverse = sort_order == "desc";

        match sort_by {
            "name" => plugins.sort_by(|a, b| a.name.cmp(&b.name)),
            "downloads" => plugins.sort_by_key(|a| a.downloads),
            "rating" => plugins.sort_by(|a, b| {
                a.rating
                    .partial_cmp(&b.rating)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            "updated" => plugins.sort_by(|a, b| a.updated_at.cmp(&b.updated_at)),
            _ => plugins.sort_by(|a, b| a.name.cmp(&b.name)),
        }

        if reverse {
            plugins.reverse();
        }
    }

    info!("Search returned {} plugins", plugins.len());

    Ok(PluginSearchResponse {
        success: true,
        message: format!("Found {} plugins", plugins.len()),
        plugins,
    })
}

#[derive(Debug, Serialize)]
pub struct PluginInstallResponse {
    pub success: bool,
    pub message: String,
    pub plugin: Option<PluginInfo>,
}

#[derive(Debug, Deserialize)]
pub struct InstallPluginRequest {
    pub plugin_id: String,
    pub repository_url: String,
    pub install_path: String,
}

#[tauri::command]
pub async fn plugin_install_from_repository(
    plugin_id: String,
    repository_url: String,
    install_path: String,
    state: State<'_, AppState>,
) -> Result<PluginInstallResponse, String> {
    debug!("Installing plugin '{}' from repository", plugin_id);

    // Fetch repository
    let client = Client::new();
    let response = client
        .get(&repository_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch repository: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Repository request failed with status: {}",
            response.status()
        ));
    }

    let repository: PluginRepository = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse repository data: {}", e))?;

    // Find plugin in repository
    let remote_plugin = repository
        .plugins
        .iter()
        .find(|p| p.id == plugin_id)
        .ok_or_else(|| format!("Plugin '{}' not found in repository", plugin_id))?;

    // Download plugin
    let download_response = client
        .get(&remote_plugin.download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download plugin: {}", e))?;

    if !download_response.status().is_success() {
        return Err(format!(
            "Plugin download failed with status: {}",
            download_response.status()
        ));
    }

    let plugin_bytes = download_response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read plugin bytes: {}", e))?;

    // Verify checksum
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&plugin_bytes);
    let checksum = format!("{:x}", hasher.finalize());

    if checksum != remote_plugin.checksum {
        return Err(format!(
            "Checksum mismatch: expected {}, got {}",
            remote_plugin.checksum, checksum
        ));
    }

    // Create install directory if needed
    let install_path_buf = PathBuf::from(&install_path);
    fs::create_dir_all(&install_path_buf)
        .map_err(|e| format!("Failed to create install directory: {}", e))?;

    // Write plugin file
    let plugin_filename = format!("{}.{}", remote_plugin.id, std::env::consts::DLL_EXTENSION);
    let plugin_path = install_path_buf.join(&plugin_filename);
    fs::write(&plugin_path, plugin_bytes)
        .map_err(|e| format!("Failed to write plugin file: {}", e))?;

    // Create manifest
    let manifest = serde_json::json!({
        "id": remote_plugin.id,
        "name": remote_plugin.name,
        "version": remote_plugin.version,
        "author": remote_plugin.author,
        "description": remote_plugin.description,
        "license": remote_plugin.license,
        "homepage": remote_plugin.homepage,
        "repository": remote_plugin.repository,
        "dependencies": remote_plugin.dependencies,
        "tags": remote_plugin.tags,
    });

    let manifest_path = install_path_buf.join(format!("{}.json", remote_plugin.id));
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .map_err(|e| format!("Failed to write manifest: {}", e))?;

    // Load the plugin
    let load_response = plugin_load(plugin_path.to_string_lossy().to_string(), state).await?;

    info!(
        "Successfully installed plugin '{}' v{}",
        remote_plugin.name, remote_plugin.version
    );

    Ok(PluginInstallResponse {
        success: true,
        message: format!(
            "Installed plugin '{}' v{}",
            remote_plugin.name, remote_plugin.version
        ),
        plugin: load_response.plugin,
    })
}

#[derive(Debug, Serialize)]
pub struct PluginUpdateResponse {
    pub success: bool,
    pub message: String,
    pub updated_plugins: Vec<String>,
}

#[tauri::command]
pub async fn plugin_check_updates(
    repository_url: String,
    state: State<'_, AppState>,
) -> Result<PluginUpdateResponse, String> {
    debug!("Checking for plugin updates from: {}", repository_url);

    let local_versions: Vec<(String, String)> = {
        let registry = state
            .plugin_registry
            .lock()
            .map_err(|e| format!("Failed to lock plugin registry: {}", e))?;
        registry
            .plugins
            .iter()
            .map(|(id, p)| (id.clone(), p.version.clone()))
            .collect()
    };

    // Fetch repository
    let client = Client::new();
    let response = client
        .get(&repository_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch repository: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Repository request failed with status: {}",
            response.status()
        ));
    }

    let repository: PluginRepository = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse repository data: {}", e))?;

    let mut updated_plugins = Vec::new();

    for (plugin_id, local_version) in &local_versions {
        if let Some(remote_plugin) = repository.plugins.iter().find(|p| p.id == *plugin_id) {
            // Compare versions (simple string comparison for now)
            if *local_version != remote_plugin.version {
                updated_plugins.push(plugin_id.clone());
                info!(
                    "Update available for '{}': {} -> {}",
                    plugin_id, local_version, remote_plugin.version
                );
            }
        }
    }

    info!(
        "Found {} plugins with updates available",
        updated_plugins.len()
    );

    Ok(PluginUpdateResponse {
        success: true,
        message: format!(
            "Found {} plugins with updates available",
            updated_plugins.len()
        ),
        updated_plugins,
    })
}

#[tauri::command]
pub async fn plugin_update_plugin(
    plugin_id: String,
    repository_url: String,
    install_path: String,
    state: State<'_, AppState>,
) -> Result<PluginInstallResponse, String> {
    debug!("Updating plugin '{}'", plugin_id);

    // Unload current version
    let _ = plugin_unload(plugin_id.clone(), state.clone()).await;

    // Install new version
    plugin_install_from_repository(plugin_id, repository_url, install_path, state).await
}
