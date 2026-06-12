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

export interface TauriAPIs {
  preview: PreviewAPI;
  editing: EditingAPI;
  rendering: RenderingAPI;
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
