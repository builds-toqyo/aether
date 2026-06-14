use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use log::{debug, info, warn};
use tauri::State;
use crate::state::AppState;

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
        self.hooks.get(&hook)
            .map(|ids| ids.iter()
                .filter_map(|id| self.plugins.get(id))
                .collect())
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
    request: LoadPluginRequest,
    state: State<'_, AppState>,
) -> Result<PluginResponse, String> {
    debug!("Loading plugin from: {}", request.path);

    let path = PathBuf::from(&request.path);
    if !path.exists() {
        return Err(format!("Plugin file not found: {}", request.path));
    }

    // For MVP: simulate loading by reading a companion .json manifest
    let manifest_path = path.with_extension("json");
    let info = if manifest_path.exists() {
        let manifest_json = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read plugin manifest: {}", e))?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_json)
            .map_err(|e| format!("Failed to parse plugin manifest: {}", e))?;

        PluginInfo {
            id: manifest.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
            name: manifest.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown Plugin").to_string(),
            version: manifest.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string(),
            author: manifest.get("author").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            description: manifest.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            path: request.path.clone(),
            loaded: true,
            hooks: manifest.get("hooks")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
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
                    .collect())
                .unwrap_or_default(),
        }
    } else {
        // No manifest: infer from filename
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        PluginInfo {
            id: name.clone(),
            name: name.clone(),
            version: "0.0.0".to_string(),
            author: "".to_string(),
            description: format!("Plugin loaded from {}", request.path),
            path: request.path.clone(),
            loaded: true,
            hooks: Vec::new(),
        }
    };

    let mut registry = state.plugin_registry.lock()
        .map_err(|e| format!("Failed to lock plugin registry: {}", e))?;

    if registry.plugins.contains_key(&info.id) {
        warn!("Plugin '{}' already loaded, replacing", info.id);
        registry.unregister(&info.id);
    }

    registry.register(info.clone());
    info!("Loaded plugin '{}' v{} from {}", info.name, info.version, request.path);

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
    let mut registry = state.plugin_registry.lock()
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
pub async fn plugin_list(
    state: State<'_, AppState>,
) -> Result<Vec<PluginInfo>, String> {
    let registry = state.plugin_registry.lock()
        .map_err(|e| format!("Failed to lock plugin registry: {}", e))?;

    Ok(registry.plugins.values().cloned().collect())
}

#[tauri::command]
pub async fn plugin_get_info(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<PluginInfo, String> {
    let registry = state.plugin_registry.lock()
        .map_err(|e| format!("Failed to lock plugin registry: {}", e))?;

    registry.plugins.get(&plugin_id)
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
    let mut registry = state.plugin_registry.lock()
        .map_err(|e| format!("Failed to lock plugin registry: {}", e))?;

    let extensions = ["dylib", "so", "dll"];
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext) {
                    let manifest_path = path.with_extension("json");
                    let info = if manifest_path.exists() {
                        if let Ok(manifest_json) = fs::read_to_string(&manifest_path) {
                            if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&manifest_json) {
                                PluginInfo {
                                    id: manifest.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                                    name: manifest.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                                    version: manifest.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string(),
                                    author: manifest.get("author").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                    description: manifest.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                    path: path.to_string_lossy().to_string(),
                                    loaded: true,
                                    hooks: manifest.get("hooks")
                                        .and_then(|v| v.as_array())
                                        .map(|arr| arr.iter()
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
                                            .collect())
                                        .unwrap_or_default(),
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
    }

    info!("Scanned '{}': loaded {} new plugin(s)", directory, loaded.len());
    Ok(loaded)
}
