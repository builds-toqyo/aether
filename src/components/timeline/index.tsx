'use client';

import React, { useState, useRef, useCallback, useEffect } from 'react';
import { TimelineCanvas } from './TimelineCanvas';
import { TimelineRuler } from './TimelineRuler';
import { TimelineProvider, useTimeline } from './TimelineContext';
import { Play, Pause, SkipBack, SkipForward, Scissors } from 'lucide-react';

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
    const localTracks: TimelineTrack[] = [];
    setState(prev => ({ ...prev, tracks: localTracks }));
  }, []);

  const handlePlay = useCallback(() => {
    setState(prev => ({ ...prev, isPlaying: !prev.isPlaying }));
  }, []);

  const handleStop = useCallback(() => {
    setState(prev => ({ ...prev, isPlaying: false, currentTime: 0 }));
  }, []);

  const handleSkipBack = useCallback(() => {
    setState(prev => ({ ...prev, currentTime: Math.max(0, prev.currentTime - 10) }));
  }, []);

  const handleSkipForward = useCallback(() => {
    setState(prev => ({ ...prev, currentTime: Math.min(prev.duration, prev.currentTime + 10) }));
  }, []);

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
  const handleClipSelect = useCallback((clipId: string, multiSelect: boolean = false) => {
    setState(prev => ({
      ...prev,
      selectedClips: multiSelect 
        ? prev.selectedClips.includes(clipId)
          ? prev.selectedClips.filter(id => id !== clipId)
          : [...prev.selectedClips, clipId]
        : [clipId]
    }));
  }, []);

  const handleClipMove = useCallback((clipId: string, newTime: number, newTrackId?: string) => {
    setState(prev => ({
      ...prev,
      tracks: prev.tracks.map(track => ({
        ...track,
        clips: track.clips.map(clip => {
          if (clip.id === clipId) {
            const updatedClip = { ...clip, startTime: newTime };
            if (newTrackId) {
              updatedClip.trackId = newTrackId;
            }
            return updatedClip;
          }
          return clip;
        })
      }))
    }));
  }, []);

  const handleClipTrim = useCallback((clipId: string, edge: 'start' | 'end', newTime: number) => {
    setState(prev => ({
      ...prev,
      tracks: prev.tracks.map(track => ({
        ...track,
        clips: track.clips.map(clip => {
          if (clip.id === clipId) {
            if (edge === 'start') {
              const duration = clip.duration - (newTime - clip.startTime);
              return { ...clip, startTime: newTime, duration: Math.max(1, duration) };
            } else {
              const duration = newTime - clip.startTime;
              return { ...clip, duration: Math.max(1, duration) };
            }
          }
          return clip;
        })
      }))
    }));
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
  }, []);

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
      <div className="flex flex-col h-full bg-gray-900">
        <div className="flex items-center justify-between px-4 py-2 bg-gray-800 border-b border-gray-700">
          <div className="flex items-center space-x-2">
            <button
              onClick={handlePlay}
              className="p-2 bg-blue-600 hover:bg-blue-700 rounded text-white transition-colors"
            >
              {state.isPlaying ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4" />}
            </button>
            <button
              onClick={handleStop}
              className="p-2 bg-gray-700 hover:bg-gray-600 rounded text-white transition-colors"
            >
              <div className="w-4 h-4 bg-white rounded-sm" />
            </button>
            <button
              onClick={handleSkipBack}
              className="p-2 bg-gray-700 hover:bg-gray-600 rounded text-white transition-colors"
            >
              <SkipBack className="w-4 h-4" />
            </button>
            <button
              onClick={handleSkipForward}
              className="p-2 bg-gray-700 hover:bg-gray-600 rounded text-white transition-colors"
            >
              <SkipForward className="w-4 h-4" />
            </button>
            <button
              onClick={() => handleClipSplit(state.selectedClips[0], state.currentTime)}
              disabled={state.selectedClips.length !== 1}
              className="p-2 bg-gray-700 hover:bg-gray-600 rounded text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Scissors className="w-4 h-4" />
            </button>
          </div>
          
          <div className="flex items-center space-x-4">
            <span className="text-white text-sm font-mono">
              {formatTime(state.currentTime)} / {formatTime(state.duration)}
            </span>
            <select
              value={state.playbackRate}
              onChange={(e) => setState(prev => ({ ...prev, playbackRate: parseFloat(e.target.value) }))}
              className="bg-gray-700 text-white px-2 py-1 rounded text-sm"
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

        <div className="absolute bottom-4 right-4 flex items-center space-x-2 bg-gray-800 rounded-lg px-2 py-1">
          <button
            onClick={handleZoomOut}
            className="p-1 hover:bg-gray-700 rounded text-white transition-colors"
          >
            <span className="text-xs">−</span>
          </button>
          <span className="text-white text-xs px-2">
            {Math.round(state.zoom * 100)}%
          </span>
          <button
            onClick={handleZoomIn}
            className="p-1 hover:bg-gray-700 rounded text-white transition-colors"
          >
            <span className="text-xs">+</span>
          </button>
          <button
            onClick={handleZoomReset}
            className="p-1 hover:bg-gray-700 rounded text-white transition-colors ml-2"
          >
            <span className="text-xs">Reset</span>
          </button>
        </div>
      </div>
    </TimelineProvider>
  );
};

// Helper function to format time
function formatTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);
  const frames = Math.floor((seconds % 1) * 30); // Assuming 30 fps
  
  if (hours > 0) {
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}:${frames.toString().padStart(2, '0')}`;
  }
  return `${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}:${frames.toString().padStart(2, '0')}`;
}

export default TimelineEditor;
