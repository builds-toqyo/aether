"use client";

import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useToast } from "@/components/Toast";
import { X, Settings, Download } from "lucide-react";

type ExportFormat = "mp4" | "mov" | "webm";
type ExportQuality = "low" | "medium" | "high" | "ultra";

interface ExportDialogProps {
  onClose: () => void;
}

export function ExportDialog({ onClose }: ExportDialogProps) {
  const { toast } = useToast();
  const [format, setFormat] = useState<ExportFormat>("mp4");
  const [quality, setQuality] = useState<ExportQuality>("high");
  const [hardwareAcceleration, setHardwareAcceleration] = useState(true);
  const [isExporting, setIsExporting] = useState(false);
  const [progress, setProgress] = useState(0);

  const qualitySettings = {
    low: { label: "Low (720p)", bitrate: "5M" },
    medium: { label: "Medium (1080p)", bitrate: "10M" },
    high: { label: "High (1080p)", bitrate: "20M" },
    ultra: { label: "Ultra (4K)", bitrate: "50M" },
  };

  const handleExport = async () => {
    try {
      const selected = await save({
        filters: [{
          name: format.toUpperCase(),
          extensions: [format]
        }],
        defaultPath: `export.${format}`
      });

      if (!selected) return;

      setIsExporting(true);
      setProgress(0);

      await invoke("rendering_start_job", {
        request: {
          output_path: selected,
          format,
          video_codec: "h264",
          audio_codec: "aac",
          video_bitrate: qualitySettings[quality].bitrate,
          audio_bitrate: "192K",
          hardware_acceleration: hardwareAcceleration,
        }
      });

      toast("Export started successfully", "success");
      onClose();
    } catch (e) {
      toast(`Export failed: ${e}`, "error");
      setIsExporting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="bg-[#1a1a1a] border border-[#2a2a2a] rounded-lg p-6 w-[420px] shadow-xl">
        <div className="flex items-center justify-between mb-5">
          <div className="flex items-center gap-2.5">
            <div className="w-7 h-7 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-md flex items-center justify-center">
              <Download size={14} className="text-white" />
            </div>
            <span className="text-[15px] font-semibold text-white">Export Settings</span>
          </div>
          <button
            onClick={onClose}
            disabled={isExporting}
            className="text-[#555] hover:text-white transition-colors disabled:opacity-50"
          >
            <X size={16} />
          </button>
        </div>

        <div className="space-y-4">
          <div>
            <label className="block text-[11px] font-semibold text-[#777] uppercase tracking-wider mb-2">
              Format
            </label>
            <div className="grid grid-cols-3 gap-2">
              {(["mp4", "mov", "webm"] as ExportFormat[]).map((fmt) => (
                <button
                  key={fmt}
                  onClick={() => setFormat(fmt)}
                  disabled={isExporting}
                  className={`px-3 py-2 rounded text-[11px] font-medium transition-colors ${
                    format === fmt
                      ? "bg-blue-600 text-white"
                      : "bg-[#252525] text-[#888] hover:bg-[#333] hover:text-white"
                  } disabled:opacity-50`}
                >
                  {fmt.toUpperCase()}
                </button>
              ))}
            </div>
          </div>

          <div>
            <label className="block text-[11px] font-semibold text-[#777] uppercase tracking-wider mb-2">
              Quality
            </label>
            <div className="grid grid-cols-2 gap-2">
              {(["low", "medium", "high", "ultra"] as ExportQuality[]).map((q) => (
                <button
                  key={q}
                  onClick={() => setQuality(q)}
                  disabled={isExporting}
                  className={`px-3 py-2 rounded text-[11px] font-medium transition-colors ${
                    quality === q
                      ? "bg-blue-600 text-white"
                      : "bg-[#252525] text-[#888] hover:bg-[#333] hover:text-white"
                  } disabled:opacity-50`}
                >
                  {qualitySettings[q].label}
                </button>
              ))}
            </div>
          </div>

          <div className="flex items-center justify-between py-2 border-t border-[#222]">
            <div className="flex items-center gap-2">
              <Settings size={14} className="text-[#666]" />
              <span className="text-[12px] text-[#ccc]">Hardware Acceleration</span>
            </div>
            <button
              onClick={() => setHardwareAcceleration(!hardwareAcceleration)}
              disabled={isExporting}
              className={`w-10 h-5 rounded-full transition-colors ${
                hardwareAcceleration ? "bg-blue-600" : "bg-[#333]"
              } disabled:opacity-50`}
            >
              <div
                className={`w-4 h-4 bg-white rounded-full transition-transform ${
                  hardwareAcceleration ? "translate-x-5" : "translate-x-0.5"
                }`}
              />
            </button>
          </div>

          {isExporting && (
            <div className="py-2">
              <div className="flex items-center justify-between mb-1">
                <span className="text-[11px] text-[#888]">Exporting...</span>
                <span className="text-[11px] text-[#888]">{progress.toFixed(0)}%</span>
              </div>
              <div className="h-1.5 bg-[#222] rounded-full overflow-hidden">
                <div
                  className="h-full bg-blue-600 transition-all duration-300"
                  style={{ width: `${progress}%` }}
                />
              </div>
            </div>
          )}

          <div className="flex gap-2 pt-2">
            <button
              onClick={onClose}
              disabled={isExporting}
              className="flex-1 px-3 py-2 bg-[#252525] hover:bg-[#333] text-[#ccc] text-[11px] rounded transition-colors disabled:opacity-50"
            >
              Cancel
            </button>
            <button
              onClick={handleExport}
              disabled={isExporting}
              className="flex-1 px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white text-[11px] font-medium rounded transition-colors disabled:opacity-50"
            >
              {isExporting ? "Exporting..." : "Export"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
