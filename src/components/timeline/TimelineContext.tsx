'use client';

import React, { createContext, useContext } from 'react';

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

interface TimelineContextType {
  state: TimelineState;
  isDragging: boolean;
  dragMode: 'move' | 'trim' | 'ripple' | 'roll';
  isTrimming: boolean;
  trimStart: { clipId: string; edge: 'start' | 'end'; time: number } | null;
  timelineRef: React.RefObject<HTMLDivElement | null>;
  canvasRef: React.RefObject<HTMLDivElement | null>;
  svgRef: React.RefObject<SVGSVGElement | null>;
  setIsDragging: (dragging: boolean) => void;
  setDragMode: (mode: 'move' | 'trim' | 'ripple' | 'roll') => void;
  setIsTrimming: (trimming: boolean) => void;
  setTrimStart: (start: { clipId: string; edge: 'start' | 'end'; time: number } | null) => void;
  handlePlay: () => void;
  handleStop: () => void;
  handleSkipBack: () => void;
  handleSkipForward: () => void;
  handleZoomIn: () => void;
  handleZoomOut: () => void;
  handleZoomReset: () => void;
  handleClipSelect: (clipId: string, multiSelect?: boolean) => void;
  handleClipMove: (clipId: string, newTime: number, newTrackId?: string) => void;
  handleClipTrim: (clipId: string, edge: 'start' | 'end', newTime: number) => void;
  handleClipSplit: (clipId: string, splitTime: number) => void;
  handleClipDelete: (clipId: string) => void;
  handleTimeChange: (newTime: number) => void;
  handleScroll: (scrollX: number) => void;
}

const TimelineContext = createContext<TimelineContextType | undefined>(undefined);

export const useTimeline = () => {
  const context = useContext(TimelineContext);
  if (!context) {
    throw new Error('useTimeline must be used within a TimelineProvider');
  }
  return context;
};

interface TimelineProviderProps {
  children: React.ReactNode;
  value: TimelineContextType;
}

export const TimelineProvider: React.FC<TimelineProviderProps> = ({ children, value }) => {
  return (
    <TimelineContext.Provider value={value}>
      {children}
    </TimelineContext.Provider>
  );
};

export type {
  TimelineClip,
  TimelineTrack,
  TimelineState,
  TimelineContextType
};
