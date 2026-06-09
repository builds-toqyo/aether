'use client';

import React, { useRef, useEffect, useState } from 'react';
import { useNodeGraph } from './NodeGraphContext';
import { NodeComponent } from './NodeComponent';
import { ConnectionComponent } from './ConnectionComponent';
import { ConnectionLine } from './ConnectionLine';

export const NodeGraphCanvas: React.FC = () => {
  const {
    graph,
    selectedNode,
    selectedConnection,
    isConnecting,
    connectionStart,
    viewport,
    canvasRef,
    svgRef,
    handleCanvasMouseDown,
    handleCanvasMouseMove,
    handleCanvasMouseUp,
    handleWheel
  } = useNodeGraph();

  const [mousePosition, setMousePosition] = useState({ x: 0, y: 0 });
  const [draggedNode, setDraggedNode] = useState<string | null>(null);
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });

  // Track mouse position for connection preview
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (canvasRef.current) {
        const rect = canvasRef.current.getBoundingClientRect();
        const x = (e.clientX - rect.left - viewport.x) / viewport.zoom;
        const y = (e.clientY - rect.top - viewport.y) / viewport.zoom;
        setMousePosition({ x, y });
      }
    };

    window.addEventListener('mousemove', handleMouseMove);
    return () => window.removeEventListener('mousemove', handleMouseMove);
  }, [viewport, canvasRef]);

  // Handle node dragging
  const handleNodeMouseDown = (nodeId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const node = graph.nodes.find(n => n.id === nodeId);
    if (node) {
      setDraggedNode(nodeId);
      const rect = canvasRef.current?.getBoundingClientRect();
      if (rect) {
        const mouseX = (e.clientX - rect.left - viewport.x) / viewport.zoom;
        const mouseY = (e.clientY - rect.top - viewport.y) / viewport.zoom;
        setDragOffset({
          x: mouseX - node.position.x,
          y: mouseY - node.position.y
        });
      }
    }
  };

  const handleNodeDrag = (e: React.MouseEvent) => {
    if (draggedNode) {
      const rect = canvasRef.current?.getBoundingClientRect();
      if (rect) {
        const mouseX = (e.clientX - rect.left - viewport.x) / viewport.zoom;
        const mouseY = (e.clientY - rect.top - viewport.y) / viewport.zoom;
        
        const newX = mouseX - dragOffset.x;
        const newY = mouseY - dragOffset.y;
        
        // Update node position (this would need to be handled in the context)
        // For now, we'll just update the local state
      }
    }
  };

  const handleNodeMouseUp = () => {
    if (draggedNode) {
      setDraggedNode(null);
      // Update the node position in the graph
    }
  };

  return (
    <div
      ref={canvasRef}
      className="w-full h-full bg-gray-800 relative overflow-hidden"
      onMouseDown={handleCanvasMouseDown}
      onMouseMove={(e) => {
        handleCanvasMouseMove(e);
        handleNodeDrag(e);
      }}
      onMouseUp={handleCanvasMouseUp}
      onWheel={handleWheel}
      style={{ cursor: isConnecting ? 'crosshair' : 'default' }}
    >
      <div
        className="absolute inset-0 opacity-20"
        style={{
          backgroundImage: `
            linear-gradient(to right, #374151 1px, transparent 1px),
            linear-gradient(to bottom, #374151 1px, transparent 1px)
          `,
          backgroundSize: `${20 * viewport.zoom}px ${20 * viewport.zoom}px`,
          backgroundPosition: `${viewport.x}px ${viewport.y}px`,
          transform: `scale(${viewport.zoom})`,
          transformOrigin: '0 0'
        }}
      />

      <svg
        ref={svgRef}
        className="absolute inset-0 pointer-events-none"
        style={{
          transform: `translate(${viewport.x}px, ${viewport.y}px) scale(${viewport.zoom})`,
          transformOrigin: '0 0'
        }}
      >
        {graph.connections.map(connection => (
          <ConnectionComponent
            key={connection.id}
            connection={connection}
            sourceNode={graph.nodes.find(n => n.id === connection.sourceNodeId)}
            targetNode={graph.nodes.find(n => n.id === connection.targetNodeId)}
            isSelected={selectedConnection?.id === connection.id}
          />
        ))}

        {isConnecting && connectionStart && (
          <ConnectionLine
            start={getNodePortPosition(graph.nodes.find(n => n.id === connectionStart.nodeId), connectionStart.portId)}
            end={mousePosition}
            isPreview={true}
          />
        )}
      </svg>

      {graph.nodes.map(node => (
        <NodeComponent
          key={node.id}
          node={node}
          isSelected={selectedNode?.id === node.id}
          isDragged={draggedNode === node.id}
          onMouseDown={handleNodeMouseDown}
          onMouseUp={handleNodeMouseUp}
        />
      ))}

      <div className="absolute bottom-4 right-4 bg-gray-900 bg-opacity-80 text-white px-3 py-1 rounded text-sm">
        {(viewport.zoom * 100).toFixed(0)}%
      </div>

      {isConnecting && (
        <div className="absolute top-4 left-1/2 transform -translate-x-1/2 bg-blue-600 text-white px-4 py-2 rounded text-sm">
          Connecting... (Click to cancel)
        </div>
      )}
    </div>
  );
};

// Helper function to get node port position
function getNodePortPosition(node: any, portId: string): { x: number; y: number } {
  if (!node) return { x: 0, y: 0 };
  
  // Find the port and return its position
  const port = [...node.inputs, ...node.outputs].find(p => p.id === portId);
  if (port) {
    return {
      x: node.position.x + port.position.x,
      y: node.position.y + port.position.y
    };
  }
  
  return { x: node.position.x, y: node.position.y };
}
