"use client";

import React, { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "./Toast";
import useAppStore, { usePlugins, type PluginInfo } from "@/store";
import { Puzzle, Trash2, RefreshCw, FolderSearch, Download, Cloud, Search, Star, Tag, ExternalLink, CheckCircle2, AlertCircle } from "lucide-react";

interface RemotePluginInfo {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  download_url: string;
  checksum: string;
  size: number;
  dependencies: string[];
  tags: string[];
  license: string;
  homepage?: string;
  repository?: string;
  created_at: string;
  updated_at: string;
  downloads: number;
  rating: number;
}

export function PluginManager() {
  const plugins = usePlugins();
  const setPlugins = useAppStore((s: { setPlugins: (p: PluginInfo[]) => void }) => s.setPlugins);
  const removePlugin = useAppStore((s: { removePlugin: (id: string) => void }) => s.removePlugin);
  const { toast } = useToast();
  const [scanning, setScanning] = useState(false);
  const [showMarketplace, setShowMarketplace] = useState(false);
  const [repositoryUrl, setRepositoryUrl] = useState("https://plugins.aether.video/index.json");
  const [remotePlugins, setRemotePlugins] = useState<RemotePluginInfo[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [sortBy, setSortBy] = useState("name");
  const [sortOrder, setSortOrder] = useState("asc");
  const [loading, setLoading] = useState(false);
  const [installing, setInstalling] = useState<string | null>(null);
  const [checkingUpdates, setCheckingUpdates] = useState(false);
  const [pluginsWithUpdates, setPluginsWithUpdates] = useState<string[]>([]);

  const refreshPlugins = useCallback(async () => {
    try {
      const list: any[] = await invoke("plugin_list");
      setPlugins(
        list.map((p) => ({
          id: p.id,
          name: p.name,
          version: p.version,
          author: p.author,
          description: p.description,
          path: p.path,
          loaded: p.loaded,
          hooks: p.hooks || [],
          installed_version: p.installed_version,
          latest_version: p.latest_version,
          update_available: p.update_available,
        }))
      );
    } catch (e) {
      toast(`Failed to refresh plugins: ${e}`, "error");
    }
  }, [setPlugins, toast]);

  useEffect(() => {
    refreshPlugins();
  }, [refreshPlugins]);

  const [scanPath, setScanPath] = useState("");

  const handleScanDirectory = useCallback(async () => {
    setScanning(true);
    try {
      const directory = scanPath.trim() || "/Users/nutcase/Documents/aether/plugins";
      const loaded: any[] = await invoke("plugin_scan_directory", { directory });
      toast(`Scanned and loaded ${loaded.length} plugin(s)`, "success");
      await refreshPlugins();
    } catch (e) {
      toast(`Scan failed: ${e}`, "error");
    } finally {
      setScanning(false);
    }
  }, [refreshPlugins, toast, scanPath]);

  const [loadPath, setLoadPath] = useState("");

  const handleLoadPlugin = useCallback(async () => {
    try {
      const path = loadPath.trim();
      if (!path) {
        toast("Please enter a plugin path", "warning");
        return;
      }
      const result = await invoke("plugin_load", { path });
      toast(`Loaded plugin: ${(result as any)?.plugin?.name || path}`, "success");
      setLoadPath("");
      await refreshPlugins();
    } catch (e) {
      toast(`Load failed: ${e}`, "error");
    }
  }, [refreshPlugins, toast, loadPath]);

  const handleUnload = useCallback(
    async (pluginId: string) => {
      try {
        await invoke("plugin_unload", { plugin_id: pluginId });
        removePlugin(pluginId);
        toast("Plugin unloaded", "success");
      } catch (e) {
        toast(`Unload failed: ${e}`, "error");
      }
    },
    [removePlugin, toast]
  );

  const fetchMarketplace = useCallback(async () => {
    setLoading(true);
    try {
      const result: any = await invoke("plugin_fetch_repository", { url: repositoryUrl });
      if (result.success && result.repository) {
        setRemotePlugins(result.repository.plugins);
        toast(`Loaded ${result.repository.plugins.length} plugins from marketplace`, "success");
      }
    } catch (e) {
      toast(`Failed to fetch marketplace: ${e}`, "error");
    } finally {
      setLoading(false);
    }
  }, [repositoryUrl, toast]);

  const searchMarketplace = useCallback(async () => {
    setLoading(true);
    try {
      const filter: any = {
        query: searchQuery || undefined,
        tags: selectedTags.length > 0 ? selectedTags : undefined,
        sort_by: sortBy,
        sort_order: sortOrder,
      };
      const result: any = await invoke("plugin_search_repository", { repository_url: repositoryUrl, filter });
      if (result.success) {
        setRemotePlugins(result.plugins);
        toast(`Found ${result.plugins.length} plugins`, "success");
      }
    } catch (e) {
      toast(`Search failed: ${e}`, "error");
    } finally {
      setLoading(false);
    }
  }, [repositoryUrl, searchQuery, selectedTags, sortBy, sortOrder, toast]);

  const installPlugin = useCallback(async (pluginId: string) => {
    setInstalling(pluginId);
    try {
      const installPath = "/Users/nutcase/Documents/aether/plugins";
      const result: any = await invoke("plugin_install_from_repository", {
        plugin_id: pluginId,
        repository_url: repositoryUrl,
        install_path: installPath,
      });
      if (result.success) {
        toast(`Installed plugin: ${result.plugin?.name}`, "success");
        await refreshPlugins();
      }
    } catch (e) {
      toast(`Installation failed: ${e}`, "error");
    } finally {
      setInstalling(null);
    }
  }, [repositoryUrl, refreshPlugins, toast]);

  const checkUpdates = useCallback(async () => {
    setCheckingUpdates(true);
    try {
      const result: any = await invoke("plugin_check_updates", { repository_url: repositoryUrl });
      if (result.success) {
        setPluginsWithUpdates(result.updated_plugins);
        toast(`Found ${result.updated_plugins.length} plugins with updates`, "success");
      }
    } catch (e) {
      toast(`Failed to check updates: ${e}`, "error");
    } finally {
      setCheckingUpdates(false);
    }
  }, [repositoryUrl, toast]);

  const updatePlugin = useCallback(async (pluginId: string) => {
    setInstalling(pluginId);
    try {
      const installPath = "/Users/nutcase/Documents/aether/plugins";
      const result: any = await invoke("plugin_update_plugin", {
        plugin_id: pluginId,
        repository_url: repositoryUrl,
        install_path: installPath,
      });
      if (result.success) {
        toast(`Updated plugin: ${result.plugin?.name}`, "success");
        await refreshPlugins();
        await checkUpdates();
      }
    } catch (e) {
      toast(`Update failed: ${e}`, "error");
    } finally {
      setInstalling(null);
    }
  }, [repositoryUrl, refreshPlugins, checkUpdates, toast]);

  const toggleTag = useCallback((tag: string) => {
    setSelectedTags(prev => 
      prev.includes(tag) 
        ? prev.filter(t => t !== tag)
        : [...prev, tag]
    );
  }, []);

  const allTags = Array.from(new Set(remotePlugins.flatMap(p => p.tags))).sort();

  const filteredPlugins = remotePlugins.filter(plugin => {
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      return plugin.name.toLowerCase().includes(query) ||
             plugin.description.toLowerCase().includes(query) ||
             plugin.author.toLowerCase().includes(query);
    }
    if (selectedTags.length > 0) {
      return selectedTags.some(tag => plugin.tags.includes(tag));
    }
    return true;
  });

  const sortedPlugins = [...filteredPlugins].sort((a, b) => {
    let comparison = 0;
    switch (sortBy) {
      case "name":
        comparison = a.name.localeCompare(b.name);
        break;
      case "downloads":
        comparison = a.downloads - b.downloads;
        break;
      case "rating":
        comparison = a.rating - b.rating;
        break;
      case "updated":
        comparison = new Date(a.updated_at).getTime() - new Date(b.updated_at).getTime();
        break;
    }
    return sortOrder === "asc" ? comparison : -comparison;
  });

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-[11px] font-semibold text-[#777] uppercase tracking-wider">
          Plugin Manager
        </span>
        <div className="flex items-center gap-1">
          <button
            onClick={() => setShowMarketplace(!showMarketplace)}
            className={`p-1 transition-colors ${showMarketplace ? "text-blue-400" : "text-[#555] hover:text-blue-400"}`}
            title="Marketplace"
          >
            <Cloud size={12} />
          </button>
          <button
            onClick={refreshPlugins}
            className="p-1 text-[#555] hover:text-blue-400 transition-colors"
            title="Refresh"
          >
            <RefreshCw size={12} />
          </button>
        </div>
      </div>

      {showMarketplace ? (
        <div className="space-y-3">
          <div className="flex items-center gap-1.5">
            <input
              type="text"
              value={repositoryUrl}
              onChange={(e) => setRepositoryUrl(e.target.value)}
              placeholder="Repository URL..."
              className="flex-1 bg-[#1a1a1a] border border-[#222] rounded px-2 py-1 text-[10px] text-[#ccc] outline-none focus:border-blue-500"
            />
            <button
              onClick={fetchMarketplace}
              disabled={loading}
              className="flex items-center gap-1 px-2 py-1 bg-blue-600/15 text-blue-400 rounded hover:bg-blue-600/25 transition-colors disabled:opacity-50 shrink-0"
            >
              <Cloud size={11} />
              <span className="text-[10px]">Fetch</span>
            </button>
          </div>

          {remotePlugins.length > 0 && (
            <>
              <div className="flex items-center gap-1.5">
                <div className="flex-1 relative">
                  <Search size={11} className="absolute left-2 top-1/2 -translate-y-1/2 text-[#555]" />
                  <input
                    type="text"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    placeholder="Search plugins..."
                    className="w-full bg-[#1a1a1a] border border-[#222] rounded pl-7 pr-2 py-1 text-[10px] text-[#ccc] outline-none focus:border-blue-500"
                  />
                </div>
                <select
                  value={sortBy}
                  onChange={(e) => setSortBy(e.target.value)}
                  className="bg-[#1a1a1a] border border-[#222] rounded px-2 py-1 text-[10px] text-[#ccc] outline-none focus:border-blue-500"
                >
                  <option value="name">Name</option>
                  <option value="downloads">Downloads</option>
                  <option value="rating">Rating</option>
                  <option value="updated">Updated</option>
                </select>
                <button
                  onClick={() => setSortOrder(sortOrder === "asc" ? "desc" : "asc")}
                  className="px-2 py-1 bg-[#1a1a1a] border border-[#222] rounded text-[10px] text-[#ccc] hover:border-[#333] transition-colors"
                >
                  {sortOrder === "asc" ? "↑" : "↓"}
                </button>
              </div>

              {allTags.length > 0 && (
                <div className="flex flex-wrap gap-1">
                  {allTags.map((tag) => (
                    <button
                      key={tag}
                      onClick={() => toggleTag(tag)}
                      className={`text-[9px] px-2 py-0.5 rounded transition-colors ${
                        selectedTags.includes(tag)
                          ? "bg-blue-600/25 text-blue-400"
                          : "bg-[#252525] text-[#888] hover:bg-[#333]"
                      }`}
                    >
                      <Tag size={8} className="inline mr-1" />
                      {tag}
                    </button>
                  ))}
                </div>
              )}

              <div className="space-y-1.5 max-h-80 overflow-y-auto">
                {sortedPlugins.map((plugin) => {
                  const isInstalled = plugins.some(p => p.id === plugin.id);
                  const isInstalling = installing === plugin.id;
                  
                  return (
                    <div
                      key={plugin.id}
                      className="flex items-start gap-2 px-2 py-2 rounded bg-[#1a1a1a] border border-[#222] hover:border-[#333] transition-colors"
                    >
                      <Puzzle size={14} className="text-blue-400 mt-0.5 shrink-0" />
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center justify-between">
                          <span className="text-[11px] text-[#ccc] font-medium truncate">
                            {plugin.name}
                          </span>
                          <div className="flex items-center gap-1">
                            <div className="flex items-center gap-0.5 text-[#666]">
                              <Star size={8} className="text-yellow-500" />
                              <span className="text-[9px]">{plugin.rating.toFixed(1)}</span>
                            </div>
                            <span className="text-[10px] text-[#555]">{plugin.version}</span>
                          </div>
                        </div>
                        <p className="text-[10px] text-[#666] truncate mt-0.5">
                          {plugin.description}
                        </p>
                        <div className="flex items-center gap-2 mt-1">
                          <span className="text-[9px] text-[#555]">by {plugin.author}</span>
                          <span className="text-[9px] text-[#555]">{(plugin.downloads / 1000).toFixed(1)}k downloads</span>
                          <span className="text-[9px] text-[#555]">{plugin.license}</span>
                        </div>
                        {plugin.tags.length > 0 && (
                          <div className="flex flex-wrap gap-1 mt-1">
                            {plugin.tags.slice(0, 3).map((tag) => (
                              <span
                                key={tag}
                                className="text-[9px] px-1 py-0.5 bg-[#252525] text-[#888] rounded"
                              >
                                {tag}
                              </span>
                            ))}
                            {plugin.tags.length > 3 && (
                              <span className="text-[9px] text-[#555]">+{plugin.tags.length - 3}</span>
                            )}
                          </div>
                        )}
                      </div>
                      <div className="flex items-center gap-1 shrink-0">
                        {plugin.homepage && (
                          <a
                            href={plugin.homepage}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="p-1 text-[#555] hover:text-blue-400 transition-colors"
                            title="Homepage"
                          >
                            <ExternalLink size={11} />
                          </a>
                        )}
                        {isInstalled ? (
                          <span title="Installed">
                            <CheckCircle2 size={14} className="text-green-400" />
                          </span>
                        ) : (
                          <button
                            onClick={() => installPlugin(plugin.id)}
                            disabled={isInstalling}
                            className="p-1 text-[#555] hover:text-blue-400 transition-colors disabled:opacity-50"
                            title="Install"
                          >
                            <Download size={11} />
                          </button>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </>
          )}
        </div>
      ) : (
        <>
          <div className="flex items-center gap-1.5">
            <button
              onClick={checkUpdates}
              disabled={checkingUpdates}
              className="flex items-center gap-1 px-2 py-1 bg-green-600/15 text-green-400 rounded hover:bg-green-600/25 transition-colors disabled:opacity-50 shrink-0"
            >
              <RefreshCw size={11} className={checkingUpdates ? "animate-spin" : ""} />
              <span className="text-[10px]">Check Updates</span>
            </button>
            {pluginsWithUpdates.length > 0 && (
              <span className="text-[10px] text-orange-400">
                {pluginsWithUpdates.length} update(s) available
              </span>
            )}
          </div>

          <div className="space-y-2">
            <div className="flex items-center gap-1.5">
              <input
                type="text"
                value={scanPath}
                onChange={(e) => setScanPath(e.target.value)}
                placeholder="Directory to scan..."
                className="flex-1 bg-[#1a1a1a] border border-[#222] rounded px-2 py-1 text-[10px] text-[#ccc] outline-none focus:border-blue-500"
              />
              <button
                onClick={handleScanDirectory}
                disabled={scanning}
                className="flex items-center gap-1 px-2 py-1 bg-blue-600/15 text-blue-400 rounded hover:bg-blue-600/25 transition-colors disabled:opacity-50 shrink-0"
              >
                <FolderSearch size={11} />
                <span className="text-[10px]">Scan</span>
              </button>
            </div>
            <div className="flex items-center gap-1.5">
              <input
                type="text"
                value={loadPath}
                onChange={(e) => setLoadPath(e.target.value)}
                placeholder="Plugin file path (.dylib/.so/.dll)..."
                className="flex-1 bg-[#1a1a1a] border border-[#222] rounded px-2 py-1 text-[10px] text-[#ccc] outline-none focus:border-blue-500"
              />
              <button
                onClick={handleLoadPlugin}
                className="flex items-center gap-1 px-2 py-1 bg-blue-600/15 text-blue-400 rounded hover:bg-blue-600/25 transition-colors shrink-0"
              >
                <Puzzle size={11} />
                <span className="text-[10px]">Load</span>
              </button>
            </div>
          </div>

          {plugins.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-10 gap-3">
              <div className="w-10 h-10 rounded-full bg-[#1a1a1a] flex items-center justify-center border border-[#222]">
                <Puzzle size={18} className="text-[#333]" />
              </div>
              <div className="text-center">
                <p className="text-[12px] text-[#555]">No plugins loaded</p>
                <p className="text-[10px] text-[#333] mt-1">Scan a directory or load a plugin</p>
              </div>
            </div>
          ) : (
            <div className="space-y-1.5">
              {plugins.map((plugin) => {
                const hasUpdate = pluginsWithUpdates.includes(plugin.id);
                const isUpdating = installing === plugin.id;
                
                return (
                  <div
                    key={plugin.id}
                    className="flex items-start gap-2 px-2 py-2 rounded bg-[#1a1a1a] border border-[#222] hover:border-[#333] transition-colors"
                  >
                    <Puzzle size={14} className="text-blue-400 mt-0.5 shrink-0" />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center justify-between">
                        <span className="text-[11px] text-[#ccc] font-medium truncate">
                          {plugin.name}
                        </span>
                        <div className="flex items-center gap-1">
                          {hasUpdate && (
                            <span title="Update available">
                              <AlertCircle size={10} className="text-orange-400" />
                            </span>
                          )}
                          <span className="text-[10px] text-[#555]">{plugin.version}</span>
                        </div>
                      </div>
                      <p className="text-[10px] text-[#666] truncate mt-0.5">
                        {plugin.description || plugin.path}
                      </p>
                      {plugin.hooks.length > 0 && (
                        <div className="flex flex-wrap gap-1 mt-1.5">
                          {plugin.hooks.map((hook) => (
                            <span
                              key={hook}
                              className="text-[9px] px-1 py-0.5 bg-[#252525] text-[#888] rounded"
                            >
                              {hook}
                            </span>
                          ))}
                        </div>
                      )}
                    </div>
                    <div className="flex items-center gap-1 shrink-0">
                      {hasUpdate && (
                        <button
                          onClick={() => updatePlugin(plugin.id)}
                          disabled={isUpdating}
                          className="p-1 text-orange-400 hover:text-orange-300 transition-colors disabled:opacity-50"
                          title="Update"
                        >
                          <RefreshCw size={11} className={isUpdating ? "animate-spin" : ""} />
                        </button>
                      )}
                      <button
                        onClick={() => handleUnload(plugin.id)}
                        className="p-1 text-[#555] hover:text-red-400 transition-colors"
                        title="Unload"
                      >
                        <Trash2 size={12} />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </>
      )}
    </div>
  );
}
