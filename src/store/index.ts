import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';

// Types
export interface Project {
  id: string;
  name: string;
  path?: string;
  createdAt: Date;
  modifiedAt: Date;
  settings: ProjectSettings;
}

export interface ProjectSettings {
  resolution: { width: number; height: number };
  framerate: number;
  audioSampleRate: number;
  autoSave: boolean;
  autoSaveInterval: number;
  proxyEnabled: boolean;
  proxyResolution: '720p' | '1080p' | '4K';
  defaultExportFormat: 'mp4' | 'mov' | 'avi' | 'mkv' | 'webm';
}

export interface MediaItem {
  id: string;
  name: string;
  path: string;
  mediaType: 'video' | 'image' | 'audio';
  duration: number;
  size: number;
  resolution?: { width: number; height: number };
  framerate?: number;
  codec?: string;
  thumbnailPath?: string;
  addedAt: Date;
}

export interface TimelineClip {
  id: string;
  mediaId: string;
  trackId: string;
  startTime: number;
  duration: number;
  offset: number;
  speed: number;
  volume: number;
}

export interface TimelineTrack {
  id: string;
  name: string;
  trackType: 'video' | 'audio';
  locked: boolean;
  muted: boolean;
  volume: number;
}

export interface ExportState {
  isExporting: boolean;
  progress: number;
  status: 'idle' | 'preparing' | 'rendering' | 'encoding' | 'finalizing' | 'completed' | 'failed';
  currentFrame: number;
  totalFrames: number;
  speed?: number;
  timeRemaining?: number;
  error?: string;
}

export interface ImportState {
  isImporting: boolean;
  progress: number;
  status: 'idle' | 'analyzing' | 'importing' | 'completed' | 'failed';
  currentFile?: string;
  totalFiles: number;
  processedFiles: number;
  error?: string;
}

export interface CameraAngle {
  id: string;
  name: string;
  media_id: string;
  offset_ms: number;
  enabled: boolean;
}

export interface MulticamClip {
  id: string;
  name: string;
  angles: CameraAngle[];
  active_angle_id?: string;
  duration_ms: number;
}

export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  path: string;
  loaded: boolean;
  hooks: string[];
}

// Store State
interface AppState {
  // Project State
  currentProject: Project | null;
  recentProjects: Project[];
  
  // Media State
  mediaItems: MediaItem[];
  selectedMediaIds: Set<string>;
  
  // Timeline State
  timelineTracks: TimelineTrack[];
  timelineClips: TimelineClip[];
  selectedTimelineClipIds: string[];

  // Export State
  exportState: ExportState;
  
  // Import State
  importState: ImportState;

  // Multicam State
  multicamClips: MulticamClip[];

  // Plugin State
  plugins: PluginInfo[];

  // UI State
  sidebarOpen: boolean;
  sidebarWidth: number;
  previewPanelOpen: boolean;
  theme: 'dark' | 'light';

  // Undo/Redo History (not persisted)
  _undoHistory: { tracks: TimelineTrack[]; clips: TimelineClip[] }[];
  _redoHistory: { tracks: TimelineTrack[]; clips: TimelineClip[] }[];

  // Clipboard
  clipboardClips: TimelineClip[];

  // History Actions
  undo: () => void;
  redo: () => void;
  _pushHistory: () => void;

  // Clipboard Actions
  copyClips: (clipIds: string[]) => void;
  cutClips: (clipIds: string[]) => void;
  pasteClips: (trackId: string, timeOffset: number) => void;

  // Project Actions
  setCurrentProject: (project: Project | null) => void;
  updateProject: (updates: Partial<Project>) => void;
  addRecentProject: (project: Project) => void;
  removeRecentProject: (projectId: string) => void;
  
  // Media Actions
  addMediaItem: (item: MediaItem) => void;
  removeMediaItem: (mediaId: string) => void;
  updateMediaItem: (mediaId: string, updates: Partial<MediaItem>) => void;
  selectMediaItem: (mediaId: string) => void;
  deselectMediaItem: (mediaId: string) => void;
  selectAllMediaItems: () => void;
  deselectAllMediaItems: () => void;
  clearMediaItems: () => void;
  
  // Timeline Actions
  addTimelineTrack: (track: TimelineTrack) => void;
  removeTimelineTrack: (trackId: string) => void;
  updateTimelineTrack: (trackId: string, updates: Partial<TimelineTrack>) => void;
  addTimelineClip: (clip: TimelineClip) => void;
  removeTimelineClip: (clipId: string) => void;
  updateTimelineClip: (clipId: string, updates: Partial<TimelineClip>) => void;
  moveTimelineClip: (clipId: string, newTrackId: string, newStartTime: number) => void;
  clearTimeline: () => void;
  selectTimelineClip: (clipId: string, multiSelect?: boolean) => void;
  deselectAllTimelineClips: () => void;
  
  // Export Actions
  startExport: (config: any) => void;
  updateExportProgress: (updates: Partial<ExportState>) => void;
  completeExport: () => void;
  failExport: (error: string) => void;
  resetExport: () => void;
  
  // Import Actions
  startImport: (files: string[]) => void;
  updateImportProgress: (updates: Partial<ImportState>) => void;
  completeImport: () => void;
  failImport: (error: string) => void;
  resetImport: () => void;

  // Multicam Actions
  setMulticamClips: (clips: MulticamClip[]) => void;
  addMulticamClip: (clip: MulticamClip) => void;
  removeMulticamClip: (clipId: string) => void;
  updateMulticamClip: (clipId: string, updates: Partial<MulticamClip>) => void;

  // Plugin Actions
  setPlugins: (plugins: PluginInfo[]) => void;
  addPlugin: (plugin: PluginInfo) => void;
  removePlugin: (pluginId: string) => void;

  // UI Actions
  toggleSidebar: () => void;
  setSidebarWidth: (width: number) => void;
  togglePreviewPanel: () => void;
  setTheme: (theme: 'dark' | 'light') => void;
}

// Create Store
const useAppStore = create<AppState>()(
  persist(
    (set, get) => ({
      // Initial State
      currentProject: null as Project | null,
      recentProjects: [],
      mediaItems: [],
      selectedMediaIds: new Set<string>(),
      timelineTracks: [],
      timelineClips: [],
      selectedTimelineClipIds: [],
      exportState: {
        isExporting: false,
        progress: 0,
        status: 'idle' as const,
        currentFrame: 0,
        totalFrames: 0,
      },
      importState: {
        isImporting: false,
        progress: 0,
        status: 'idle' as const,
        totalFiles: 0,
        processedFiles: 0,
      },
      multicamClips: [],
      plugins: [],
      sidebarOpen: true,
      sidebarWidth: 300,
      previewPanelOpen: true,
      theme: 'dark' as const,
      _undoHistory: [],
      _redoHistory: [],
      clipboardClips: [],

      // Helper to push timeline state to history
      _pushHistory: () => {
        const state = get();
        set({
          _undoHistory: [...state._undoHistory, { tracks: state.timelineTracks, clips: state.timelineClips }].slice(-50),
          _redoHistory: [],
        });
      },

      // History Actions
      undo: () => set((state) => {
        if (state._undoHistory.length === 0) return {};
        const prev = state._undoHistory[state._undoHistory.length - 1];
        return {
          timelineTracks: prev.tracks,
          timelineClips: prev.clips,
          _undoHistory: state._undoHistory.slice(0, -1),
          _redoHistory: [...state._redoHistory, { tracks: state.timelineTracks, clips: state.timelineClips }],
        };
      }),

      redo: () => set((state) => {
        if (state._redoHistory.length === 0) return {};
        const next = state._redoHistory[state._redoHistory.length - 1];
        return {
          timelineTracks: next.tracks,
          timelineClips: next.clips,
          _undoHistory: [...state._undoHistory, { tracks: state.timelineTracks, clips: state.timelineClips }],
          _redoHistory: state._redoHistory.slice(0, -1),
        };
      }),

      // Clipboard Actions
      copyClips: (clipIds) => set((state) => {
        const clips = state.timelineClips.filter((c) => clipIds.includes(c.id));
        return { clipboardClips: clips.map((c) => ({ ...c, id: `clip_${Date.now()}_${Math.random().toString(36).slice(2, 7)}` })) };
      }),

      cutClips: (clipIds) => set((state) => {
        const clips = state.timelineClips.filter((c) => clipIds.includes(c.id));
        return {
          _undoHistory: [...state._undoHistory, { tracks: state.timelineTracks, clips: state.timelineClips }].slice(-50),
          _redoHistory: [],
          clipboardClips: clips.map((c) => ({ ...c, id: `clip_${Date.now()}_${Math.random().toString(36).slice(2, 7)}` })),
          timelineClips: state.timelineClips.filter((c) => !clipIds.includes(c.id)),
        };
      }),

      pasteClips: (trackId, timeOffset) => set((state) => {
        if (state.clipboardClips.length === 0) return {};
        const minStart = Math.min(...state.clipboardClips.map((c) => c.startTime));
        const pasted = state.clipboardClips.map((c) => ({
          ...c,
          id: `clip_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
          trackId,
          startTime: c.startTime - minStart + timeOffset,
        }));
        return {
          _undoHistory: [...state._undoHistory, { tracks: state.timelineTracks, clips: state.timelineClips }].slice(-50),
          _redoHistory: [],
          timelineClips: [...state.timelineClips, ...pasted],
        };
      }),

      // Project Actions
      setCurrentProject: (project) => set({ currentProject: project }),
      
      updateProject: (updates) => set((state) => ({
        currentProject: state.currentProject
          ? { ...state.currentProject, ...updates, modifiedAt: new Date() }
          : null,
      })),
      
      addRecentProject: (project) => set((state) => {
        const filtered = state.recentProjects.filter((p) => p.id !== project.id);
        return {
          recentProjects: [project, ...filtered].slice(0, 10),
        };
      }),
      
      removeRecentProject: (projectId) => set((state) => ({
        recentProjects: state.recentProjects.filter((p) => p.id !== projectId),
      })),
      
      // Media Actions
      addMediaItem: (item) => set((state) => ({
        mediaItems: [...state.mediaItems, item],
      })),
      
      removeMediaItem: (mediaId) => set((state) => ({
        mediaItems: state.mediaItems.filter((m) => m.id !== mediaId),
        selectedMediaIds: new Set([...state.selectedMediaIds].filter((id) => id !== mediaId)),
        timelineClips: state.timelineClips.filter((c) => c.mediaId !== mediaId),
      })),
      
      updateMediaItem: (mediaId, updates) => set((state) => ({
        mediaItems: state.mediaItems.map((m) =>
          m.id === mediaId ? { ...m, ...updates } : m
        ),
      })),
      
      selectMediaItem: (mediaId) => set((state) => ({
        selectedMediaIds: new Set([...state.selectedMediaIds, mediaId]),
      })),
      
      deselectMediaItem: (mediaId) => set((state) => {
        const newSelected = new Set(state.selectedMediaIds);
        newSelected.delete(mediaId);
        return { selectedMediaIds: newSelected };
      }),
      
      selectAllMediaItems: () => set((state) => ({
        selectedMediaIds: new Set(state.mediaItems.map((m) => m.id)),
      })),
      
      deselectAllMediaItems: () => set({ selectedMediaIds: new Set() }),
      
      clearMediaItems: () => set({
        mediaItems: [],
        selectedMediaIds: new Set(),
      }),
      
      // Timeline Actions
      addTimelineTrack: (track) => set((state) => ({
        _undoHistory: [...state._undoHistory, { tracks: state.timelineTracks, clips: state.timelineClips }].slice(-50),
        _redoHistory: [],
        timelineTracks: [...state.timelineTracks, track],
      })),

      removeTimelineTrack: (trackId) => set((state) => ({
        _undoHistory: [...state._undoHistory, { tracks: state.timelineTracks, clips: state.timelineClips }].slice(-50),
        _redoHistory: [],
        timelineTracks: state.timelineTracks.filter((t) => t.id !== trackId),
        timelineClips: state.timelineClips.filter((c) => c.trackId !== trackId),
      })),

      updateTimelineTrack: (trackId, updates) => set((state) => ({
        timelineTracks: state.timelineTracks.map((t) =>
          t.id === trackId ? { ...t, ...updates } : t
        ),
      })),

      addTimelineClip: (clip) => set((state) => ({
        _undoHistory: [...state._undoHistory, { tracks: state.timelineTracks, clips: state.timelineClips }].slice(-50),
        _redoHistory: [],
        timelineClips: [...state.timelineClips, clip],
      })),

      removeTimelineClip: (clipId) => set((state) => ({
        _undoHistory: [...state._undoHistory, { tracks: state.timelineTracks, clips: state.timelineClips }].slice(-50),
        _redoHistory: [],
        timelineClips: state.timelineClips.filter((c) => c.id !== clipId),
      })),

      updateTimelineClip: (clipId, updates) => set((state) => ({
        timelineClips: state.timelineClips.map((c) =>
          c.id === clipId ? { ...c, ...updates } : c
        ),
      })),

      moveTimelineClip: (clipId, newTrackId, newStartTime) => set((state) => ({
        _undoHistory: [...state._undoHistory, { tracks: state.timelineTracks, clips: state.timelineClips }].slice(-50),
        _redoHistory: [],
        timelineClips: state.timelineClips.map((c) =>
          c.id === clipId ? { ...c, trackId: newTrackId, startTime: newStartTime } : c
        ),
      })),

      clearTimeline: () => set((state) => ({
        _undoHistory: [...state._undoHistory, { tracks: state.timelineTracks, clips: state.timelineClips }].slice(-50),
        _redoHistory: [],
        timelineTracks: [],
        timelineClips: [],
      })),

      selectTimelineClip: (clipId, multiSelect = false) => set((state) => ({
        selectedTimelineClipIds: multiSelect
          ? state.selectedTimelineClipIds.includes(clipId)
            ? state.selectedTimelineClipIds.filter((id) => id !== clipId)
            : [...state.selectedTimelineClipIds, clipId]
          : [clipId],
      })),

      deselectAllTimelineClips: () => set({ selectedTimelineClipIds: [] }),

      // Export Actions
      startExport: (config) => set({
        exportState: {
          isExporting: true,
          progress: 0,
          status: 'preparing',
          currentFrame: 0,
          totalFrames: config.totalFrames || 0,
        },
      }),
      
      updateExportProgress: (updates) => set((state) => ({
        exportState: { ...state.exportState, ...updates },
      })),
      
      completeExport: () => set((state) => ({
        exportState: {
          ...state.exportState,
          isExporting: false,
          progress: 100,
          status: 'completed',
        },
      })),
      
      failExport: (error) => set((state) => ({
        exportState: {
          ...state.exportState,
          isExporting: false,
          status: 'failed',
          error,
        },
      })),
      
      resetExport: () => set({
        exportState: {
          isExporting: false,
          progress: 0,
          status: 'idle',
          currentFrame: 0,
          totalFrames: 0,
        },
      }),
      
      // Import Actions
      startImport: (files) => set({
        importState: {
          isImporting: true,
          progress: 0,
          status: 'analyzing',
          totalFiles: files.length,
          processedFiles: 0,
        },
      }),
      
      updateImportProgress: (updates) => set((state) => ({
        importState: { ...state.importState, ...updates },
      })),
      
      completeImport: () => set((state) => ({
        importState: {
          ...state.importState,
          isImporting: false,
          progress: 100,
          status: 'completed',
        },
      })),
      
      failImport: (error) => set((state) => ({
        importState: {
          ...state.importState,
          isImporting: false,
          status: 'failed',
          error,
        },
      })),
      
      resetImport: () => set({
        importState: {
          isImporting: false,
          progress: 0,
          status: 'idle',
          totalFiles: 0,
          processedFiles: 0,
        },
      }),

      // Multicam Actions
      setMulticamClips: (clips) => set({ multicamClips: clips }),

      addMulticamClip: (clip) => set((state) => ({
        multicamClips: [...state.multicamClips, clip],
      })),

      removeMulticamClip: (clipId) => set((state) => ({
        multicamClips: state.multicamClips.filter((c) => c.id !== clipId),
      })),

      updateMulticamClip: (clipId, updates) => set((state) => ({
        multicamClips: state.multicamClips.map((c) =>
          c.id === clipId ? { ...c, ...updates } : c
        ),
      })),

      // Plugin Actions
      setPlugins: (plugins) => set({ plugins }),

      addPlugin: (plugin) => set((state) => ({
        plugins: [...state.plugins, plugin],
      })),

      removePlugin: (pluginId) => set((state) => ({
        plugins: state.plugins.filter((p) => p.id !== pluginId),
      })),

      // UI Actions
      toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
      
      setSidebarWidth: (width) => set({ sidebarWidth: width }),
      
      togglePreviewPanel: () => set((state) => ({ previewPanelOpen: !state.previewPanelOpen })),
      
      setTheme: (theme) => set({ theme }),
    }),
    {
      name: 'aether-storage',
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        // Persist only specific state
        recentProjects: state.recentProjects,
        sidebarOpen: state.sidebarOpen,
        sidebarWidth: state.sidebarWidth,
        previewPanelOpen: state.previewPanelOpen,
        theme: state.theme,
      }),
    }
  )
);

// Selectors
export const useCurrentProject = () => useAppStore((state) => state.currentProject);
export const useRecentProjects = () => useAppStore((state) => state.recentProjects);
export const useMediaItems = () => useAppStore((state) => state.mediaItems);
export const useSelectedMediaIds = () => useAppStore((state) => state.selectedMediaIds);
export const useTimelineTracks = () => useAppStore((state) => state.timelineTracks);
export const useTimelineClips = () => useAppStore((state) => state.timelineClips);
export const useExportState = () => useAppStore((state) => state.exportState);
export const useImportState = () => useAppStore((state) => state.importState);
export const useMulticamClips = () => useAppStore((state) => state.multicamClips);
export const usePlugins = () => useAppStore((state) => state.plugins);
export const useUISettings = () => useAppStore((state) => ({
  sidebarOpen: state.sidebarOpen,
  sidebarWidth: state.sidebarWidth,
  previewPanelOpen: state.previewPanelOpen,
  theme: state.theme,
}));

export default useAppStore;
