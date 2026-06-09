'use client';

import React from 'react';
import { useTimeline } from './TimelineContext';

export const TimelinePlayhead: React.FC = () => {
  const { state } = useTimeline();

  return (
    <div
      className="absolute top-0 bottom-0 w-0.5 bg-red-500 pointer-events-none z-20"
      style={{ left: `${state.currentTime * 50 * state.zoom}px` }}
    >
      <div className="absolute -top-1 -left-1 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-red-500" />
      
      <div className="absolute top-0 bottom-0 w-0.5 bg-red-500" />
      
      <div className="absolute -bottom-1 -left-1 w-0 h-0 border-l-4 border-r-4 border-b-4 border-transparent border-b-red-500" />
      
      <div className="absolute -top-6 left-2 bg-red-500 text-white text-xs px-1 py-0.5 rounded whitespace-nowrap">
        {formatTime(state.currentTime)}
      </div>
    </div>
  );
};

function formatTime(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  const frames = Math.floor((seconds % 1) * 30);
  
  return `${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}:${frames.toString().padStart(2, '0')}`;
}
