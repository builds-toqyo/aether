import React, { useRef, useEffect, useMemo, useCallback } from 'react';
import { ScopeData, ScopeSettings } from './index';

interface VectorscopeProps {
  data?: ScopeData['vectorscope'];
  settings: ScopeSettings['vectorscope'];
  isLoading: boolean;
}

export const Vectorscope: React.FC<VectorscopeProps> = ({
  data,
  settings,
  isLoading,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Calculate canvas dimensions
  const canvasSize = useMemo(() => {
    const size = 300;
    return { width: size, height: size };
  }, []);

  // Default color targets for vectorscope
  const defaultTargets = useMemo(() => [
    { name: 'White', x: 0, y: 0, color: '#ffffff', radius: 5 },
    { name: 'Black', x: 0, y: 0, color: '#000000', radius: 5 },
    { name: 'Red', x: 0.63, y: 0.33, color: '#ff0000', radius: 8 },
    { name: 'Green', x: -0.33, y: -0.33, color: '#00ff00', radius: 8 },
    { name: 'Blue', x: -0.33, y: 0.67, color: '#0000ff', radius: 8 },
    { name: 'Cyan', x: -0.33, y: 0.33, color: '#00ffff', radius: 6 },
    { name: 'Magenta', x: 0.33, y: 0.33, color: '#ff00ff', radius: 6 },
    { name: 'Yellow', x: 0.33, y: -0.33, color: '#ffff00', radius: 6 },
  ], []);

  // Draw grid lines
  const drawGrid = useCallback((ctx: CanvasRenderingContext2D) => {
    if (!settings.grid) return;

    const { width, height } = canvasSize;
    const centerX = width / 2;
    const centerY = height / 2;
    const radius = Math.min(width, height) / 2 - 20;

    ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
    ctx.lineWidth = 1;

    // Draw circular grid
    for (let r = 0.2; r <= 1.0; r += 0.2) {
      ctx.beginPath();
      ctx.arc(centerX, centerY, radius * r, 0, 2 * Math.PI);
      ctx.stroke();
    }

    // Draw cross lines
    ctx.beginPath();
    ctx.moveTo(centerX - radius, centerY);
    ctx.lineTo(centerX + radius, centerY);
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(centerX, centerY - radius);
    ctx.lineTo(centerX, centerY + radius);
    ctx.stroke();

    // Draw diagonal lines
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.05)';
    ctx.beginPath();
    ctx.moveTo(centerX - radius * 0.707, centerY - radius * 0.707);
    ctx.lineTo(centerX + radius * 0.707, centerY + radius * 0.707);
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(centerX + radius * 0.707, centerY - radius * 0.707);
    ctx.lineTo(centerX - radius * 0.707, centerY + radius * 0.707);
    ctx.stroke();
  }, [settings.grid, canvasSize]);

  // Draw color targets
  const drawTargets = useCallback((ctx: CanvasRenderingContext2D) => {
    if (!settings.targets) return;

    const { width, height } = canvasSize;
    const centerX = width / 2;
    const centerY = height / 2;
    const radius = Math.min(width, height) / 2 - 20;
    const targets = data?.targets || defaultTargets;

    targets.forEach(target => {
      // Convert UV coordinates to canvas coordinates
      const x = centerX + (target.x * radius * settings.chromaScale);
      const y = centerY - (target.y * radius * settings.chromaScale);

      // Draw target circle
      ctx.strokeStyle = target.color;
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(x, y, target.radius, 0, 2 * Math.PI);
      ctx.stroke();

      // Draw crosshair
      ctx.beginPath();
      ctx.moveTo(x - target.radius - 2, y);
      ctx.lineTo(x + target.radius + 2, y);
      ctx.stroke();

      ctx.beginPath();
      ctx.moveTo(x, y - target.radius - 2);
      ctx.lineTo(x, y + target.radius + 2);
      ctx.stroke();

      // Draw label
      ctx.fillStyle = target.color;
      ctx.font = '10px monospace';
      ctx.fillText(target.name, x + target.radius + 5, y - 5);
    });
  }, [settings.targets, settings.chromaScale, data?.targets, defaultTargets, canvasSize]);

  // Draw vectorscope data
  const drawVectorscope = useCallback((ctx: CanvasRenderingContext2D) => {
    if (!data?.points) return;

    const { width, height } = canvasSize;
    const centerX = width / 2;
    const centerY = height / 2;
    const radius = Math.min(width, height) / 2 - 20;

    // Create image data for efficient pixel drawing
    const imageData = ctx.createImageData(width, height);
    const pixels = imageData.data;

    data.points.forEach(point => {
      // Convert UV coordinates to canvas coordinates
      const x = Math.round(centerX + (point.x * radius * settings.chromaScale));
      const y = Math.round(centerY - (point.y * radius * settings.chromaScale));

      // Check if point is within canvas bounds
      if (x >= 0 && x < width && y >= 0 && y < height) {
        const index = (y * width + x) * 4;
        
        // Set pixel color with intensity
        const intensity = Math.min(255, point.intensity * 255 * settings.intensity);
        pixels[index] = intensity;     // R
        pixels[index + 1] = intensity; // G
        pixels[index + 2] = intensity; // B
        pixels[index + 3] = 255;       // A
      }
    });

    ctx.putImageData(imageData, 0, 0);
  }, [data, settings.intensity, settings.chromaScale, canvasSize]);

  // Draw color boxes in corners
  const drawColorBoxes = useCallback((ctx: CanvasRenderingContext2D) => {
    const { width, height } = canvasSize;
    const boxSize = 20;
    const padding = 10;

    // Primary colors
    const colors = [
      { color: '#ff0000', x: padding, y: height - padding - boxSize }, // Red (bottom left)
      { color: '#00ff00', x: width - padding - boxSize, y: height - padding - boxSize }, // Green (bottom right)
      { color: '#0000ff', x: padding, y: padding }, // Blue (top left)
      { color: '#ffffff', x: width - padding - boxSize, y: padding }, // White (top right)
    ];

    colors.forEach(({ color, x, y }) => {
      ctx.fillStyle = color;
      ctx.fillRect(x, y, boxSize, boxSize);
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.3)';
      ctx.lineWidth = 1;
      ctx.strokeRect(x, y, boxSize, boxSize);
    });
  }, [canvasSize]);

  // Main drawing function
  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas with black background
    ctx.fillStyle = '#000000';
    ctx.fillRect(0, 0, canvasSize.width, canvasSize.height);

    // Draw circular clipping mask
    const centerX = canvasSize.width / 2;
    const centerY = canvasSize.height / 2;
    const radius = Math.min(canvasSize.width, canvasSize.height) / 2 - 20;

    ctx.save();
    ctx.beginPath();
    ctx.arc(centerX, centerY, radius, 0, 2 * Math.PI);
    ctx.clip();

    // Draw grid
    drawGrid(ctx);

    // Draw vectorscope data
    drawVectorscope(ctx);

    ctx.restore();

    // Draw targets (outside clipping)
    drawTargets(ctx);

    // Draw color boxes
    drawColorBoxes(ctx);
  }, [canvasSize, drawGrid, drawVectorscope, drawTargets, drawColorBoxes]);

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
    <div className="flex flex-col h-full" ref={containerRef}>
      <div className="flex justify-between items-center p-2 bg-gray-800 border-b border-gray-700">
        <h4 className="text-sm font-semibold text-white">Vectorscope</h4>
        <div className="text-xs text-gray-300">
          <span className="text-gray-600">Chroma: {(settings.chromaScale * 100).toFixed(0)}%</span>
          {settings.intensity !== 1.0 && <span className="ml-2 text-gray-600">Intensity: {settings.intensity.toFixed(1)}x</span>}
        </div>
      </div>
      <div className="relative flex items-center justify-center bg-black min-h-[300px]">
        <canvas
          ref={canvasRef}
          className={`block ${isLoading ? 'opacity-50' : ''}`}
          style={{ imageRendering: 'crisp-edges', width: canvasSize.width, height: canvasSize.height }}
        />
        {isLoading && (
          <div className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
            <div className="w-6 h-6 border-2 border-gray-600 border-t-2 border-t-white rounded-full animate-spin"></div>
          </div>
        )}
      </div>
      <div className="p-2 bg-gray-800 border-t border-gray-700">
        <div className="flex gap-1">
          <span className="px-1 py-0.5 bg-red-500 bg-opacity-70 text-white text-xs font-semibold rounded">R</span>
          <span className="px-1 py-0.5 bg-green-500 bg-opacity-70 text-black text-xs font-semibold rounded">G</span>
          <span className="px-1 py-0.5 bg-blue-500 bg-opacity-70 text-white text-xs font-semibold rounded">B</span>
          <span className="px-1 py-0.5 bg-cyan-500 bg-opacity-70 text-black text-xs font-semibold rounded">C</span>
          <span className="px-1 py-0.5 bg-purple-500 bg-opacity-70 text-white text-xs font-semibold rounded">M</span>
          <span className="px-1 py-0.5 bg-yellow-500 bg-opacity-70 text-black text-xs font-semibold rounded">Y</span>
        </div>
      </div>
    </div>
  );
};
