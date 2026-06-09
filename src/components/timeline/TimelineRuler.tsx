'use client';

import React, { useRef, useEffect, useState } from 'react';
import { useTimeline } from './TimelineContext';

export const TimelineRuler: React.FC = () => {
  const { state, handleTimeChange } = useTimeline();
  const rulerRef = useRef<HTMLDivElement>(null);
  const [rulerWidth, setRulerWidth] = useState(0);

  useEffect(() => {
    const updateWidth = () => {
      if (rulerRef.current) {
        setRulerWidth(rulerRef.current.offsetWidth);
      }
    };

    updateWidth();
    window.addEventListener('resize', updateWidth);
    return () => window.removeEventListener('resize', updateWidth);
  }, []);

  // Generate time markings
  const generateTimeMarkings = () => {
    const markings = [];
    const pixelsPerSecond = 50 * state.zoom;
    const totalWidth = state.duration * pixelsPerSecond;
    const interval = getTimeInterval(pixelsPerSecond);
    
    for (let time = 0; time <= state.duration; time += interval) {
      const position = time * pixelsPerSecond;
      const isMajor = time % (interval * 2) === 0;
      
      markings.push(
        <div
          key={time}
          className="absolute flex flex-col items-center"
          style={{ left: `${position}px` }}
        >
          <div
            className={`bg-gray-400 ${isMajor ? 'w-px h-4' : 'w-px h-2'}`}
          />
          {isMajor && (
            <span className="text-xs text-gray-500 mt-1 whitespace-nowrap">
              {formatTime(time)}
            </span>
          )}
        </div>
      );
    }
    
    return markings;
  };

  // Get appropriate time interval based on zoom
  const getTimeInterval = (pixelsPerSecond: number): number => {
    if (pixelsPerSecond < 10) return 60; // 1 minute
    if (pixelsPerSecond < 20) return 30; // 30 seconds
    if (pixelsPerSecond < 40) return 15; // 15 seconds
    if (pixelsPerSecond < 80) return 5; // 5 seconds
    if (pixelsPerSecond < 160) return 1; // 1 second
    return 0.5; // 0.5 seconds
  };

  // Format time display
  const formatTime = (seconds: number): string => {
    const minutes = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  // Handle ruler click for time navigation
  const handleRulerClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const rect = rulerRef.current?.getBoundingClientRect();
    if (rect) {
      const clickX = e.clientX - rect.left;
      const pixelsPerSecond = 50 * state.zoom;
      const newTime = Math.max(0, Math.min(state.duration, (clickX + state.scrollX) / pixelsPerSecond));
      handleTimeChange(newTime);
    }
  };

  return (
    <div className="relative h-12 bg-gray-800 border-b border-gray-700 overflow-hidden">
      <div
        ref={rulerRef}
        className="absolute inset-0 flex items-end cursor-pointer"
        onClick={handleRulerClick}
        style={{
          transform: `translateX(-${state.scrollX}px)`,
          width: `${state.duration * 50 * state.zoom}px`
        }}
      >
        {generateTimeMarkings()}
        
        {/* Playhead indicator */}
        <div
          className="absolute top-0 bottom-0 w-0.5 bg-red-500 pointer-events-none"
          style={{ left: `${state.currentTime * 50 * state.zoom}px` }}
        >
          <div className="absolute -top-1 -left-1 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-red-500" />
        </div>
      </div>
      
      {/* Current time display */}
      <div className="absolute top-2 left-2 bg-gray-900 px-2 py-1 rounded text-xs text-white font-mono">
        {formatTime(state.currentTime)}
      </div>
    </div>
  );
};
