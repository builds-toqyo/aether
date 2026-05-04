'use client';

import React, { useState, useRef } from 'react';
import { usePreview } from './PreviewContext';
import { Play, Pause, SkipBack, SkipForward, Volume2, VolumeX, Maximize2, Settings } from 'lucide-react';

export const PreviewControls: React.FC = () => {
  const { 
    state, 
    handlePlay, 
    handleStop, 
    handleSkipBack, 
    handleSkipForward, 
    handleTimeChange,
    handleVolumeChange,
    handleMuteToggle,
    handleFullscreenToggle,
    handleSettingsToggle
  } = usePreview();

  const [isDragging, setIsDragging] = useState(false);
  const progressBarRef = useRef<HTMLDivElement>(null);

  // Handle progress bar click
  const handleProgressClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const rect = progressBarRef.current?.getBoundingClientRect();
    if (rect && state.duration > 0) {
      const clickX = e.clientX - rect.left;
      const percentage = clickX / rect.width;
      const newTime = percentage * state.duration;
      handleTimeChange(newTime);
    }
  };

  // Handle progress bar drag
  const handleProgressMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    setIsDragging(true);
    handleProgressClick(e);
  };

  const handleProgressMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    if (isDragging) {
      handleProgressClick(e);
    }
  };

  const handleProgressMouseUp = () => {
    setIsDragging(false);
  };

  // Handle volume change
  const handleVolumeSliderChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const volume = parseFloat(e.target.value);
    handleVolumeChange(volume);
  };

  // Format time display
  const formatTime = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);
    const frames = Math.floor((seconds % 1) * 30);
    
    if (hours > 0) {
      return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}:${frames.toString().padStart(2, '0')}`;
    }
    return `${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}:${frames.toString().padStart(2, '0')}`;
  };

  const progressPercentage = state.duration > 0 ? (state.currentTime / state.duration) * 100 : 0;

  return (
    <div className="bg-black bg-opacity-75 backdrop-blur-sm p-4">
      <div className="flex items-center space-x-4">
        <div className="flex items-center space-x-2">
          <button
            onClick={handleSkipBack}
            className="p-2 bg-gray-800 hover:bg-gray-700 rounded text-white transition-colors"
            title="Skip back 10 seconds"
          >
            <SkipBack className="w-4 h-4" />
          </button>
          
          <button
            onClick={handlePlay}
            className="p-3 bg-blue-600 hover:bg-blue-700 rounded text-white transition-colors"
            title={state.isPlaying ? 'Pause' : 'Play'}
          >
            {state.isPlaying ? <Pause className="w-5 h-5" /> : <Play className="w-5 h-5" />}
          </button>
          
          <button
            onClick={handleSkipForward}
            className="p-2 bg-gray-800 hover:bg-gray-700 rounded text-white transition-colors"
            title="Skip forward 10 seconds"
          >
            <SkipForward className="w-4 h-4" />
          </button>
        </div>

        <div className="flex-1 flex items-center space-x-3">
          <span className="text-white text-sm font-mono min-w-[80px] text-right">
            {formatTime(state.currentTime)}
          </span>
          
          <div
            ref={progressBarRef}
            className="flex-1 h-2 bg-gray-700 rounded-full cursor-pointer relative"
            onClick={handleProgressClick}
            onMouseDown={handleProgressMouseDown}
            onMouseMove={handleProgressMouseMove}
            onMouseUp={handleProgressMouseUp}
            onMouseLeave={handleProgressMouseUp}
          >
            <div
              className="absolute top-0 left-0 h-full bg-blue-600 rounded-full transition-all duration-100"
              style={{ width: `${progressPercentage}%` }}
            />
            
            <div
              className="absolute top-1/2 transform -translate-y-1/2 w-4 h-4 bg-white rounded-full shadow-lg transition-all duration-100"
              style={{ left: `${progressPercentage}%`, marginLeft: '-8px' }}
            />
          </div>
          
          <span className="text-white text-sm font-mono min-w-[80px]">
            {formatTime(state.duration)}
          </span>
        </div>

        <div className="flex items-center space-x-2">
          <button
            onClick={handleMuteToggle}
            className="p-2 bg-gray-800 hover:bg-gray-700 rounded text-white transition-colors"
            title={state.isMuted ? 'Unmute' : 'Mute'}
          >
            {state.isMuted ? <VolumeX className="w-4 h-4" /> : <Volume2 className="w-4 h-4" />}
          </button>
          
          <div className="relative w-24">
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={state.isMuted ? 0 : state.volume}
              onChange={handleVolumeSliderChange}
              className="w-full h-1 bg-gray-700 rounded-full appearance-none cursor-pointer"
              style={{
                background: `linear-gradient(to right, #3B82F6 0%, #3B82F6 ${state.isMuted ? 0 : state.volume * 100}%, #374151 ${state.isMuted ? 0 : state.volume * 100}%, #374151 100%)`
              }}
            />
          </div>
        </div>

        <select
          value={state.playbackRate}
          onChange={(e) => {
            const rate = parseFloat(e.target.value);
            console.log('Playback rate changed to:', rate);
          }}
          className="bg-gray-800 text-white px-2 py-1 rounded text-sm border border-gray-700"
        >
          <option value={0.25}>0.25x</option>
          <option value={0.5}>0.5x</option>
          <option value={1}>1x</option>
          <option value={1.5}>1.5x</option>
          <option value={2}>2x</option>
        </select>

        <div className="flex items-center space-x-2">
          <button
            onClick={handleSettingsToggle}
            className="p-2 bg-gray-800 hover:bg-gray-700 rounded text-white transition-colors"
            title="Settings"
          >
            <Settings className="w-4 h-4" />
          </button>
          
          <button
            onClick={handleFullscreenToggle}
            className="p-2 bg-gray-800 hover:bg-gray-700 rounded text-white transition-colors"
            title="Fullscreen"
          >
            <Maximize2 className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
};
