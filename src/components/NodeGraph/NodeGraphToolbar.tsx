'use client';

import React from 'react';
import { useNodeGraph } from './NodeGraphContext';
import { FolderOpen, Upload, Merge, RotateCw, Palette, Droplets } from 'lucide-react';

export const NodeGraphToolbar: React.FC = () => {
  const { handleCreateNode, graph, viewport, setViewport } = useNodeGraph();

  const nodeTypes = [
    { type: 'input', name: 'Input', icon: <FolderOpen className="w-5 h-5" /> },
    { type: 'output', name: 'Output', icon: <Upload className="w-5 h-5" /> },
    { type: 'merge', name: 'Merge', icon: <Merge className="w-5 h-5" /> },
    { type: 'transform', name: 'Transform', icon: <RotateCw className="w-5 h-5" /> },
    { type: 'colorCorrection', name: 'Color Correction', icon: <Palette className="w-5 h-5" /> },
    { type: 'blur', name: 'Blur', icon: <Droplets className="w-5 h-5" /> }
  ];

  const handleNodeCreate = (nodeType: string) => {
    // Position new node in center of viewport
    const centerX = (window.innerWidth / 2 - viewport.x) / viewport.zoom;
    const centerY = (window.innerHeight / 2 - viewport.y) / viewport.zoom;
    
    handleCreateNode(nodeType, { x: centerX, y: centerY });
  };

  const handleZoomIn = () => {
    setViewport((prev) => ({
      ...prev,
      zoom: Math.min(5, prev.zoom * 1.2)
    }));
  };

  const handleZoomOut = () => {
    setViewport((prev) => ({
      ...prev,
      zoom: Math.max(0.1, prev.zoom / 1.2)
    }));
  };

  const handleZoomReset = () => {
    setViewport({ x: 0, y: 0, zoom: 1 });
  };

  const handleFitToScreen = () => {
    if (graph.nodes.length === 0) return;

    // Calculate bounding box of all nodes
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    
    graph.nodes.forEach(node => {
      minX = Math.min(minX, node.position.x);
      minY = Math.min(minY, node.position.y);
      maxX = Math.max(maxX, node.position.x + 200); // Approximate node width
      maxY = Math.max(maxY, node.position.y + 150); // Approximate node height
    });

    const graphWidth = maxX - minX;
    const graphHeight = maxY - minY;
    const centerX = (minX + maxX) / 2;
    const centerY = (minY + maxY) / 2;

    // Calculate zoom to fit graph in view
    const viewWidth = window.innerWidth - 200; // Account for toolbar and sidebar
    const viewHeight = window.innerHeight - 100; // Account for toolbar
    
    const zoom = Math.min(viewWidth / graphWidth, viewHeight / graphHeight, 2);

    setViewport({
      x: (viewWidth / 2) - (centerX * zoom),
      y: (viewHeight / 2) - (centerY * zoom),
      zoom
    });
  };

  return (
    <div className="w-16 bg-gray-900 border-r border-gray-700 flex flex-col items-center py-4 space-y-2">
      <div className="mb-4">
        <div className="w-10 h-10 bg-blue-600 rounded-lg flex items-center justify-center text-white font-bold">
          A
        </div>
      </div>

      <div className="space-y-2">
        {nodeTypes.map(nodeType => (
          <button
            key={nodeType.type}
            onClick={() => handleNodeCreate(nodeType.type)}
            className="w-12 h-12 bg-gray-800 hover:bg-gray-700 rounded-lg flex flex-col items-center justify-center text-gray-300 hover:text-white transition-colors group"
            title={`Create ${nodeType.name} node`}
          >
            <div className="text-lg">{nodeType.icon}</div>
            <span className="text-xs opacity-0 group-hover:opacity-100 transition-opacity">
              {nodeType.name}
            </span>
          </button>
        ))}
      </div>

      <div className="w-10 h-px bg-gray-700" />

      <div className="space-y-2">
        <button
          onClick={handleZoomIn}
          className="w-12 h-12 bg-gray-800 hover:bg-gray-700 rounded-lg flex items-center justify-center text-gray-300 hover:text-white transition-colors"
          title="Zoom In"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
          </svg>
        </button>

        <button
          onClick={handleZoomOut}
          className="w-12 h-12 bg-gray-800 hover:bg-gray-700 rounded-lg flex items-center justify-center text-gray-300 hover:text-white transition-colors"
          title="Zoom Out"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 12H4" />
          </svg>
        </button>

        <button
          onClick={handleZoomReset}
          className="w-12 h-12 bg-gray-800 hover:bg-gray-700 rounded-lg flex items-center justify-center text-gray-300 hover:text-white transition-colors"
          title="Reset View"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
          </svg>
        </button>

        <button
          onClick={handleFitToScreen}
          className="w-12 h-12 bg-gray-800 hover:bg-gray-700 rounded-lg flex items-center justify-center text-gray-300 hover:text-white transition-colors"
          title="Fit to Screen"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
          </svg>
        </button>
      </div>

      <div className="mt-auto text-xs text-gray-500 text-center">
        <div>{graph.nodes.length} nodes</div>
        <div>{graph.connections.length} connections</div>
      </div>
    </div>
  );
};
