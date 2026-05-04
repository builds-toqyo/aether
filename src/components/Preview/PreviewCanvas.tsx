'use client';

import React, { useRef, useEffect, useState } from 'react';
import { usePreview } from './PreviewContext';

export const PreviewCanvas: React.FC = () => {
  const { state, canvasRef, handleWheel } = usePreview();
  const [imageLoaded, setImageLoaded] = useState(false);
  const [imageError, setImageError] = useState(false);

  // Handle image load
  useEffect(() => {
    if (state.currentFrame?.url) {
      setImageLoaded(false);
      setImageError(false);
    }
  }, [state.currentFrame?.url]);

  const handleImageLoad = () => {
    setImageLoaded(true);
  };

  const handleImageError = () => {
    setImageError(true);
  };

  // Calculate transform for zoom and pan
  const getTransform = () => {
    const scale = state.zoom;
    const translateX = state.panX * scale;
    const translateY = state.panY * scale;
    return `translate(${translateX}px, ${translateY}px) scale(${scale})`;
  };

  return (
    <div className="relative w-full h-full flex items-center justify-center bg-black">
      <canvas
        ref={canvasRef}
        className="absolute inset-0 w-full h-full"
        style={{ display: state.currentFrame?.url ? 'none' : 'block' }}
      />

      {state.currentFrame?.url && (
        <div className="relative w-full h-full flex items-center justify-center overflow-hidden">
          <img
            src={state.currentFrame.url}
            alt={`Preview frame at ${state.currentTime.toFixed(2)}s`}
            className="max-w-full max-h-full object-contain"
            style={{
              transform: getTransform(),
              transformOrigin: 'center',
              transition: 'transform 0.1s ease-out',
              cursor: state.zoom > 1 ? 'move' : 'default'
            }}
            onLoad={handleImageLoad}
            onError={handleImageError}
            draggable={false}
          />

          {!imageLoaded && !imageError && (
            <div className="absolute inset-0 flex items-center justify-center bg-black bg-opacity-50">
              <div className="text-white text-lg">Loading frame...</div>
            </div>
          )}

          {imageError && (
            <div className="absolute inset-0 flex items-center justify-center bg-black bg-opacity-50">
              <div className="text-white text-lg">Failed to load frame</div>
            </div>
          )}
        </div>
      )}

      {!state.currentFrame && (
        <div className="flex flex-col items-center justify-center text-gray-500">
          <div className="text-6xl mb-4">🎬</div>
          <div className="text-lg">No preview available</div>
          <div className="text-sm mt-2">Start playback or seek to a frame</div>
        </div>
      )}

      {state.zoom !== 1 && (
        <div className="absolute bottom-4 left-4 bg-black bg-opacity-75 text-white px-3 py-1 rounded text-sm">
          {Math.round(state.zoom * 100)}%
        </div>
      )}

      {state.currentFrame && imageLoaded && (
        <div className="absolute top-4 left-4 bg-black bg-opacity-75 text-white px-3 py-1 rounded text-sm">
          {state.currentFrame.width} × {state.currentFrame.height}
        </div>
      )}

      {state.isPlaying && (
        <div className="absolute top-4 right-4 bg-red-600 text-white px-3 py-1 rounded text-sm flex items-center space-x-2">
          <div className="w-2 h-2 bg-white rounded-full animate-pulse" />
          <span>PLAYING</span>
        </div>
      )}

      {state.zoom > 2 && (
        <div className="absolute inset-0 pointer-events-none">
          <svg className="w-full h-full">
            <defs>
              <pattern id="grid" width="50" height="50" patternUnits="userSpaceOnUse">
                <path d="M 50 0 L 0 0 0 50" fill="none" stroke="white" strokeWidth="0.5" opacity="0.3"/>
              </pattern>
            </defs>
            <rect width="100%" height="100%" fill="url(#grid)" />
          </svg>
        </div>
      )}

      {state.zoom > 1 && (
        <div className="absolute inset-0 pointer-events-none">
          <div className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 border-2 border-yellow-400 opacity-50"
            style={{
              width: `${80 * state.zoom}%`,
              height: `${80 * state.zoom / (16/9)}%` // 16:9 aspect ratio
            }}
          >
            <div className="absolute -top-2 left-1/2 transform -translate-x-1/2 text-yellow-400 text-xs bg-black bg-opacity-75 px-1">
              Safe Area
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
