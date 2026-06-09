'use client';

import React, { useState, useRef } from 'react';
import { useTimeline } from './TimelineContext';
import { Video, Music, Image } from 'lucide-react';

interface TimelineClipProps {
  clip: {
    id: string;
    name: string;
    trackId: string;
    startTime: number;
    duration: number;
    type: 'video' | 'audio' | 'image';
    source: string;
    color: string;
  };
  pixelsPerSecond: number;
  isHovered: boolean;
  isSelected: boolean;
  isLocked: boolean;
  onSelect: (clipId: string, multiSelect?: boolean) => void;
  onTrim: (clipId: string, edge: 'start' | 'end', newTime: number) => void;
}

export const TimelineClip: React.FC<TimelineClipProps> = ({
  clip,
  pixelsPerSecond,
  isHovered,
  isSelected,
  isLocked,
  onSelect,
  onTrim
}) => {
  const { state, isTrimming, trimStart, setIsTrimming, setTrimStart } = useTimeline();
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, time: 0 });
  const clipRef = useRef<HTMLDivElement>(null);

  const clipLeft = clip.startTime * pixelsPerSecond;
  const clipWidth = clip.duration * pixelsPerSecond;

  // Handle clip selection
  const handleClipMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    e.stopPropagation();
    
    if (isLocked) return;
    
    const rect = clipRef.current?.getBoundingClientRect();
    if (rect) {
      const clickX = e.clientX - rect.left;
      const clickTime = clip.startTime + (clickX / pixelsPerSecond);
      
      // Check if clicking on trim handles
      const trimHandleWidth = 8;
      const isStartHandle = clickX <= trimHandleWidth;
      const isEndHandle = clickX >= clipWidth - trimHandleWidth;
      
      if (isStartHandle || isEndHandle) {
        // Start trimming
        setIsTrimming(true);
        setTrimStart({
          clipId: clip.id,
          edge: isStartHandle ? 'start' : 'end',
          time: clickTime
        });
      } else {
        // Start dragging
        setIsDragging(true);
        setDragStart({ x: e.clientX, time: clickTime });
        onSelect(clip.id, e.shiftKey);
      }
    }
  };

  // Handle mouse move for trimming
  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    if (isTrimming && trimStart && trimStart.clipId === clip.id) {
      const rect = clipRef.current?.getBoundingClientRect();
      if (rect) {
        const mouseX = e.clientX - rect.left;
        const newTime = clip.startTime + (mouseX / pixelsPerSecond);
        
        // Constrain to reasonable limits
        const constrainedTime = Math.max(0, Math.min(state.duration, newTime));
        
        if (trimStart.edge === 'start') {
          onTrim(clip.id, 'start', constrainedTime);
        } else {
          onTrim(clip.id, 'end', constrainedTime);
        }
      }
    }
  };

  // Handle mouse up
  const handleMouseUp = () => {
    if (isDragging) {
      setIsDragging(false);
    }
    if (isTrimming) {
      setIsTrimming(false);
      setTrimStart(null);
    }
  };

  // Get clip opacity based on state
  const getClipOpacity = () => {
    if (isLocked) return 0.5;
    if (isDragging) return 0.8;
    if (isHovered) return 1;
    if (isSelected) return 0.9;
    return 0.8;
  };

  // Get clip border style
  const getClipBorderStyle = () => {
    if (isSelected) {
      return '2px solid #3B82F6';
    }
    if (isHovered) {
      return '1px solid #60A5FA';
    }
    return '1px solid transparent';
  };

  return (
    <div
      ref={clipRef}
      className={`absolute top-1 bottom-1 rounded cursor-move transition-all ${
        isLocked ? 'cursor-not-allowed' : ''
      }`}
      style={{
        left: `${clipLeft}px`,
        width: `${clipWidth}px`,
        backgroundColor: clip.color,
        opacity: getClipOpacity(),
        border: getClipBorderStyle(),
        boxShadow: isSelected ? '0 0 10px rgba(59, 130, 246, 0.3)' : 'none'
      }}
      onMouseDown={handleClipMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
    >
      {!isLocked && (
        <>
          <div
            className="absolute left-0 top-0 bottom-0 w-2 bg-white opacity-50 cursor-ew-resize hover:opacity-75"
            style={{ width: '8px' }}
          />
          <div
            className="absolute right-0 top-0 bottom-0 w-2 bg-white opacity-50 cursor-ew-resize hover:opacity-75"
            style={{ width: '8px' }}
          />
        </>
      )}

      <div className="absolute inset-0 flex items-center justify-between px-2 pointer-events-none">
        <div className="text-white opacity-75">
          {clip.type === 'video' && <Video className="w-3 h-3" />}
          {clip.type === 'audio' && <Music className="w-3 h-3" />}
          {clip.type === 'image' && <Image className="w-3 h-3" />}
        </div>

        <div className="text-white text-xs truncate flex-1 mx-2">
          {clip.name}
        </div>

        <div className="text-white text-xs opacity-75 whitespace-nowrap">
          {formatDuration(clip.duration)}
        </div>
      </div>

      {clip.type === 'audio' && (
        <div className="absolute inset-0 opacity-30">
          <svg className="w-full h-full">
            <path
              d="M 0 20 Q 25 10 50 20 T 100 20 T 150 20 T 200 20"
              stroke="white"
              strokeWidth="1"
              fill="none"
            />
            <path
              d="M 0 20 Q 25 30 50 20 T 100 20 T 150 20 T 200 20"
              stroke="white"
              strokeWidth="1"
              fill="none"
              transform="scale(1, -1) translate(0, -40)"
            />
          </svg>
        </div>
      )}

      {(clip.type === 'video' || clip.type === 'image') && (
        <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white to-transparent opacity-10" />
      )}
    </div>
  );
};

// Helper function to format duration
function formatDuration(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${minutes}:${secs.toString().padStart(2, '0')}`;
}
