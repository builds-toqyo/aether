import React, { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Tauri API types
export interface PreviewAPI {
  get_performance_stats: () => Promise<any>;
  get_preview_frame: (timestamp?: number, quality?: string, format?: string) => Promise<any>;
  play_preview: () => Promise<void>;
  pause_preview: () => Promise<void>;
  stop_preview: () => Promise<void>;
  seek_preview: (time: number) => Promise<void>;
}

export interface EditingAPI {
  init_project: (request: any) => Promise<any>;
  save_project: (request: any) => Promise<any>;
  load_project: (request: any) => Promise<any>;
  import_media: (request: any) => Promise<any>;
}

export interface RenderingAPI {
  start_job: (request: any) => Promise<any>;
  cancel_job: (jobId: string) => Promise<any>;
  get_job_status: (jobId: string) => Promise<any>;
}

export interface MulticamAPI {
  create: (name: string, angleMediaIds: string[]) => Promise<any>;
  addAngle: (clipId: string, name: string, mediaId: string) => Promise<any>;
  removeAngle: (clipId: string, angleId: string) => Promise<any>;
  setActiveAngle: (clipId: string, angleId: string) => Promise<any>;
  getClip: (clipId: string) => Promise<any>;
  listClips: () => Promise<any[]>;
  deleteClip: (clipId: string) => Promise<any>;
  syncByTimecode: (clipId: string) => Promise<any>;
}

export interface PluginAPI {
  load: (path: string) => Promise<any>;
  unload: (pluginId: string) => Promise<any>;
  list: () => Promise<any[]>;
  getInfo: (pluginId: string) => Promise<any>;
  scanDirectory: (directory: string) => Promise<any[]>;
}

export interface TauriAPIs {
  preview: PreviewAPI;
  editing: EditingAPI;
  rendering: RenderingAPI;
  multicam: MulticamAPI;
  plugin: PluginAPI;
}

// Actual Tauri API implementation
const previewAPI: PreviewAPI = {
  get_performance_stats: async () => {
    return await invoke('preview_get_performance_stats');
  },

  get_preview_frame: async (timestamp = 0, quality?: string, format?: string) => {
    const payload: any = { timestamp };
    if (quality) payload.quality = quality;
    if (format) payload.format = format;
    return await invoke('preview_get_frame', payload);
  },

  play_preview: async () => {
    return await invoke('preview_playback_control', { action: 'Play' });
  },

  pause_preview: async () => {
    return await invoke('preview_playback_control', { action: 'Pause' });
  },

  stop_preview: async () => {
    return await invoke('preview_playback_control', { action: 'Stop' });
  },

  seek_preview: async (time: number) => {
    return await invoke('preview_seek', { time });
  },
};

const editingAPI: EditingAPI = {
  init_project: async (request: any) => {
    return await invoke('project_init', request);
  },

  save_project: async (request: any) => {
    return await invoke('project_save', { request });
  },

  load_project: async (request: any) => {
    return await invoke('project_load', { request });
  },

  import_media: async (request: any) => {
    return await invoke('media_import', { request });
  },
};

const renderingAPI: RenderingAPI = {
  start_job: async (request: any) => {
    return await invoke('rendering_start_job', request);
  },

  cancel_job: async (jobId: string) => {
    return await invoke('rendering_cancel_job', { job_id: jobId });
  },

  get_job_status: async (jobId: string) => {
    return await invoke('rendering_get_job_status', { job_id: jobId });
  },
};

const multicamAPI: MulticamAPI = {
  create: async (name: string, angleMediaIds: string[]) => {
    return await invoke('multicam_create', { name, angle_media_ids: angleMediaIds });
  },

  addAngle: async (clipId: string, name: string, mediaId: string) => {
    return await invoke('multicam_add_angle', { clip_id: clipId, name, media_id: mediaId });
  },

  removeAngle: async (clipId: string, angleId: string) => {
    return await invoke('multicam_remove_angle', { clip_id: clipId, angle_id: angleId });
  },

  setActiveAngle: async (clipId: string, angleId: string) => {
    return await invoke('multicam_set_active_angle', { clip_id: clipId, angle_id: angleId });
  },

  getClip: async (clipId: string) => {
    return await invoke('multicam_get_clip', { clip_id: clipId });
  },

  listClips: async () => {
    return await invoke('multicam_list_clips');
  },

  deleteClip: async (clipId: string) => {
    return await invoke('multicam_delete_clip', { clip_id: clipId });
  },

  syncByTimecode: async (clipId: string) => {
    return await invoke('multicam_sync_by_timecode', { clip_id: clipId });
  },
};

const pluginAPI: PluginAPI = {
  load: async (path: string) => {
    return await invoke('plugin_load', { path });
  },

  unload: async (pluginId: string) => {
    return await invoke('plugin_unload', { plugin_id: pluginId });
  },

  list: async () => {
    return await invoke('plugin_list');
  },

  getInfo: async (pluginId: string) => {
    return await invoke('plugin_get_info', { plugin_id: pluginId });
  },

  scanDirectory: async (directory: string) => {
    return await invoke('plugin_scan_directory', { directory });
  },
};

export const useTauriAPI = (): TauriAPIs => {
  const [isConnected, setIsConnected] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Check Tauri connection
  const checkConnection = useCallback(() => {
    if (typeof window !== 'undefined' && window.__TAURI__) {
      setIsConnected(true);
      setError(null);
    } else {
      setIsConnected(false);
      setError('Tauri API not available - please run in Tauri environment');
    }
  }, []);

  // Tauri APIs
  const apis: TauriAPIs = {
    preview: previewAPI,
    editing: editingAPI,
    rendering: renderingAPI,
    multicam: multicamAPI,
    plugin: pluginAPI,
  };

  // Check connection on mount
  React.useEffect(() => {
    checkConnection();
  }, [checkConnection]);

  // Log connection status for debugging
  React.useEffect(() => {
    if (error) {
      console.warn(error);
    }
  }, [error]);

  return apis;
};

// Type declaration for Tauri global
declare global {
  interface Window {
    __TAURI__?: any;
  }
}
