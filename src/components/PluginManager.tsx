"use client";

import React, { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "./Toast";
import useAppStore, { usePlugins, type PluginInfo } from "@/store";
import { Puzzle, Trash2, RefreshCw, FolderSearch } from "lucide-react";

export function PluginManager() {
  const plugins = usePlugins();
  const setPlugins = useAppStore((s: { setPlugins: (p: PluginInfo[]) => void }) => s.setPlugins);
  const removePlugin = useAppStore((s: { removePlugin: (id: string) => void }) => s.removePlugin);
  const { toast } = useToast();
  const [scanning, setScanning] = useState(false);

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

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-[11px] font-semibold text-[#777] uppercase tracking-wider">
          Plugin Manager
        </span>
        <div className="flex items-center gap-1">
          <button
            onClick={refreshPlugins}
            className="p-1 text-[#555] hover:text-blue-400 transition-colors"
            title="Refresh"
          >
            <RefreshCw size={12} />
          </button>
        </div>
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
          {plugins.map((plugin) => (
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
                  <span className="text-[10px] text-[#555]">{plugin.version}</span>
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
              <button
                onClick={() => handleUnload(plugin.id)}
                className="p-1 text-[#555] hover:text-red-400 transition-colors shrink-0"
                title="Unload"
              >
                <Trash2 size={12} />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
