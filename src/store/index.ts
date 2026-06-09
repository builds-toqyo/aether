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
  
  // Export State
  exportState: ExportState;
  
  // Import State
  importState: ImportState;
  
  // UI State
  sidebarOpen: boolean;
  sidebarWidth: number;
  previewPanelOpen: boolean;
  theme: 'dark' | 'light';
  
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
      currentProject: null,
      recentProjects: [],
      mediaItems: [],
      selectedMediaIds: new Set<string>(),
      timelineTracks: [],
      timelineClips: [],
      exportState: {
        isExporting: false,
        progress: 0,
        status: 'idle',
        currentFrame: 0,
        totalFrames: 0,
      },
      importState: {
        isImporting: false,
        progress: 0,
        status: 'idle',
        totalFiles: 0,
        processedFiles: 0,
      },
      sidebarOpen: true,
      sidebarWidth: 300,
      previewPanelOpen: true,
      theme: 'dark',
      
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
        timelineTracks: [...state.timelineTracks, track],
      })),
      
      removeTimelineTrack: (trackId) => set((state) => ({
        timelineTracks: state.timelineTracks.filter((t) => t.id !== trackId),
        timelineClips: state.timelineClips.filter((c) => c.trackId !== trackId),
      })),
      
      updateTimelineTrack: (trackId, updates) => set((state) => ({
        timelineTracks: state.timelineTracks.map((t) =>
          t.id === trackId ? { ...t, ...updates } : t
        ),
      })),
      
      addTimelineClip: (clip) => set((state) => ({
        timelineClips: [...state.timelineClips, clip],
      })),
      
      removeTimelineClip: (clipId) => set((state) => ({
        timelineClips: state.timelineClips.filter((c) => c.id !== clipId),
      })),
      
      updateTimelineClip: (clipId, updates) => set((state) => ({
        timelineClips: state.timelineClips.map((c) =>
          c.id === clipId ? { ...c, ...updates } : c
        ),
      })),
      
      moveTimelineClip: (clipId, newTrackId, newStartTime) => set((state) => ({
        timelineClips: state.timelineClips.map((c) =>
          c.id === clipId ? { ...c, trackId: newTrackId, startTime: newStartTime } : c
        ),
      })),
      
      clearTimeline: () => set({
        timelineTracks: [],
        timelineClips: [],
      }),
      
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
export const useUISettings = () => useAppStore((state) => ({
  sidebarOpen: state.sidebarOpen,
  sidebarWidth: state.sidebarWidth,
  previewPanelOpen: state.previewPanelOpen,
  theme: state.theme,
}));

export default useAppStore;
