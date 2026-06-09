import React, { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Tauri API types
export interface PreviewAPI {
  get_scope_data: () => Promise<{ data: any }>;
  get_preview_frame: () => Promise<{ data: ImageData | null }>;
  play_preview: () => Promise<void>;
  pause_preview: () => Promise<void>;
  seek_preview: (time: number) => Promise<void>;
}

export interface EditingAPI {
  create_project: (request: any) => Promise<any>;
  save_project: (request: any) => Promise<any>;
  load_project: (request: any) => Promise<any>;
  import_media: (request: any) => Promise<any>;
}

export interface RenderingAPI {
  start_rendering: (request: any) => Promise<any>;
  cancel_rendering: (jobId: string) => Promise<any>;
  get_rendering_status: (jobId: string) => Promise<any>;
}

export interface TauriAPIs {
  preview: PreviewAPI;
  editing: EditingAPI;
  rendering: RenderingAPI;
}

// Actual Tauri API implementation
const previewAPI: PreviewAPI = {
  get_scope_data: async () => {
    return await invoke('preview_get_scope_data');
  },

  get_preview_frame: async () => {
    return await invoke('preview_get_frame');
  },

  play_preview: async () => {
    return await invoke('preview_play');
  },

  pause_preview: async () => {
    return await invoke('preview_pause');
  },

  seek_preview: async (time: number) => {
    return await invoke('preview_seek', { time });
  },
};

const editingAPI: EditingAPI = {
  create_project: async (request: any) => {
    return await invoke('project_create', { request });
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
  start_rendering: async (request: any) => {
    return await invoke('rendering_start', { request });
  },

  cancel_rendering: async (jobId: string) => {
    return await invoke('rendering_cancel', { jobId });
  },

  get_rendering_status: async (jobId: string) => {
    return await invoke('rendering_get_status', { jobId });
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
