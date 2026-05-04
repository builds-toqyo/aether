'use client';

import React, { useState, useRef, useCallback, useEffect } from 'react';
import { PreviewCanvas } from './PreviewCanvas';
import { PreviewControls } from './PreviewControls';
import { PreviewSettings } from './PreviewSettings';
import { PreviewProvider, usePreview } from './PreviewContext';
import { Play, Pause, SkipBack, SkipForward, Volume2, Maximize2, Settings } from 'lucide-react';

// Types
interface PreviewFrame {
  id: string;
  timestamp: number;
  width: number;
  height: number;
  data: ImageData | null;
  url: string | null;
}

interface PreviewState {
  isPlaying: boolean;
  currentTime: number;
  duration: number;
  volume: number;
  isMuted: boolean;
  playbackRate: number;
  quality: 'low' | 'medium' | 'high' | 'auto';
  zoom: number;
  panX: number;
  panY: number;
  isFullscreen: boolean;
  showSettings: boolean;
  currentFrame: PreviewFrame | null;
  isLoading: boolean;
}

export const PreviewWindow: React.FC = () => {
  const [state, setState] = useState<PreviewState>({
    isPlaying: false,
    currentTime: 0,
    duration: 300, // 5 minutes in seconds
    volume: 1,
    isMuted: false,
    playbackRate: 1,
    quality: 'auto',
    zoom: 1,
    panX: 0,
    panY: 0,
    isFullscreen: false,
    showSettings: false,
    currentFrame: null,
    isLoading: false
  });

  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const [showControls, setShowControls] = useState(true);
  const [controlsTimeout, setControlsTimeout] = useState<NodeJS.Timeout | null>(null);

  const previewRef = useRef<HTMLDivElement>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  // Auto-hide controls
  useEffect(() => {
    if (state.isPlaying && !state.showSettings) {
      const timeout = setTimeout(() => setShowControls(false), 3000);
      setControlsTimeout(timeout);
      return () => {
        if (timeout) clearTimeout(timeout);
      };
    }
  }, [state.isPlaying, state.currentTime, state.showSettings]);

  // Show controls on mouse movement
  const handleMouseMove = useCallback(() => {
    setShowControls(true);
    if (controlsTimeout) {
      clearTimeout(controlsTimeout);
    }
    if (state.isPlaying && !state.showSettings) {
      const timeout = setTimeout(() => setShowControls(false), 3000);
      setControlsTimeout(timeout);
    }
  }, [state.isPlaying, state.showSettings, controlsTimeout]);

  // Playback controls
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

  const handleTimeChange = useCallback((newTime: number) => {
    setState(prev => ({ ...prev, currentTime: Math.max(0, Math.min(prev.duration, newTime)) }));
  }, []);

  // Volume controls
  const handleVolumeChange = useCallback((volume: number) => {
    setState(prev => ({ ...prev, volume: Math.max(0, Math.min(1, volume)), isMuted: volume === 0 }));
  }, []);

  const handleMuteToggle = useCallback(() => {
    setState(prev => ({ ...prev, isMuted: !prev.isMuted }));
  }, []);

  // Zoom and pan controls
  const handleZoomIn = useCallback(() => {
    setState(prev => ({ ...prev, zoom: Math.min(5, prev.zoom * 1.2) }));
  }, []);

  const handleZoomOut = useCallback(() => {
    setState(prev => ({ ...prev, zoom: Math.max(0.1, prev.zoom / 1.2) }));
  }, []);

  const handleZoomReset = useCallback(() => {
    setState(prev => ({ ...prev, zoom: 1, panX: 0, panY: 0 }));
  }, []);

  const handlePan = useCallback((deltaX: number, deltaY: number) => {
    setState(prev => ({
      ...prev,
      panX: prev.panX + deltaX,
      panY: prev.panY + deltaY
    }));
  }, []);

  // Mouse wheel zoom
  const handleWheel = useCallback((e: React.WheelEvent) => {
    if (e.ctrlKey) {
      e.preventDefault();
      const scaleFactor = e.deltaY > 0 ? 0.9 : 1.1;
      const newZoom = Math.max(0.1, Math.min(5, state.zoom * scaleFactor));
      setState(prev => ({ ...prev, zoom: newZoom }));
    }
  }, [state.zoom]);

  // Mouse drag for panning
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button === 0 && e.shiftKey) {
      setIsDragging(true);
      setDragStart({ x: e.clientX, y: e.clientY });
      e.preventDefault();
    }
  }, []);

  const handleMouseMoveDrag = useCallback((e: React.MouseEvent) => {
    if (isDragging) {
      const deltaX = (e.clientX - dragStart.x) / state.zoom;
      const deltaY = (e.clientY - dragStart.y) / state.zoom;
      handlePan(deltaX, deltaY);
      setDragStart({ x: e.clientX, y: e.clientY });
    }
  }, [isDragging, dragStart, state.zoom, handlePan]);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  // Fullscreen toggle
  const handleFullscreenToggle = useCallback(() => {
    if (!document.fullscreenElement) {
      previewRef.current?.requestFullscreen();
      setState(prev => ({ ...prev, isFullscreen: true }));
    } else {
      document.exitFullscreen();
      setState(prev => ({ ...prev, isFullscreen: false }));
    }
  }, []);

  // Settings toggle
  const handleSettingsToggle = useCallback(() => {
    setState(prev => ({ ...prev, showSettings: !prev.showSettings }));
  }, []);

  // Quality change
  const handleQualityChange = useCallback((quality: 'low' | 'medium' | 'high' | 'auto') => {
    setState(prev => ({ ...prev, quality }));
  }, []);

  // Playback rate change
  const handlePlaybackRateChange = useCallback((rate: number) => {
    setState(prev => ({ ...prev, playbackRate: rate }));
  }, []);

  // Frame loading simulation
  const loadFrame = useCallback(async (timestamp: number) => {
    setState(prev => ({ ...prev, isLoading: true }));
    
    // Simulate frame loading
    await new Promise(resolve => setTimeout(resolve, 50));
    
    const mockFrame: PreviewFrame = {
      id: `frame-${timestamp}`,
      timestamp,
      width: 1920,
      height: 1080,
      data: null,
      url: `https://picsum.photos/seed/${timestamp}/1920/1080.jpg`
    };
    
    setState(prev => ({ ...prev, currentFrame: mockFrame, isLoading: false }));
  }, []);

  // Load frame when time changes
  useEffect(() => {
    loadFrame(state.currentTime);
  }, [state.currentTime, loadFrame]);

  // Playback simulation
  useEffect(() => {
    let interval: NodeJS.Timeout;
    
    if (state.isPlaying) {
      interval = setInterval(() => {
        setState(prev => {
          const newTime = prev.currentTime + (0.1 * prev.playbackRate);
          if (newTime >= prev.duration) {
            return { ...prev, isPlaying: false, currentTime: prev.duration };
          }
          return { ...prev, currentTime: newTime };
        });
      }, 100);
    }
    
    return () => {
      if (interval) clearInterval(interval);
    };
  }, [state.isPlaying, state.playbackRate, state.duration]);

  const contextValue = {
    state,
    previewRef,
    videoRef,
    canvasRef,
    handlePlay,
    handleStop,
    handleSkipBack,
    handleSkipForward,
    handleTimeChange,
    handleVolumeChange,
    handleMuteToggle,
    handleZoomIn,
    handleZoomOut,
    handleZoomReset,
    handlePan,
    handleWheel,
    handleMouseDown,
    handleMouseMove,
    handleMouseMoveDrag,
    handleMouseUp,
    handleFullscreenToggle,
    handleSettingsToggle,
    handleQualityChange,
    handlePlaybackRateChange,
    loadFrame
  };

  return (
    <PreviewProvider value={contextValue}>
      <div
        ref={previewRef}
        className={`relative w-full h-full bg-black overflow-hidden ${state.isFullscreen ? 'fixed inset-0 z-50' : ''}`}
        onMouseMove={(e) => {
          if (handleMouseMove) handleMouseMove();
          if (handleMouseMoveDrag) handleMouseMoveDrag(e);
        }}
        onWheel={handleWheel}
        onMouseDown={handleMouseDown}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      >
        <PreviewCanvas />

        {state.isLoading && (
          <div className="absolute inset-0 flex items-center justify-center bg-black bg-opacity-50">
            <div className="text-white text-lg">Loading frame...</div>
          </div>
        )}

        <div className={`absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black to-transparent transition-opacity duration-300 ${
          showControls || state.showSettings ? 'opacity-100' : 'opacity-0'
        }`}>
          <PreviewControls />

          {state.showSettings && (
            <PreviewSettings />
          )}
        </div>

        <div className={`absolute top-0 left-0 right-0 bg-gradient-to-b from-black to-transparent p-4 transition-opacity duration-300 ${
          showControls || state.showSettings ? 'opacity-100' : 'opacity-0'
        }`}>
          <div className="flex items-center justify-between">
            <div className="text-white text-sm font-mono">
              {formatTime(state.currentTime)} / {formatTime(state.duration)}
            </div>
          <div className="flex items-center space-x-4">
              <div className="text-white text-sm">
                {state.quality.toUpperCase()}
              </div>

              <div className="text-white text-sm">
                {Math.round(state.zoom * 100)}%
              </div>

              <button
                onClick={handleSettingsToggle}
                className="p-2 bg-gray-800 hover:bg-gray-700 rounded text-white transition-colors"
              >
                <Settings className="w-4 h-4" />
              </button>

              <button
                onClick={handleFullscreenToggle}
                className="p-2 bg-gray-800 hover:bg-gray-700 rounded text-white transition-colors"
              >
                <Maximize2 className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>

        {!showControls && !state.showSettings && (
          <div className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 text-white text-sm opacity-50 pointer-events-none">
            Move mouse to show controls
          </div>
        )}
      </div>
    </PreviewProvider>
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

export default PreviewWindow;
