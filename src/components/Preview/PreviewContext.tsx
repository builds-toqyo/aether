'use client';

import React, { createContext, useContext } from 'react';

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

interface PreviewContextType {
  state: PreviewState;
  previewRef: React.RefObject<HTMLDivElement | null>;
  videoRef: React.RefObject<HTMLVideoElement | null>;
  canvasRef: React.RefObject<HTMLCanvasElement | null>;
  handlePlay: () => void;
  handleStop: () => void;
  handleSkipBack: () => void;
  handleSkipForward: () => void;
  handleTimeChange: (newTime: number) => void;
  handleVolumeChange: (volume: number) => void;
  handleMuteToggle: () => void;
  handleZoomIn: () => void;
  handleZoomOut: () => void;
  handleZoomReset: () => void;
  handlePan: (deltaX: number, deltaY: number) => void;
  handleWheel: (e: React.WheelEvent) => void;
  handleMouseDown: (e: React.MouseEvent) => void;
  handleMouseMove: (e: React.MouseEvent) => void;
  handleMouseMoveDrag: (e: React.MouseEvent) => void;
  handleMouseUp: () => void;
  handleFullscreenToggle: () => void;
  handleSettingsToggle: () => void;
  handleQualityChange: (quality: 'low' | 'medium' | 'high' | 'auto') => void;
  handlePlaybackRateChange: (rate: number) => void;
  loadFrame: (timestamp: number) => Promise<void>;
}

const PreviewContext = createContext<PreviewContextType | undefined>(undefined);

export const usePreview = () => {
  const context = useContext(PreviewContext);
  if (!context) {
    throw new Error('usePreview must be used within a PreviewProvider');
  }
  return context;
};

interface PreviewProviderProps {
  children: React.ReactNode;
  value: PreviewContextType;
}

export const PreviewProvider: React.FC<PreviewProviderProps> = ({ children, value }) => {
  return (
    <PreviewContext.Provider value={value}>
      {children}
    </PreviewContext.Provider>
  );
};

export type {
  PreviewFrame,
  PreviewState,
  PreviewContextType
};
