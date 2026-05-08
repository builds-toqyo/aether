import React, { useRef, useEffect, useMemo, useCallback } from 'react';
import { ScopeData, ScopeSettings } from './index';

interface WaveformScopeProps {
  data?: ScopeData['waveform'];
  settings: ScopeSettings['waveform'];
  isLoading: boolean;
}

export const WaveformScope: React.FC<WaveformScopeProps> = ({
  data,
  settings,
  isLoading,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Calculate canvas dimensions
  const canvasSize = useMemo(() => {
    const width = 400;
    const height = 300;
    return { width, height };
  }, []);

  // Draw grid lines
  const drawGrid = useCallback((ctx: CanvasRenderingContext2D) => {
    if (!settings.grid) return;

    const { width, height } = canvasSize;
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
    ctx.lineWidth = 1;

    // Vertical lines
    for (let x = 0; x <= width; x += 40) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
    }

    // Horizontal lines
    for (let y = 0; y <= height; y += 30) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }

    // Draw IRE markers (0, 100, 7.5, 100 IRE levels)
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.3)';
    ctx.lineWidth = 2;
    
    // 0 IRE (black)
    ctx.beginPath();
    ctx.moveTo(0, height);
    ctx.lineTo(width, height);
    ctx.stroke();
    
    // 100 IRE (white)
    ctx.beginPath();
    ctx.moveTo(0, 0);
    ctx.lineTo(width, 0);
    ctx.stroke();
    
    // 7.5 IRE (black level for NTSC)
    const blackLevel = height * (1 - 0.075);
    ctx.beginPath();
    ctx.moveTo(0, blackLevel);
    ctx.lineTo(width, blackLevel);
    ctx.stroke();
  }, [settings.grid, canvasSize]);

  // Draw waveform data
  const drawWaveform = useCallback((ctx: CanvasRenderingContext2D) => {
    if (!data) return;

    const { width, height } = canvasSize;
    
    if (settings.mode === 'luma' && data.luma) {
      // Draw luma waveform
      ctx.strokeStyle = `rgba(255, 255, 255, ${settings.intensity})`;
      ctx.lineWidth = 1;
      ctx.beginPath();

      for (let x = 0; x < width && x < data.luma.length; x++) {
        const column = data.luma[x];
        if (!column) continue;

        for (let y = 0; y < height && y < column.length; y++) {
          const value = column[y];
          if (value > 0.01) { // Threshold for visibility
            const canvasY = height - y;
            if (x === 0) {
              ctx.moveTo(x, canvasY);
            } else {
              ctx.lineTo(x, canvasY);
            }
          }
        }
      }
      ctx.stroke();
    } else if (settings.mode === 'rgb' && data.rgb) {
      // Draw RGB waveform (overlaid)
      const channels = [
        { data: data.rgb.red, color: `rgba(255, 0, 0, ${settings.intensity * 0.7})` },
        { data: data.rgb.green, color: `rgba(0, 255, 0, ${settings.intensity * 0.7})` },
        { data: data.rgb.blue, color: `rgba(0, 0, 255, ${settings.intensity * 0.7})` },
      ];

      channels.forEach(channel => {
        if (!channel.data) return;
        
        ctx.strokeStyle = channel.color;
        ctx.lineWidth = 1;
        ctx.beginPath();

        for (let x = 0; x < width && x < channel.data.length; x++) {
          const column = channel.data[x];
          if (!column) continue;

          for (let y = 0; y < height && y < column.length; y++) {
            const value = column[y];
            if (value > 0.01) {
              const canvasY = height - y;
              if (x === 0) {
                ctx.moveTo(x, canvasY);
              } else {
                ctx.lineTo(x, canvasY);
              }
            }
          }
        }
        ctx.stroke();
      });
    } else if (settings.mode === 'parade' && data.rgb) {
      // Draw RGB parade (side by side)
      const channelWidth = width / 3;
      const channels = [
        { data: data.rgb.red, color: `rgba(255, 0, 0, ${settings.intensity})`, offsetX: 0 },
        { data: data.rgb.green, color: `rgba(0, 255, 0, ${settings.intensity})`, offsetX: channelWidth },
        { data: data.rgb.blue, color: `rgba(0, 0, 255, ${settings.intensity})`, offsetX: channelWidth * 2 },
      ];

      channels.forEach(channel => {
        if (!channel.data) return;
        
        ctx.strokeStyle = channel.color;
        ctx.lineWidth = 1;
        ctx.beginPath();

        for (let x = 0; x < channelWidth && x < channel.data.length; x++) {
          const column = channel.data[x];
          if (!column) continue;

          for (let y = 0; y < height && y < column.length; y++) {
            const value = column[y];
            if (value > 0.01) {
              const canvasX = channel.offsetX + x;
              const canvasY = height - y;
              if (x === 0 && channel.offsetX === 0) {
                ctx.moveTo(canvasX, canvasY);
              } else {
                ctx.lineTo(canvasX, canvasY);
              }
            }
          }
        }
        ctx.stroke();
      });

      // Draw channel separators
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.2)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(channelWidth, 0);
      ctx.lineTo(channelWidth, height);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(channelWidth * 2, 0);
      ctx.lineTo(channelWidth * 2, height);
      ctx.stroke();
    }
  }, [data, settings.mode, settings.intensity, canvasSize]);

  // Main drawing function
  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.fillStyle = '#000000';
    ctx.fillRect(0, 0, canvasSize.width, canvasSize.height);

    // Draw grid
    drawGrid(ctx);

    // Draw waveform
    drawWaveform(ctx);
  }, [canvasSize, drawGrid, drawWaveform]);

  // Update canvas when data or settings change
  useEffect(() => {
    draw();
  }, [draw]);

  // Set canvas size
  useEffect(() => {
    const canvas = canvasRef.current;
    if (canvas) {
      canvas.width = canvasSize.width;
      canvas.height = canvasSize.height;
      draw();
    }
  }, [canvasSize, draw]);

  return (
    <div className="waveform-scope" ref={containerRef}>
      <div className="scope-header">
        <h4>Waveform</h4>
        <div className="scope-mode">
          {settings.mode === 'luma' && 'Luma'}
          {settings.mode === 'rgb' && 'RGB'}
          {settings.mode === 'parade' && 'Parade'}
        </div>
      </div>
      <div className="scope-content">
        <canvas
          ref={canvasRef}
          className={`scope-canvas ${isLoading ? 'loading' : ''}`}
          style={{ width: canvasSize.width, height: canvasSize.height }}
        />
        {isLoading && (
          <div className="scope-loading">
            <div className="loading-spinner"></div>
          </div>
        )}
      </div>
      <div className="scope-footer">
        <div className="scope-info">
          <span className="ire-markers">0-100 IRE</span>
          {settings.mode === 'parade' && <span className="channel-info">R | G | B</span>}
        </div>
      </div>
    </div>
  );
};
