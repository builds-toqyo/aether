import React, { useRef, useEffect, useMemo, useCallback } from 'react';
import { ScopeData, ScopeSettings } from './index';

interface HistogramProps {
  data?: ScopeData['histogram'];
  settings: ScopeSettings['histogram'];
  isLoading: boolean;
}

export const Histogram: React.FC<HistogramProps> = ({
  data,
  settings,
  isLoading,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Calculate canvas dimensions
  const canvasSize = useMemo(() => {
    const width = 400;
    const height = 250;
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
    for (let y = 0; y <= height; y += 25) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }

    // Draw percentage markers
    ctx.fillStyle = 'rgba(255, 255, 255, 0.5)';
    ctx.font = '10px monospace';
    
    for (let i = 0; i <= 100; i += 25) {
      const y = height - (height * i / 100);
      ctx.fillText(`${i}%`, 5, y - 2);
    }
  }, [settings.grid, canvasSize]);

  // Apply logarithmic scaling to histogram values
  const applyLogScale = useCallback((value: number) => {
    if (!settings.logarithmic) return value;
    return Math.log10(value + 1) / Math.log10(2); // Log scale with base 2
  }, [settings.logarithmic]);

  // Get maximum value for scaling
  const getMaxValue = useCallback(() => {
    if (!data) return 1;

    let maxValue = 0;
    
    if (settings.mode === 'rgb' || settings.mode === 'all') {
      if (data.red) maxValue = Math.max(maxValue, ...data.red);
      if (data.green) maxValue = Math.max(maxValue, ...data.green);
      if (data.blue) maxValue = Math.max(maxValue, ...data.blue);
    }
    
    if (settings.mode === 'luma' || settings.mode === 'all') {
      if (data.luma) maxValue = Math.max(maxValue, ...data.luma);
    }

    return maxValue || 1;
  }, [data, settings.mode]);

  // Draw histogram data
  const drawHistogram = useCallback((ctx: CanvasRenderingContext2D) => {
    if (!data) return;

    const { width, height } = canvasSize;
    const maxValue = getMaxValue();
    const barWidth = width / 256; // 256 levels for 8-bit color

    const drawChannel = (channelData: number[], color: string, alpha: number = 1.0) => {
      if (!channelData) return;

      ctx.fillStyle = color;
      ctx.globalAlpha = alpha;

      for (let i = 0; i < channelData.length && i < 256; i++) {
        const value = channelData[i];
        if (value > 0) {
          const scaledValue = applyLogScale(value);
          const barHeight = (scaledValue / maxValue) * height;
          const x = i * barWidth;
          const y = height - barHeight;

          ctx.fillRect(x, y, barWidth - 1, barHeight);
        }
      }

      ctx.globalAlpha = 1.0;
    };

    // Draw based on mode
    if (settings.mode === 'rgb') {
      drawChannel(data.red, 'rgba(255, 0, 0, 0.7)');
      drawChannel(data.green, 'rgba(0, 255, 0, 0.7)');
      drawChannel(data.blue, 'rgba(0, 0, 255, 0.7)');
    } else if (settings.mode === 'luma') {
      drawChannel(data.luma, 'rgba(255, 255, 255, 0.9)');
    } else if (settings.mode === 'all') {
      drawChannel(data.red, 'rgba(255, 0, 0, 0.5)');
      drawChannel(data.green, 'rgba(0, 255, 0, 0.5)');
      drawChannel(data.blue, 'rgba(0, 0, 255, 0.5)');
      drawChannel(data.luma, 'rgba(255, 255, 255, 0.3)');
    }
  }, [data, settings.mode, canvasSize, getMaxValue, applyLogScale]);

  // Draw channel labels
  const drawLabels = useCallback((ctx: CanvasRenderingContext2D) => {
    const { width, height } = canvasSize;
    
    ctx.fillStyle = 'rgba(255, 255, 255, 0.7)';
    ctx.font = '12px monospace';

    if (settings.mode === 'rgb' || settings.mode === 'all') {
      // RGB labels
      ctx.fillStyle = 'rgba(255, 0, 0, 0.8)';
      ctx.fillText('R', width - 20, 15);
      
      ctx.fillStyle = 'rgba(0, 255, 0, 0.8)';
      ctx.fillText('G', width - 35, 15);
      
      ctx.fillStyle = 'rgba(0, 0, 255, 0.8)';
      ctx.fillText('B', width - 50, 15);
    }

    if (settings.mode === 'luma') {
      ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
      ctx.fillText('Luma', width - 40, 15);
    }

    if (settings.mode === 'all') {
      ctx.fillStyle = 'rgba(255, 255, 255, 0.5)';
      ctx.fillText('Luma', width - 70, 15);
    }

    // Draw scale indicators
    ctx.fillStyle = 'rgba(255, 255, 255, 0.5)';
    ctx.font = '10px monospace';
    ctx.fillText('0', 5, height - 5);
    ctx.fillText('255', width - 25, height - 5);
  }, [settings.mode, canvasSize]);

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

    // Draw histogram
    drawHistogram(ctx);

    // Draw labels
    drawLabels(ctx);
  }, [canvasSize, drawGrid, drawHistogram, drawLabels]);

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
    <div className="histogram" ref={containerRef}>
      <div className="scope-header">
        <h4>Histogram</h4>
        <div className="scope-mode">
          {settings.mode === 'rgb' && 'RGB'}
          {settings.mode === 'luma' && 'Luma'}
          {settings.mode === 'all' && 'All'}
          {settings.logarithmic && ' (Log)'}
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
          <span className="scale-info">0-255 levels</span>
          <span className="max-value">Max: {getMaxValue().toFixed(0)}</span>
        </div>
      </div>
    </div>
  );
};
