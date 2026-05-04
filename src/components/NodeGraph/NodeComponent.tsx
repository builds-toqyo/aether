'use client';

import React from 'react';
import { useNodeGraph } from './NodeGraphContext';
import { NodePort } from './NodePort';

interface NodeComponentProps {
  node: any;
  isSelected: boolean;
  isDragged: boolean;
  onMouseDown: (nodeId: string, e: React.MouseEvent) => void;
  onMouseUp: () => void;
}

export const NodeComponent: React.FC<NodeComponentProps> = ({
  node,
  isSelected,
  isDragged,
  onMouseDown,
  onMouseUp
}) => {
  const {
    setSelectedNode,
    handleCreateConnection,
    handleDeleteNode
  } = useNodeGraph();

  const handleNodeClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setSelectedNode(node);
  };

  const handlePortClick = (portId: string, portType: 'input' | 'output', e: React.MouseEvent) => {
    e.stopPropagation();
    
    if (portType === 'output') {
      // Start connection from output port
      // This would be handled in the context
    } else if (portType === 'input') {
      // Complete connection to input port
      // This would be handled in the context
    }
  };

  const handleDeleteClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    handleDeleteNode(node.id);
  };

  const getNodeColor = (nodeType: string) => {
    const colors = {
      input: 'bg-blue-600',
      output: 'bg-green-600',
      merge: 'bg-purple-600',
      transform: 'bg-orange-600',
      colorCorrection: 'bg-pink-600',
      blur: 'bg-indigo-600'
    };
    return colors[nodeType as keyof typeof colors] || 'bg-gray-600';
  };

  return (
    <div
      className={`absolute bg-gray-800 border-2 rounded-lg shadow-lg transition-all ${
        isSelected ? 'border-blue-400 shadow-blue-400/50' : 'border-gray-600'
      } ${isDragged ? 'opacity-75' : ''}`}
      style={{
        left: `${node.position.x}px`,
        top: `${node.position.y}px`,
        minWidth: '200px'
      }}
      onMouseDown={(e) => onMouseDown(node.id, e)}
      onMouseUp={onMouseUp}
      onClick={handleNodeClick}
    >

      <div className={`px-3 py-2 rounded-t-lg flex items-center justify-between ${getNodeColor(node.type)}`}>
        <span className="text-white font-medium text-sm">{node.name}</span>
        <button
          onClick={handleDeleteClick}
          className="text-white hover:text-red-300 transition-colors"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div className="p-3">
        {node.inputs.length > 0 && (
          <div className="space-y-1 mb-2">
            {node.inputs.map((port: any) => (
              <NodePort
                key={port.id}
                port={port}
                onPortClick={(portId, e) => handlePortClick(portId, 'input', e)}
              />
            ))}
          </div>
        )}

        <div className="space-y-2 mb-2">
          {Object.entries(node.parameters).map(([key, value]) => (
            <div key={key} className="flex items-center justify-between">
              <label className="text-xs text-gray-400">{key}:</label>
              <input
                type="text"
                value={String(value)}
                onChange={(e) => {
                  // Handle parameter update
                }}
                className="bg-gray-700 text-white text-xs px-2 py-1 rounded w-20"
              />
            </div>
          ))}
        </div>

        {node.outputs.length > 0 && (
          <div className="space-y-1">
            {node.outputs.map((port: any) => (
              <NodePort
                key={port.id}
                port={port}
                onPortClick={(portId, e) => handlePortClick(portId, 'output', e)}
              />
            ))}
          </div>
        )}
      </div>

      <div className="absolute bottom-0 right-0 w-3 h-3 cursor-se-resize">
        <svg className="w-full h-full text-gray-500" fill="currentColor" viewBox="0 0 24 24">
          <path d="M22 22H20V20H22V22ZM22 18H20V16H22V18ZM18 22H16V20H18V22ZM18 18H16V16H18V18ZM14 22H12V20H14V22Z" />
        </svg>
      </div>
    </div>
  );
};
