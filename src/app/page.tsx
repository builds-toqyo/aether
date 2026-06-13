"use client";

import React, { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { TimelineEditor } from "@/components/timeline";
import { PreviewWindow } from "@/components/Preview";
import { NodeGraphEditor } from "@/components/NodeGraph";
import { ColorScopes } from "@/components/ColorScopes";
import { WelcomeScreen } from "@/components/WelcomeScreen";
import { StatusBar } from "@/components/StatusBar";
import { InspectorPanel } from "@/components/InspectorPanel";
import { useToast } from "@/components/Toast";
import { PluginManager } from "@/components/PluginManager";
import { MulticamPanel } from "@/components/MulticamPanel";
import useAppStore from "@/store";
import {
  Film,
  Play,
  GitBranch,
  Sparkles,
  Layout,
  Layers,
  Settings,
  FolderOpen,
  Type,
  X,
} from "lucide-react";

type Workspace = "edit" | "preview" | "nodes" | "color";
type LeftTab = "media" | "effects" | "nodes" | "transitions" | "multicam" | "plugins";

export default function Home() {
  const currentProject = useAppStore((s) => s.currentProject);
  const setCurrentProject = useAppStore((s) => s.setCurrentProject);
  const updateProject = useAppStore((s) => s.updateProject);
  const undo = useAppStore((s) => s.undo);
  const redo = useAppStore((s) => s.redo);
  const copyClips = useAppStore((s) => s.copyClips);
  const cutClips = useAppStore((s) => s.cutClips);
  const pasteClips = useAppStore((s) => s.pasteClips);
  const selectedTimelineClipIds = useAppStore((s) => s.selectedTimelineClipIds);
  const timelineTracks = useAppStore((s) => s.timelineTracks);
  const { toast } = useToast();
  const [ws, setWs] = useState<Workspace>("edit");
  const [leftTab, setLeftTab] = useState<LeftTab>("media");
  const [showRight, setShowRight] = useState(true);
  const [showLeft, setShowLeft] = useState(true);
  const [editingName, setEditingName] = useState(false);
  const [nameValue, setNameValue] = useState(currentProject?.name ?? "");
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const [showAbout, setShowAbout] = useState(false);
  const nameInputRef = useRef<HTMLInputElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (editingName && nameInputRef.current) {
      nameInputRef.current.focus();
      nameInputRef.current.select();
    }
  }, [editingName]);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setActiveMenu(null);
      }
    }
    if (activeMenu) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [activeMenu]);

  const handleNameSave = () => {
    const trimmed = nameValue.trim();
    if (trimmed && currentProject && trimmed !== currentProject.name) {
      updateProject({ name: trimmed });
    } else if (currentProject) {
      setNameValue(currentProject.name);
    }
    setEditingName(false);
  };

  const handleNameKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleNameSave();
    if (e.key === "Escape") {
      setNameValue(currentProject?.name ?? "");
      setEditingName(false);
    }
  };

  if (!currentProject) {
    return <WelcomeScreen />;
  }

  return (
    <div className="h-screen flex flex-col bg-[#0d0d0d] text-[#e0e0e0] overflow-hidden select-none">
      <header className="h-9 bg-[#1a1a1a] border-b border-[#222] flex items-center px-3 shrink-0">
        <div className="flex items-center gap-2.5">
          <div className="w-5 h-5 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-sm flex items-center justify-center">
            <Film size={11} className="text-white" />
          </div>
          <span className="font-semibold text-[13px] text-white">Aether</span>
          <span className="text-[#333] text-[11px]">|</span>
          {editingName ? (
            <input
              ref={nameInputRef}
              value={nameValue}
              onChange={(e) => setNameValue(e.target.value)}
              onBlur={handleNameSave}
              onKeyDown={handleNameKeyDown}
              className="bg-[#252525] border border-[#333] rounded px-1.5 py-0.5 text-[12px] text-white w-[160px] outline-none focus:border-blue-500"
            />
          ) : (
            <button
              onClick={() => {
                setNameValue(currentProject.name);
                setEditingName(true);
              }}
              className="text-[#888] text-[12px] hover:text-white transition-colors cursor-text"
              title="Click to rename"
            >
              {currentProject.name}
            </button>
          )}
        </div>
        <div ref={menuRef} className="ml-auto flex items-center gap-1">
          {[
            { label: "File", items: [
              { label: "New Project", shortcut: "Ctrl+N", action: () => setCurrentProject(null) },
              { label: "Open Project", shortcut: "Ctrl+O", action: () =>
                toast("Open Project: file picker integration coming soon", "warning") },
              { label: "Save", shortcut: "Ctrl+S", action: async () => {
                if (!currentProject) return;
                try {
                  await invoke("project_save", { request: { project_id: currentProject.id } });
                  toast("Project saved successfully", "success");
                } catch (e) {
                  toast(`Save failed: ${e}`, "error");
                }
              }},
              { label: "Save As...", shortcut: "", action: () =>
                toast("Save As: dialog coming soon", "warning") },
              { divider: true },
              { label: "Close Project", shortcut: "", action: () => setCurrentProject(null) },
            ]},
            { label: "Edit", items: [
              { label: "Undo", shortcut: "Ctrl+Z", action: () => {
                undo();
                toast("Undo", "success");
              }},
              { label: "Redo", shortcut: "Ctrl+Shift+Z", action: () => {
                redo();
                toast("Redo", "success");
              }},
              { divider: true },
              { label: "Cut", shortcut: "Ctrl+X", action: () => {
                if (selectedTimelineClipIds.length === 0) {
                  toast("No clips selected", "warning");
                  return;
                }
                cutClips(selectedTimelineClipIds);
                toast(`Cut ${selectedTimelineClipIds.length} clip(s)`, "success");
              }},
              { label: "Copy", shortcut: "Ctrl+C", action: () => {
                if (selectedTimelineClipIds.length === 0) {
                  toast("No clips selected", "warning");
                  return;
                }
                copyClips(selectedTimelineClipIds);
                toast(`Copied ${selectedTimelineClipIds.length} clip(s)`, "success");
              }},
              { label: "Paste", shortcut: "Ctrl+V", action: () => {
                const targetTrack = timelineTracks[0]?.id;
                if (!targetTrack) {
                  toast("No track to paste into", "warning");
                  return;
                }
                pasteClips(targetTrack, 0);
                toast("Pasted clip(s)", "success");
              }},
            ]},
            { label: "View", items: [
              { label: "Media Panel", shortcut: "", action: () => setShowLeft(!showLeft) },
              { label: "Inspector Panel", shortcut: "", action: () => setShowRight(!showRight) },
              { divider: true },
              { label: "Fullscreen", shortcut: "F11", action: () => {
                if (document.fullscreenElement) {
                  document.exitFullscreen();
                } else {
                  document.documentElement.requestFullscreen();
                }
              }},
            ]},
            { label: "Timeline", items: [
              { label: "Add Track", shortcut: "", action: async () => {
                try {
                  await invoke("timeline_create_track", { track_type: "video", name: "Video Track" });
                  toast("Video track added", "success");
                } catch (e) {
                  toast(`Add track failed: ${e}`, "error");
                }
              }},
              { label: "Split Clip", shortcut: "Ctrl+K", action: () =>
                toast("Split Clip: needs playhead position — coming soon", "warning") },
            ]},
            { label: "Render", items: [
              { label: "Quick Export", shortcut: "Ctrl+M", action: () =>
                toast("Quick Export: dialog coming soon", "warning") },
              { label: "Render Queue", shortcut: "", action: async () => {
                try {
                  const jobs = await invoke("rendering_get_all_jobs") as any[];
                  toast(`Render queue: ${jobs.length} jobs`, "info");
                } catch (e) {
                  toast(`Render queue: ${e}`, "error");
                }
              }},
            ]},
            { label: "Help", items: [
              { label: "Documentation", shortcut: "", action: () =>
                toast("Docs: github.com/aether-video-editor", "info") },
              { label: "Keyboard Shortcuts", shortcut: "", action: () =>
                toast("Shortcuts: Ctrl+S Save, Ctrl+K Split, Space Play/Pause, F11 Fullscreen", "info") },
              { label: "About Aether", shortcut: "", action: () => setShowAbout(true) },
            ]},
          ].map((menu) => (
            <div key={menu.label} className="relative">
              <button
                onClick={() => setActiveMenu(activeMenu === menu.label ? null : menu.label)}
                className={`text-[11px] px-2 py-0.5 rounded transition-colors ${
                  activeMenu === menu.label
                    ? "text-white bg-[#2a2a2a]"
                    : "text-[#777] hover:text-white"
                }`}
              >
                {menu.label}
              </button>
              {activeMenu === menu.label && (
                <div className="absolute top-full right-0 mt-1 w-48 bg-[#1a1a1a] border border-[#2a2a2a] rounded-md shadow-xl py-1 z-50">
                  {menu.items.map((item, idx) =>
                    "divider" in item ? (
                      <div key={idx} className="my-1 border-t border-[#2a2a2a]" />
                    ) : (
                      <button
                        key={idx}
                        onClick={async () => {
                          setActiveMenu(null);
                          await item.action();
                        }}
                        className="w-full flex items-center justify-between px-3 py-1.5 text-[11px] text-[#ccc] hover:bg-[#252525] hover:text-white transition-colors"
                      >
                        <span>{item.label}</span>
                        {item.shortcut && (
                          <span className="text-[10px] text-[#555]">{item.shortcut}</span>
                        )}
                      </button>
                    )
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      </header>

      <div className="h-10 bg-[#141414] border-b border-[#222] flex items-center px-3 gap-2 shrink-0">
        <div className="flex items-center gap-0.5 bg-[#1a1a1a] rounded-md p-0.5">
          {[
            { id: "edit" as Workspace, label: "Edit", icon: Layout },
            { id: "preview" as Workspace, label: "Preview", icon: Play },
            { id: "nodes" as Workspace, label: "Nodes", icon: GitBranch },
            { id: "color" as Workspace, label: "Color", icon: Sparkles },
          ].map((t) => {
            const Icon = t.icon;
            return (
              <button
                key={t.id}
                onClick={() => setWs(t.id)}
                className={`flex items-center gap-1.5 px-3 py-1.5 rounded text-[11px] font-medium transition-all ${
                  ws === t.id
                    ? "bg-[#2d2d2d] text-white shadow-sm"
                    : "text-[#666] hover:text-[#aaa]"
                }`}
              >
                <Icon size={12} />
                {t.label}
              </button>
            );
          })}
        </div>

        <div className="w-px h-5 bg-[#222]" />

        <button
          onClick={() => setShowLeft(!showLeft)}
          className={`flex items-center gap-1 px-2 py-1 rounded text-[11px] transition-colors ${
            showLeft ? "text-blue-400 bg-blue-400/10" : "text-[#555] hover:text-[#aaa]"
          }`}
        >
          <Layers size={12} />
          Media
        </button>
        <button
          onClick={() => setShowRight(!showRight)}
          className={`flex items-center gap-1 px-2 py-1 rounded text-[11px] transition-colors ${
            showRight ? "text-blue-400 bg-blue-400/10" : "text-[#555] hover:text-[#aaa]"
          }`}
        >
          <Settings size={12} />
          Inspector
        </button>

        <div className="ml-auto">
          <button
            onClick={() => setCurrentProject(null)}
            className="flex items-center gap-1 px-2.5 py-1 text-[11px] text-[#777] hover:text-white transition-colors"
          >
            <FolderOpen size={12} />
            Projects
          </button>
        </div>
      </div>

      <div className="flex-1 flex overflow-hidden">
        {showLeft && ws === "edit" && (
          <aside className="w-[280px] bg-[#141414] border-r border-[#222] flex flex-col shrink-0">
            <div className="flex border-b border-[#222]">
              {[
                { id: "media" as LeftTab, label: "Media" },
                { id: "multicam" as LeftTab, label: "Multicam" },
                { id: "effects" as LeftTab, label: "Effects" },
                { id: "nodes" as LeftTab, label: "Nodes" },
                { id: "transitions" as LeftTab, label: "Transitions" },
                { id: "plugins" as LeftTab, label: "Plugins" },
              ].map((t) => (
                <button
                  key={t.id}
                  onClick={() => setLeftTab(t.id)}
                  className={`flex-1 py-2 text-[10px] font-semibold uppercase tracking-wider transition-colors border-b-2 ${
                    leftTab === t.id
                      ? "text-blue-400 border-blue-400 bg-[#1a1a1a]"
                      : "text-[#555] border-transparent hover:text-[#888]"
                  }`}
                >
                  {t.label}
                </button>
              ))}
            </div>
            <div className="flex-1 overflow-y-auto p-3">
              {leftTab === "media" && <MediaPool />}
              {leftTab === "multicam" && <MulticamPanel />}
              {leftTab === "effects" && <EffectsList />}
              {leftTab === "nodes" && <NodesList />}
              {leftTab === "transitions" && <TransitionsList />}
              {leftTab === "plugins" && <PluginManager />}
            </div>
          </aside>
        )}

        <div className="flex-1 flex flex-col min-w-0">
          <div className="flex-1 flex min-h-0">
            <div className="flex-1 bg-[#0a0a0a] relative flex flex-col">
              {ws === "edit" && (
                <>
                  <div className="flex-1 min-h-0">
                    <PreviewWindow />
                  </div>
                  <div className="h-7 bg-[#1a1a1a] border-t border-[#222] flex items-center px-3 gap-4">
                    <span className="text-[10px] text-[#444] font-mono">1920 x 1080</span>
                    <span className="text-[10px] text-[#444] font-mono">30.00 fps</span>
                    <span className="text-[10px] text-[#444] font-mono">16:9</span>
                    <div className="ml-auto flex items-center gap-3">
                      <span className="text-[10px] text-[#444]">Fit</span>
                      <span className="text-[10px] text-[#444]">100%</span>
                    </div>
                  </div>
                </>
              )}
              {ws === "preview" && <PreviewWindow />}
              {ws === "nodes" && <NodeGraphEditor />}
              {ws === "color" && <ColorScopes />}
            </div>

            {showRight && ws === "edit" && (
              <aside className="w-[260px] bg-[#141414] border-l border-[#222] shrink-0">
                <InspectorPanel />
              </aside>
            )}
          </div>

          {ws === "edit" && (
            <div className="h-[260px] border-t border-[#222] shrink-0 flex flex-col">
              <TimelineEditor />
            </div>
          )}
        </div>
      </div>

      {/* About Dialog */}
      {showAbout && (
        <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 backdrop-blur-sm">
          <div className="bg-[#1a1a1a] border border-[#2a2a2a] rounded-lg p-6 w-[360px] shadow-xl">
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-2.5">
                <div className="w-7 h-7 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-md flex items-center justify-center">
                  <Film size={14} className="text-white" />
                </div>
                <span className="text-[15px] font-semibold text-white">Aether</span>
              </div>
              <button
                onClick={() => setShowAbout(false)}
                className="text-[#555] hover:text-white transition-colors"
              >
                <X size={16} />
              </button>
            </div>
            <div className="space-y-3 text-[12px] text-[#888]">
              <p>
                <strong className="text-[#ccc]">Version:</strong> 0.1.0 (Alpha)
              </p>
              <p>
                <strong className="text-[#ccc]">Backend:</strong> Rust + Tauri + FFmpeg + GStreamer
              </p>
              <p>
                <strong className="text-[#ccc]">Frontend:</strong> Next.js + React + Tailwind CSS
              </p>
              <p>
                <strong className="text-[#ccc]">GPU:</strong> Metal (macOS)
              </p>
              <div className="border-t border-[#222] pt-3 mt-3">
                <p className="text-[11px] text-[#555] leading-relaxed">
                  Aether is a professional video editor built for speed.
                  Features real-time GPU compositing, node-based effects,
                  and cinematic color grading tools.
                </p>
              </div>
            </div>
            <div className="mt-5 flex justify-end">
              <button
                onClick={() => setShowAbout(false)}
                className="px-3 py-1.5 bg-[#252525] hover:bg-[#333] text-[#ccc] text-[11px] rounded transition-colors"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      <StatusBar />
    </div>
  );
}

function MediaPool() {
  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-[11px] font-semibold text-[#777] uppercase tracking-wider">Media Pool</span>
        <button className="text-[10px] px-2 py-1 bg-blue-600/15 text-blue-400 rounded hover:bg-blue-600/25 transition-colors">
          + Import
        </button>
      </div>
      <div className="flex flex-col items-center justify-center py-12 gap-3">
        <div className="w-12 h-12 rounded-full bg-[#1a1a1a] flex items-center justify-center border border-[#222]">
          <Film size={20} className="text-[#333]" />
        </div>
        <div className="text-center">
          <p className="text-[12px] text-[#555]">No media imported</p>
          <p className="text-[10px] text-[#333] mt-1">Import footage to get started</p>
        </div>
      </div>
    </div>
  );
}

function EffectsList() {
  const effects = ["Blur", "Sharpen", "Color Balance", "Curves", "LUT", "Chroma Key", "Glow", "Grain"];
  return (
    <div className="space-y-2">
      <span className="text-[11px] font-semibold text-[#777] uppercase tracking-wider">Video Effects</span>
      {effects.map((e) => (
        <div key={e} className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-[#1a1a1a] cursor-pointer transition-colors">
          <Sparkles size={12} className="text-[#444]" />
          <span className="text-[11px] text-[#999]">{e}</span>
        </div>
      ))}
    </div>
  );
}

function NodesList() {
  const nodes = ["Media Input", "Color Correct", "Transform", "Merge", "Blur", "Output"];
  return (
    <div className="space-y-2">
      <span className="text-[11px] font-semibold text-[#777] uppercase tracking-wider">Node Toolbox</span>
      {nodes.map((n) => (
        <div key={n} className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-[#1a1a1a] cursor-pointer transition-colors">
          <GitBranch size={12} className="text-[#444]" />
          <span className="text-[11px] text-[#999]">{n}</span>
        </div>
      ))}
    </div>
  );
}

function TransitionsList() {
  const transitions = ["Cross Dissolve", "Fade to Black", "Wipe Left", "Wipe Right", "Slide", "Zoom"];
  return (
    <div className="space-y-2">
      <span className="text-[11px] font-semibold text-[#777] uppercase tracking-wider">Transitions</span>
      {transitions.map((t) => (
        <div key={t} className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-[#1a1a1a] cursor-pointer transition-colors">
          <Type size={12} className="text-[#444]" />
          <span className="text-[11px] text-[#999]">{t}</span>
        </div>
      ))}
    </div>
  );
}
