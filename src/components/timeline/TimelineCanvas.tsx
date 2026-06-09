'use client';

import React, { useRef, useEffect, useState, useCallback } from 'react';
import { useTimeline } from './TimelineContext';
import { TimelineTrack } from './TimelineTrack';
import { TimelinePlayhead } from './TimelinePlayhead';

export const TimelineCanvas: React.FC = () => {
  const { 
    state, 
    isDragging, 
    dragMode, 
    isTrimming, 
    trimStart,
    setIsDragging,
    setDragMode,
    setIsTrimming,
    setTrimStart,
    handleClipSelect,
    handleClipMove,
    handleClipTrim,
    handleClipDelete,
    handleScroll,
    canvasRef
  } = useTimeline();

  const [draggedClip, setDraggedClip] = useState<{ clipId: string; startX: number; startTrackId: string; startTime: number } | null>(null);
  const [hoveredClip, setHoveredClip] = useState<string | null>(null);
  const [totalHeight, setTotalHeight] = useState(0);

  // Calculate total height of all tracks
  useEffect(() => {
    const height = state.tracks.reduce((sum, track) => sum + track.height + 1, 0); // +1 for gap
    setTotalHeight(height);
  }, [state.tracks]);

  // Handle mouse events for clip manipulation
  const handleCanvasMouseDown = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    if (e.button === 0) { // Left click only
      const rect = canvasRef.current?.getBoundingClientRect();
      if (rect) {
        const clickX = e.clientX - rect.left;
        const clickY = e.clientY - rect.top;
        
        // Convert to timeline coordinates
        const pixelsPerSecond = 50 * state.zoom;
        const time = (clickX + state.scrollX) / pixelsPerSecond;
        
        // Find which track was clicked
        let currentY = 0;
        let clickedTrack: typeof state.tracks[0] | null = null;
        
        for (const track of state.tracks) {
          if (clickY >= currentY && clickY < currentY + track.height) {
            clickedTrack = track;
            break;
          }
          currentY += track.height + 1;
        }
        
        // Find if a clip was clicked
        let clickedClip = null;
        if (clickedTrack) {
          clickedClip = clickedTrack.clips.find(clip => 
            time >= clip.startTime && time <= clip.startTime + clip.duration
          );
        }
        
        if (clickedClip) {
          // Start dragging the clip
          setDraggedClip({
            clipId: clickedClip.id,
            startX: clickX,
            startTrackId: clickedClip.trackId,
            startTime: clickedClip.startTime
          });
          setIsDragging(true);
          handleClipSelect(clickedClip.id, e.shiftKey);
        } else {
          // Deselect all clips
          handleClipSelect('', false);
        }
      }
    }
  }, [state, canvasRef, handleClipSelect, setIsDragging]);

  const handleCanvasMouseMove = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    const rect = canvasRef.current?.getBoundingClientRect();
    if (rect) {
      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;
      
      // Handle horizontal scrolling
      if (mouseX < 50) {
        handleScroll(Math.max(0, state.scrollX - 5));
      } else if (mouseX > rect.width - 50) {
        handleScroll(state.scrollX + 5);
      }
      
      // Handle clip dragging
      if (isDragging && draggedClip) {
        const pixelsPerSecond = 50 * state.zoom;
        const deltaTime = (mouseX - draggedClip.startX) / pixelsPerSecond;
        const newTime = Math.max(0, draggedClip.startTime + deltaTime);
        
        // Find target track
        let currentY = 0;
        let targetTrackId = draggedClip.startTrackId;
        
        for (const track of state.tracks) {
          if (mouseY >= currentY && mouseY < currentY + track.height) {
            targetTrackId = track.id;
            break;
          }
          currentY += track.height + 1;
        }
        
        handleClipMove(draggedClip.clipId, newTime, targetTrackId);
      }
      
      // Update hover state
      const pixelsPerSecond = 50 * state.zoom;
      const time = (mouseX + state.scrollX) / pixelsPerSecond;
      
      let newHoveredClip = null;
      let currentY = 0;
      
      for (const track of state.tracks) {
        if (mouseY >= currentY && mouseY < currentY + track.height) {
          newHoveredClip = track.clips.find(clip => 
            time >= clip.startTime && time <= clip.startTime + clip.duration
          )?.id || null;
          break;
        }
        currentY += track.height + 1;
      }
      
      setHoveredClip(newHoveredClip);
    }
  }, [isDragging, draggedClip, state, canvasRef, handleScroll, handleClipMove]);

  const handleCanvasMouseUp = useCallback(() => {
    if (isDragging) {
      setIsDragging(false);
      setDraggedClip(null);
    }
    if (isTrimming) {
      setIsTrimming(false);
      setTrimStart(null);
    }
  }, [isDragging, isTrimming, setIsDragging, setIsTrimming, setTrimStart]);

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement) return;
      
      switch (e.key) {
        case 'Delete':
        case 'Backspace':
          state.selectedClips.forEach(clipId => handleClipDelete(clipId));
          break;
        case ' ':
          e.preventDefault();
          // Toggle play/pause
          break;
        case 'm':
          if (e.shiftKey) {
            setDragMode('move');
          }
          break;
        case 't':
          if (e.shiftKey) {
            setDragMode('trim');
          }
          break;
        case 's':
          if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            // Split selected clips at current time
            state.selectedClips.forEach(clipId => {
              // Handle split logic
            });
          }
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [state.selectedClips, handleClipDelete, setDragMode]);

  return (
    <div
      ref={canvasRef}
      className="relative w-full h-full bg-gray-850 overflow-hidden cursor-crosshair"
      onMouseDown={handleCanvasMouseDown}
      onMouseMove={handleCanvasMouseMove}
      onMouseUp={handleCanvasMouseUp}
      onMouseLeave={handleCanvasMouseUp}
      style={{ backgroundColor: '#1a1a1a' }}
    >
      <div
        className="relative"
        style={{
          transform: `translateX(-${state.scrollX}px)`,
          width: `${state.duration * 50 * state.zoom}px`,
          height: `${totalHeight}px`
        }}
      >
        {state.tracks.map((track, index) => {
          const trackTop = state.tracks.slice(0, index).reduce((sum, t) => sum + t.height + 1, 0);
          return (
            <TimelineTrack
              key={track.id}
              track={track}
              top={trackTop}
              pixelsPerSecond={50 * state.zoom}
              hoveredClip={hoveredClip}
              selectedClips={state.selectedClips}
              onClipSelect={handleClipSelect}
              onClipTrim={handleClipTrim}
            />
          );
        })}
        
        <TimelinePlayhead />
      </div>
      
      <div className="absolute top-2 left-2 bg-gray-800 px-2 py-1 rounded text-xs text-white">
        Mode: {dragMode.charAt(0).toUpperCase() + dragMode.slice(1)}
      </div>
      
      {state.selectedClips.length > 0 && (
        <div className="absolute top-2 left-24 bg-blue-600 px-2 py-1 rounded text-xs text-white">
          {state.selectedClips.length} clip{state.selectedClips.length > 1 ? 's' : ''} selected
        </div>
      )}
    </div>
  );
};
