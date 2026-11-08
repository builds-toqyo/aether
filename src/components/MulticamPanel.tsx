"use client";

import React, { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "./Toast";
import useAppStore, { useMulticamClips, useSelectedMediaIds, useMediaItems, type MulticamClip } from "@/store";
import { Video, Plus, Trash2, RefreshCw, Camera, Check } from "lucide-react";

export function MulticamPanel() {
  const clips = useMulticamClips();
  const selectedMediaIds = useSelectedMediaIds();
  const mediaItems = useMediaItems();
  const setMulticamClips = useAppStore((s: { setMulticamClips: (c: MulticamClip[]) => void }) => s.setMulticamClips);
  const updateMulticamClip = useAppStore((s: { updateMulticamClip: (id: string, u: Partial<MulticamClip>) => void }) => s.updateMulticamClip);
  const removeMulticamClip = useAppStore((s: { removeMulticamClip: (id: string) => void }) => s.removeMulticamClip);
  const addMulticamClip = useAppStore((s: { addMulticamClip: (c: MulticamClip) => void }) => s.addMulticamClip);
  const { toast } = useToast();
  const [clipName, setClipName] = useState("");

  const refreshClips = useCallback(async () => {
    try {
      const list: any[] = await invoke("multicam_list_clips");
      setMulticamClips(
        list.map((c) => ({
          id: c.id,
          name: c.name,
          angles: c.angles || [],
          active_angle_id: c.active_angle_id,
          duration_ms: c.duration_ms || 0,
        }))
      );
    } catch (e) {
      toast(`Failed to refresh multicam clips: ${e}`, "error");
    }
  }, [setMulticamClips, toast]);

  useEffect(() => {
    refreshClips();
  }, [refreshClips]);

  const handleCreate = useCallback(async () => {
    if (selectedMediaIds.size < 2) {
      toast("Select at least 2 media items to create a multicam clip", "warning");
      return;
    }
    const ids = Array.from(selectedMediaIds);
    try {
      const result = await invoke("multicam_create", {
        name: clipName.trim() || `Multicam ${clips.length + 1}`,
        angle_media_ids: ids,
      });
      const created = (result as any)?.clip;
      if (created) {
        addMulticamClip({
          id: created.id,
          name: created.name,
          angles: created.angles || [],
          active_angle_id: created.active_angle_id,
          duration_ms: created.duration_ms || 0,
        });
        toast(`Created multicam clip: ${created.name}`, "success");
        setClipName("");
      }
    } catch (e) {
      toast(`Create failed: ${e}`, "error");
    }
  }, [selectedMediaIds, clipName, clips.length, addMulticamClip, toast]);

  const handleSetActive = useCallback(async (clipId: string, angleId: string) => {
    try {
      await invoke("multicam_set_active_angle", { clip_id: clipId, angle_id: angleId });
      updateMulticamClip(clipId, { active_angle_id: angleId });
      toast("Active angle switched", "success");
    } catch (e) {
      toast(`Switch failed: ${e}`, "error");
    }
  }, [updateMulticamClip, toast]);

  const handleDelete = useCallback(async (clipId: string) => {
    try {
      await invoke("multicam_delete_clip", { clip_id: clipId });
      removeMulticamClip(clipId);
      toast("Multicam clip deleted", "success");
    } catch (e) {
      toast(`Delete failed: ${e}`, "error");
    }
  }, [removeMulticamClip, toast]);

  const selectedMedia = mediaItems.filter((m) => selectedMediaIds.has(m.id));

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-[11px] font-semibold text-[#777] uppercase tracking-wider">
          Multicam
        </span>
        <button
          onClick={refreshClips}
          className="p-1 text-[#555] hover:text-blue-400 transition-colors"
          title="Refresh"
        >
          <RefreshCw size={12} />
        </button>
      </div>

      <div className="space-y-2">
        <div className="flex items-center gap-1.5">
          <input
            type="text"
            value={clipName}
            onChange={(e) => setClipName(e.target.value)}
            placeholder="Clip name..."
            className="flex-1 bg-[#1a1a1a] border border-[#222] rounded px-2 py-1 text-[10px] text-[#ccc] outline-none focus:border-blue-500"
          />
          <button
            onClick={handleCreate}
            disabled={selectedMediaIds.size < 2}
            className="flex items-center gap-1 px-2 py-1 bg-blue-600/15 text-blue-400 rounded hover:bg-blue-600/25 transition-colors disabled:opacity-50 shrink-0"
          >
            <Plus size={11} />
            <span className="text-[10px]">Create</span>
          </button>
        </div>
        {selectedMediaIds.size > 0 && (
          <p className="text-[10px] text-[#555]">
            {selectedMediaIds.size} media selected: {selectedMedia.map((m) => m.name).join(", ")}
          </p>
        )}
      </div>

      {clips.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-10 gap-3">
          <div className="w-10 h-10 rounded-full bg-[#1a1a1a] flex items-center justify-center border border-[#222]">
            <Video size={18} className="text-[#333]" />
          </div>
          <div className="text-center">
            <p className="text-[12px] text-[#555]">No multicam clips</p>
            <p className="text-[10px] text-[#333] mt-1">Select 2+ media items and click Create</p>
          </div>
        </div>
      ) : (
        <div className="space-y-2">
          {clips.map((clip) => (
            <div
              key={clip.id}
              className="rounded bg-[#1a1a1a] border border-[#222] hover:border-[#333] transition-colors overflow-hidden"
            >
              <div className="flex items-center justify-between px-2 py-1.5 border-b border-[#222]">
                <div className="flex items-center gap-1.5 min-w-0">
                  <Video size={12} className="text-blue-400 shrink-0" />
                  <span className="text-[11px] text-[#ccc] font-medium truncate">
                    {clip.name}
                  </span>
                </div>
                <button
                  onClick={() => handleDelete(clip.id)}
                  className="p-0.5 text-[#555] hover:text-red-400 transition-colors shrink-0"
                  title="Delete"
                >
                  <Trash2 size={11} />
                </button>
              </div>
              <div className="px-2 py-1.5 space-y-1">
                {clip.angles.map((angle) => {
                  const isActive = clip.active_angle_id === angle.id;
                  return (
                    <button
                      key={angle.id}
                      onClick={() => handleSetActive(clip.id, angle.id)}
                      className={`w-full flex items-center gap-1.5 px-1.5 py-1 rounded text-[10px] transition-colors ${
                        isActive
                          ? "bg-blue-600/15 text-blue-400"
                          : "text-[#888] hover:bg-[#252525] hover:text-[#ccc]"
                      }`}
                    >
                      <Camera size={10} className="shrink-0" />
                      <span className="truncate flex-1 text-left">{angle.name}</span>
                      {isActive && <Check size={10} className="shrink-0" />}
                    </button>
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
