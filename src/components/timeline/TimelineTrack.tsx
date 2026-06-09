'use client';

import React from 'react';
import { useTimeline } from './TimelineContext';
import { TimelineClip } from './TimelineClip';

interface TimelineTrackProps {
  track: {
    id: string;
    name: string;
    type: 'video' | 'audio';
    height: number;
    muted: boolean;
    solo: boolean;
    locked: boolean;
    clips: any[];
  };
  top: number;
  pixelsPerSecond: number;
  hoveredClip: string | null;
  selectedClips: string[];
  onClipSelect: (clipId: string, multiSelect?: boolean) => void;
  onClipTrim: (clipId: string, edge: 'start' | 'end', newTime: number) => void;
}

export const TimelineTrack: React.FC<TimelineTrackProps> = ({
  track,
  top,
  pixelsPerSecond,
  hoveredClip,
  selectedClips,
  onClipSelect,
  onClipTrim
}) => {
  const { state } = useTimeline();

  const getTrackColor = () => {
    if (track.type === 'video') {
      return track.locked ? '#374151' : '#1f2937';
    } else {
      return track.locked ? '#451a03' : '#451a03';
    }
  };

  const getTrackBorderColor = () => {
    if (track.type === 'video') {
      return track.locked ? '#4b5563' : '#374151';
    } else {
      return track.locked ? '#78350f' : '#78350f';
    }
  };

  return (
    <div
      className={`absolute border-b ${track.type === 'video' ? 'border-gray-700' : 'border-yellow-900'}`}
      style={{
        top: `${top}px`,
        left: 0,
        right: 0,
        height: `${track.height}px`,
        backgroundColor: getTrackColor(),
        borderColor: getTrackBorderColor()
      }}
    >
      {/* Track header */}
      <div
        className="absolute left-0 top-0 bottom-0 w-32 flex items-center justify-between px-2 text-xs text-white border-r"
        style={{
          backgroundColor: track.type === 'video' ? '#111827' : '#451a03',
          borderColor: track.type === 'video' ? '#374151' : '#78350f',
          zIndex: 10
        }}
      >
        <span className="truncate">{track.name}</span>
        <div className="flex items-center space-x-1">
          {track.muted && (
            <span className="text-red-400">M</span>
          )}
          {track.solo && (
            <span className="text-yellow-400">S</span>
          )}
          {track.locked && (
            <span className="text-gray-400">L</span>
          )}
        </div>
      </div>

      {/* Track content area */}
      <div
        className="absolute inset-0"
        style={{ marginLeft: '128px' }}
      >
        {/* Render clips */}
        {track.clips.map((clip) => (
          <TimelineClip
            key={clip.id}
            clip={clip}
            pixelsPerSecond={pixelsPerSecond}
            isHovered={hoveredClip === clip.id}
            isSelected={selectedClips.includes(clip.id)}
            isLocked={track.locked}
            onSelect={onClipSelect}
            onTrim={onClipTrim}
          />
        ))}
        
        {/* Track type indicator */}
        <div className="absolute top-1 right-1 text-xs opacity-50">
          {track.type === 'video' ? '🎬' : '🎵'}
        </div>
      </div>
    </div>
  );
};
