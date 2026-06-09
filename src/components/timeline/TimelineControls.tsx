'use client';

import React from 'react';
import { useTimeline } from './TimelineContext';
import { Volume2, VolumeX, Headphones, Lock } from 'lucide-react';

export const TimelineControls: React.FC = () => {
  const { state } = useTimeline();

  return (
    <div className="flex items-center justify-between px-4 py-2 bg-gray-800 border-t border-gray-700">
      <div className="flex items-center space-x-4">
        {state.tracks.map((track) => (
          <div key={track.id} className="flex items-center space-x-2">
            <span className="text-white text-sm">{track.name}</span>
            
            <button
              className={`p-1 rounded ${track.muted ? 'bg-red-600' : 'bg-gray-700'} hover:bg-gray-600 transition-colors`}
            >
              {track.muted ? <VolumeX className="w-3 h-3 text-white" /> : <Volume2 className="w-3 h-3 text-white" />}
            </button>
            
            <button
              className={`p-1 rounded ${track.solo ? 'bg-yellow-600' : 'bg-gray-700'} hover:bg-gray-600 transition-colors`}
            >
              <span className="text-xs text-white font-bold">S</span>
            </button>
            
            <button
              className={`p-1 rounded ${track.locked ? 'bg-orange-600' : 'bg-gray-700'} hover:bg-gray-600 transition-colors`}
            >
              <Lock className="w-3 h-3 text-white" />
            </button>
            
            <div className="flex items-center space-x-1">
              <div
                className="w-8 h-2 rounded"
                style={{
                  backgroundColor: track.type === 'video' ? '#3B82F6' : '#F59E0B'
                }}
              />
              <span className="text-xs text-gray-400">{track.height}px</span>
            </div>
          </div>
        ))}
      </div>

      <div className="flex items-center space-x-4 text-sm text-gray-400">
        <span>Duration: {formatDuration(state.duration)}</span>
        <span>Zoom: {Math.round(state.zoom * 100)}%</span>
        <span>Tracks: {state.tracks.length}</span>
        <span>Clips: {state.tracks.reduce((total, track) => total + track.clips.length, 0)}</span>
      </div>
    </div>
  );
};

function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);
  
  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }
  return `${minutes}:${secs.toString().padStart(2, '0')}`;
}
