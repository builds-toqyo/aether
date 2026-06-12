"use client";

import React, { useState } from "react";
import useAppStore, { type Project } from "@/store";
import {
  Film,
  Plus,
  FolderOpen,
  Clock,
  Settings,
  ChevronRight,
  Zap,
  Layers,
  Wand2,
} from "lucide-react";

export const WelcomeScreen: React.FC = () => {
  const setCurrentProject = useAppStore((s) => s.setCurrentProject);
  const recentProjects = useAppStore((s) => s.recentProjects);
  const addRecentProject = useAppStore((s) => s.addRecentProject);
  const [showDialog, setShowDialog] = useState(false);
  const [newName, setNewName] = useState("");
  const dialogInputRef = React.useRef<HTMLInputElement>(null);

  React.useEffect(() => {
    if (showDialog && dialogInputRef.current) {
      dialogInputRef.current.focus();
      dialogInputRef.current.select();
    }
  }, [showDialog]);

  const createProject = (name: string) => {
    const trimmed = name.trim() || "Untitled Project";
    const newProject: Project = {
      id: `project_${Date.now()}`,
      name: trimmed,
      createdAt: new Date(),
      modifiedAt: new Date(),
      settings: {
        resolution: { width: 1920, height: 1080 },
        framerate: 30,
        audioSampleRate: 48000,
        autoSave: true,
        autoSaveInterval: 300,
        proxyEnabled: false,
        proxyResolution: "1080p",
        defaultExportFormat: "mp4",
      },
    };
    addRecentProject(newProject);
    setCurrentProject(newProject);
    setShowDialog(false);
    setNewName("");
  };

  const handleDialogKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") createProject(newName);
    if (e.key === "Escape") {
      setShowDialog(false);
      setNewName("");
    }
  };

  return (
    <div className="h-screen flex flex-col bg-[#0a0a0a] text-[#e0e0e0] overflow-hidden">
      <div className="h-10 flex items-center justify-between px-5 shrink-0">
        <div className="flex items-center gap-2.5">
          <div className="w-6 h-6 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-md flex items-center justify-center shadow-lg shadow-blue-500/20">
            <Film size={14} className="text-white" />
          </div>
          <span className="font-bold text-[13px] tracking-wide text-white">Aether</span>
        </div>
        <button className="p-1.5 text-[#555] hover:text-white transition-colors">
          <Settings size={16} />
        </button>
      </div>

      <div className="flex-1 flex overflow-hidden">
        <div className="flex-1 flex flex-col justify-center px-16 pb-20">
          <h1 className="text-5xl font-bold text-white mb-4 tracking-tight">
            Aether
          </h1>
          <p className="text-[#888] text-lg mb-8 max-w-md leading-relaxed">
            Professional video editing with real-time GPU compositing, node-based effects, and cinematic color tools.
          </p>

          <div className="flex items-center gap-3 mb-10">
            <button
              onClick={() => { setNewName("Untitled Project"); setShowDialog(true); }}
              className="flex items-center gap-2 px-5 py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-md text-sm font-semibold transition-all shadow-lg shadow-blue-600/20 active:scale-[0.98]"
            >
              <Plus size={16} />
              New Project
            </button>
            <button className="flex items-center gap-2 px-5 py-2.5 bg-[#1a1a1a] hover:bg-[#222] border border-[#2a2a2a] text-white rounded-md text-sm font-medium transition-all active:scale-[0.98]">
              <FolderOpen size={16} />
              Open Project
            </button>
          </div>

          <div className="flex items-center gap-3">
            <FeaturePill icon={<Zap size={12} />} label="GPU Accelerated" />
            <FeaturePill icon={<Layers size={12} />} label="Node Compositing" />
            <FeaturePill icon={<Wand2 size={12} />} label="Color Grading" />
          </div>
        </div>

        {/* Right: Recent Projects */}
        <div className="w-[420px] bg-[#111] border-l border-[#1f1f1f] p-6 overflow-y-auto">
          <div className="flex items-center gap-2 mb-6">
            <Clock size={14} className="text-[#555]" />
            <h2 className="text-xs font-semibold text-[#888] uppercase tracking-wider">
              Recent Projects
            </h2>
          </div>

          {recentProjects.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 gap-3">
              <div className="w-14 h-14 rounded-full bg-[#1a1a1a] flex items-center justify-center border border-[#222]">
                <FolderOpen size={24} className="text-[#333]" />
              </div>
              <p className="text-[13px] text-[#555]">No recent projects</p>
              <p className="text-[11px] text-[#333]">Create a new project to get started</p>
            </div>
          ) : (
            <div className="space-y-2">
              {recentProjects.map((project) => (
                <button
                  key={project.id}
                  onClick={() => setCurrentProject(project)}
                  className="w-full flex items-center gap-3 p-3 rounded-lg bg-[#161616] hover:bg-[#1a1a1a] border border-transparent hover:border-[#2a2a2a] transition-all text-left group"
                >
                  <div className="w-10 h-10 rounded bg-[#1f1f1f] flex items-center justify-center shrink-0">
                    <Film size={16} className="text-[#444] group-hover:text-blue-400 transition-colors" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-[13px] text-white font-medium truncate">
                      {project.name}
                    </p>
                    <p className="text-[11px] text-[#555]">
                      {project.modifiedAt
                        ? new Date(project.modifiedAt).toLocaleDateString()
                        : "Just now"}
                    </p>
                  </div>
                  <ChevronRight size={14} className="text-[#333] group-hover:text-[#666] transition-colors" />
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {showDialog && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
          <div className="bg-[#1a1a1a] border border-[#2a2a2a] rounded-lg p-5 w-80 shadow-xl">
            <h3 className="text-[14px] font-semibold text-white mb-3">New Project</h3>
            <label className="text-[11px] text-[#888] mb-1 block">Project Name</label>
            <input
              ref={dialogInputRef}
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={handleDialogKeyDown}
              className="w-full bg-[#252525] border border-[#333] rounded px-2.5 py-1.5 text-[13px] text-white outline-none focus:border-blue-500 mb-4"
            />
            <div className="flex justify-end gap-2">
              <button
                onClick={() => { setShowDialog(false); setNewName(""); }}
                className="px-3 py-1.5 text-[11px] text-[#888] hover:text-white transition-colors rounded"
              >
                Cancel
              </button>
              <button
                onClick={() => createProject(newName)}
                className="px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white text-[11px] font-medium rounded transition-colors"
              >
                Create
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

function FeaturePill({ icon, label }: { icon: React.ReactNode; label: string }) {
  return (
    <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-[#151515] border border-[#222] text-[11px] text-[#777]">
      {icon}
      {label}
    </div>
  );
}

export default WelcomeScreen;
