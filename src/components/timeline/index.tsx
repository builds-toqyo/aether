'use client';

import React, { useState, useRef, useCallback, useEffect } from 'react';
import { TimelineCanvas } from './TimelineCanvas';
import { TimelineControls } from './TimelineControls';
import { TimelineRuler } from './TimelineRuler';
import { TimelineProvider, useTimeline } from './TimelineContext';
import { Play, Pause, SkipBack, SkipForward, Scissors } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import useAppStore from '@/store';
import { formatTime } from '@/lib/utils';

// Types
interface TimelineClip {
  id: string;
  name: string;
  trackId: string;
  startTime: number;
  duration: number;
  type: 'video' | 'audio' | 'image';
  source: string;
  color: string;
}

interface TimelineTrack {
  id: string;
  name: string;
  type: 'video' | 'audio';
  height: number;
  muted: boolean;
  solo: boolean;
  locked: boolean;
  clips: TimelineClip[];
}

interface TimelineState {
  tracks: TimelineTrack[];
  currentTime: number;
  duration: number;
  zoom: number;
  scrollX: number;
  isPlaying: boolean;
  selectedClips: string[];
  playbackRate: number;
}

export const TimelineEditor: React.FC = () => {
  const [state, setState] = useState<TimelineState>({
    tracks: [],
    currentTime: 0,
    duration: 300, // 5 minutes in seconds
    zoom: 1,
    scrollX: 0,
    isPlaying: false,
    selectedClips: [],
    playbackRate: 1
  });

  const [isDragging, setIsDragging] = useState(false);
  const [dragMode, setDragMode] = useState<'move' | 'trim' | 'ripple' | 'roll'>('move');
  const [isTrimming, setIsTrimming] = useState(false);
  const [trimStart, setTrimStart] = useState<{ clipId: string; edge: 'start' | 'end'; time: number } | null>(null);

  const timelineRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLDivElement>(null);
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    const initializeTimeline = async () => {
      try {
        const timelineData = await invoke<any>('get_timeline_info');
        console.log('Timeline initialized:', timelineData);
        
        // Load existing timeline if available
        if (timelineData && timelineData.tracks) {
          setState(prev => ({
            ...prev,
            tracks: timelineData.tracks,
            duration: timelineData.duration || prev.duration,
            currentTime: timelineData.currentTime || prev.currentTime
          }));
        }
      } catch (error) {
        console.error('Failed to initialize timeline:', error);
        // Start with empty timeline if backend fails
        console.log('Starting with empty timeline');
      }
    };
    
    initializeTimeline();
  }, []);

  const handlePlay = useCallback(async () => {
    try {
      // Call backend to start/stop playback
      const action = state.isPlaying ? 'pause' : 'play';
      await invoke('timeline_playback_control', { action, currentTime: state.currentTime });
      
      // Update local state immediately for responsiveness
      setState(prev => ({ ...prev, isPlaying: !prev.isPlaying }));
      console.log(`Timeline ${action} successfully`);
    } catch (error) {
      console.error('Failed to control playback:', error);
      // Fallback to local state update
      setState(prev => ({ ...prev, isPlaying: !prev.isPlaying }));
    }
  }, [state.isPlaying, state.currentTime]);

  const handleStop = useCallback(async () => {
    try {
      // Call backend to stop playback
      await invoke('timeline_playback_control', { action: 'stop' });
      
      // Update local state
      setState(prev => ({ ...prev, isPlaying: false, currentTime: 0 }));
      console.log('Timeline stopped successfully');
    } catch (error) {
      console.error('Failed to stop timeline:', error);
      // Fallback to local state update
      setState(prev => ({ ...prev, isPlaying: false, currentTime: 0 }));
    }
  }, []);

  const handleSkipBack = useCallback(async () => {
    try {
      const newTime = Math.max(0, state.currentTime - 10);
      
      // Call backend to seek to new time
      await invoke('timeline_seek', { time: newTime });
      
      // Update local state
      setState(prev => ({ ...prev, currentTime: newTime }));
      console.log(`Timeline seeked to ${newTime}s`);
    } catch (error) {
      console.error('Failed to seek backward:', error);
      // Fallback to local state update
      setState(prev => ({ ...prev, currentTime: Math.max(0, prev.currentTime - 10) }));
    }
  }, [state.currentTime]);

  const handleSkipForward = useCallback(async () => {
    try {
      const newTime = Math.min(state.duration, state.currentTime + 10);
      
      // Call backend to seek to new time
      await invoke('timeline_seek', { time: newTime });
      
      // Update local state
      setState(prev => ({ ...prev, currentTime: newTime }));
      console.log(`Timeline seeked to ${newTime}s`);
    } catch (error) {
      console.error('Failed to seek forward:', error);
      // Fallback to local state update
      setState(prev => ({ ...prev, currentTime: Math.min(prev.duration, prev.currentTime + 10) }));
    }
  }, [state.currentTime, state.duration]);

  // Zoom controls
  const handleZoomIn = useCallback(() => {
    setState(prev => ({ ...prev, zoom: Math.min(10, prev.zoom * 1.2) }));
  }, []);

  const handleZoomOut = useCallback(() => {
    setState(prev => ({ ...prev, zoom: Math.max(0.1, prev.zoom / 1.2) }));
  }, []);

  const handleZoomReset = useCallback(() => {
    setState(prev => ({ ...prev, zoom: 1 }));
  }, []);

  // Clip manipulation
  const selectStoreClip = useAppStore((s) => s.selectTimelineClip);
  const deselectAllStoreClips = useAppStore((s) => s.deselectAllTimelineClips);

  const handleClipSelect = useCallback((clipId: string, multiSelect: boolean = false) => {
    setState(prev => ({
      ...prev,
      selectedClips: multiSelect
        ? prev.selectedClips.includes(clipId)
          ? prev.selectedClips.filter(id => id !== clipId)
          : [...prev.selectedClips, clipId]
        : [clipId]
    }));
    selectStoreClip(clipId, multiSelect);
  }, [selectStoreClip]);

  const handleClipMove = useCallback(async (clipId: string, newTime: number, newTrackId?: string) => {
    try {
      // Call backend to move clip
      await invoke('timeline_move_clip', { 
        clipId, 
        newTime, 
        newTrackId: newTrackId || null 
      });
      
      // Update local state
      setState(prev => ({
        ...prev,
        tracks: prev.tracks.map(track => ({
          ...track,
          clips: track.clips.map(clip => 
            clip.id === clipId 
              ? { ...clip, startTime: newTime, trackId: newTrackId || clip.trackId }
              : clip
          )
        }))
      }));
      
      console.log(`Clip ${clipId} moved to time ${newTime}`);
    } catch (error) {
      console.error('Failed to move clip:', error);
      // Fallback to local state update
      setState(prev => ({
        ...prev,
        tracks: prev.tracks.map(track => ({
          ...track,
          clips: track.clips.map(clip => 
            clip.id === clipId 
              ? { ...clip, startTime: newTime, trackId: newTrackId || clip.trackId }
              : clip
          )
        }))
      }));
    }
  }, []);

  const handleClipTrim = useCallback(async (clipId: string, edge: 'start' | 'end', newTime: number) => {
    try {
      // Call backend to trim clip
      await invoke('timeline_trim_clip', { clipId, edge, newTime });
      
      // Update local state
      setState(prev => ({
        ...prev,
        tracks: prev.tracks.map(track => ({
          ...track,
          clips: track.clips.map(clip => 
            clip.id === clipId 
              ? edge === 'start'
                ? { ...clip, startTime: newTime, duration: clip.duration - (newTime - clip.startTime) }
                : { ...clip, duration: newTime - clip.startTime }
              : clip
          )
        }))
      }));
      
      console.log(`Clip ${clipId} trimmed at ${edge} to ${newTime}`);
    } catch (error) {
      console.error('Failed to trim clip:', error);
      // Fallback to local state update
      setState(prev => ({
        ...prev,
        tracks: prev.tracks.map(track => ({
          ...track,
          clips: track.clips.map(clip => 
            clip.id === clipId 
              ? edge === 'start'
                ? { ...clip, startTime: newTime, duration: clip.duration - (newTime - clip.startTime) }
                : { ...clip, duration: newTime - clip.startTime }
              : clip
          )
        }))
      }));
    }
  }, []);

  const handleClipSplit = useCallback((clipId: string, splitTime: number) => {
    setState(prev => ({
      ...prev,
      tracks: prev.tracks.map(track => ({
        ...track,
        clips: track.clips.flatMap(clip => {
          if (clip.id === clipId && splitTime > clip.startTime && splitTime < clip.startTime + clip.duration) {
            const firstDuration = splitTime - clip.startTime;
            const secondDuration = clip.duration - firstDuration;
            
            return [
              { ...clip, id: `${clip.id}-1`, duration: firstDuration },
              { ...clip, id: `${clip.id}-2`, startTime: splitTime, duration: secondDuration }
            ];
          }
          return clip;
        })
      }))
    }));
  }, []);

  const handleClipDelete = useCallback((clipId: string) => {
    setState(prev => ({
      ...prev,
      tracks: prev.tracks.map(track => ({
        ...track,
        clips: track.clips.filter(clip => clip.id !== clipId)
      })),
      selectedClips: prev.selectedClips.filter(id => id !== clipId)
    }));
    deselectAllStoreClips();
  }, [deselectAllStoreClips]);

  // Time navigation
  const handleTimeChange = useCallback((newTime: number) => {
    setState(prev => ({ ...prev, currentTime: Math.max(0, Math.min(prev.duration, newTime)) }));
  }, []);

  // Scroll handling
  const handleScroll = useCallback((scrollX: number) => {
    setState(prev => ({ ...prev, scrollX: Math.max(0, scrollX) }));
  }, []);

  const contextValue = {
    state,
    isDragging,
    dragMode,
    isTrimming,
    trimStart,
    timelineRef,
    canvasRef,
    svgRef,
    setIsDragging,
    setDragMode,
    setIsTrimming,
    setTrimStart,
    handlePlay,
    handleStop,
    handleSkipBack,
    handleSkipForward,
    handleZoomIn,
    handleZoomOut,
    handleZoomReset,
    handleClipSelect,
    handleClipMove,
    handleClipTrim,
    handleClipSplit,
    handleClipDelete,
    handleTimeChange,
    handleScroll
  };

  return (
    <TimelineProvider value={contextValue}>
      <div className="flex flex-col h-full bg-[#141414]">
        <div className="flex items-center justify-between px-3 py-2 bg-[#1a1a1a] border-b border-[#222]">
          <div className="flex items-center space-x-2">
            <button
              onClick={handlePlay}
              className="p-1.5 bg-blue-600/90 hover:bg-blue-500 rounded-md text-white transition-colors"
            >
              {state.isPlaying ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4" />}
            </button>
            <button
              onClick={handleStop}
              className="p-1.5 bg-[#252525] hover:bg-[#333] rounded-md text-[#ccc] transition-colors"
            >
              <div className="w-4 h-4 bg-white rounded-sm" />
            </button>
            <button
              onClick={handleSkipBack}
              className="p-1.5 bg-[#252525] hover:bg-[#333] rounded-md text-[#ccc] transition-colors"
            >
              <SkipBack className="w-4 h-4" />
            </button>
            <button
              onClick={handleSkipForward}
              className="p-1.5 bg-[#252525] hover:bg-[#333] rounded-md text-[#ccc] transition-colors"
            >
              <SkipForward className="w-4 h-4" />
            </button>
            <button
              onClick={() => handleClipSplit(state.selectedClips[0], state.currentTime)}
              disabled={state.selectedClips.length !== 1}
              className="p-1.5 bg-[#252525] hover:bg-[#333] rounded-md text-[#ccc] transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            >
              <Scissors className="w-4 h-4" />
            </button>
          </div>
          
          <div className="flex items-center space-x-4">
            <span className="text-[#aaa] text-[12px] font-mono tracking-wide">
              {formatTime(state.currentTime)} / {formatTime(state.duration)}
            </span>
            <select
              value={state.playbackRate}
              onChange={(e) => setState(prev => ({ ...prev, playbackRate: parseFloat(e.target.value) }))}
              className="bg-[#1a1a1a] text-[#aaa] px-2 py-1 rounded-md text-[11px] border border-[#2a2a2a]"
            >
              <option value={0.25}>0.25x</option>
              <option value={0.5}>0.5x</option>
              <option value={1}>1x</option>
              <option value={1.5}>1.5x</option>
              <option value={2}>2x</option>
            </select>
          </div>
        </div>

        <TimelineRuler />

        <div className="flex-1 relative overflow-hidden">
          <TimelineCanvas />
        </div>

        <div className="absolute bottom-3 right-3 flex items-center space-x-1.5 bg-[#1a1a1a] rounded-md px-2 py-1 border border-[#222]">
          <button
            onClick={handleZoomOut}
            className="p-1 hover:bg-[#2a2a2a] rounded text-[#888] transition-colors"
          >
            <span className="text-xs">−</span>
          </button>
          <span className="text-[#888] text-[10px] px-1.5 min-w-[28px] text-center">
            {Math.round(state.zoom * 100)}%
          </span>
          <button
            onClick={handleZoomIn}
            className="p-1 hover:bg-[#2a2a2a] rounded text-[#888] transition-colors"
          >
            <span className="text-xs">+</span>
          </button>
          <button
            onClick={handleZoomReset}
            className="p-1 hover:bg-[#2a2a2a] rounded text-[#888] transition-colors ml-1.5"
          >
            <span className="text-xs">Reset</span>
          </button>
        </div>
      </div>
    </TimelineProvider>
  );
};


export default TimelineEditor;
