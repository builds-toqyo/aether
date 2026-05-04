'use client';

import React from 'react';
import { useNodeGraph } from './NodeGraphContext';

export const NodeGraphSidebar: React.FC = () => {
  const { selectedNode, selectedConnection, handleUpdateNodeParameter, graph } = useNodeGraph();

  if (!selectedNode && !selectedConnection) {
    return (
      <div className="w-80 bg-gray-900 border-l border-gray-700 p-4">
        <div className="text-gray-500 text-center mt-8">
          <svg className="w-16 h-16 mx-auto mb-4 text-gray-700" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p className="text-sm">Select a node or connection to view properties</p>
        </div>
      </div>
    );
  }

  if (selectedConnection) {
    return (
      <div className="w-80 bg-gray-900 border-l border-gray-700 p-4">
        <h3 className="text-white font-semibold mb-4">Connection Properties</h3>
        
        <div className="space-y-4">
          <div>
            <label className="text-gray-400 text-sm">Connection ID</label>
            <p className="text-white text-sm font-mono">{selectedConnection.id}</p>
          </div>
          
          <div>
            <label className="text-gray-400 text-sm">Source</label>
            <p className="text-white text-sm">
              {graph.nodes.find(n => n.id === selectedConnection.sourceNodeId)?.name || 'Unknown'}
            </p>
            <p className="text-gray-500 text-xs">
              Port: {selectedConnection.sourcePortId}
            </p>
          </div>
          
          <div>
            <label className="text-gray-400 text-sm">Target</label>
            <p className="text-white text-sm">
              {graph.nodes.find(n => n.id === selectedConnection.targetNodeId)?.name || 'Unknown'}
            </p>
            <p className="text-gray-500 text-xs">
              Port: {selectedConnection.targetPortId}
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="w-80 bg-gray-900 border-l border-gray-700 p-4">
      <h3 className="text-white font-semibold mb-4">Node Properties</h3>
      
      <div className="space-y-4">
        <div>
          <label className="text-gray-400 text-sm">Node Type</label>
          <p className="text-white text-sm capitalize">{selectedNode?.type}</p>
        </div>
        
        <div>
          <label className="text-gray-400 text-sm">Node ID</label>
          <p className="text-white text-sm font-mono">{selectedNode?.id}</p>
        </div>
        
        <div>
          <label className="text-gray-400 text-sm">Position</label>
          <div className="flex space-x-2">
            <div>
              <label className="text-gray-500 text-xs">X</label>
              <input
                type="number"
                value={selectedNode?.position.x || 0}
                onChange={(e) => {
                  // Handle position update
                }}
                className="bg-gray-800 text-white text-sm px-2 py-1 rounded w-20"
              />
            </div>
            <div>
              <label className="text-gray-500 text-xs">Y</label>
              <input
                type="number"
                value={selectedNode?.position.y || 0}
                onChange={(e) => {
                  // Handle position update
                }}
                className="bg-gray-800 text-white text-sm px-2 py-1 rounded w-20"
              />
            </div>
          </div>
        </div>

        <div>
          <h4 className="text-white font-medium mb-2">Parameters</h4>
          <div className="space-y-2">
            {selectedNode && Object.entries(selectedNode.parameters).map(([key, value]) => (
              <div key={key}>
                <label className="text-gray-400 text-sm capitalize">{key}</label>
                {typeof value === 'boolean' ? (
                  <input
                    type="checkbox"
                    checked={value as boolean}
                    onChange={(e) => handleUpdateNodeParameter(selectedNode.id, key, e.target.checked)}
                    className="bg-gray-800 rounded"
                  />
                ) : typeof value === 'number' ? (
                  <input
                    type="number"
                    value={value as number}
                    onChange={(e) => handleUpdateNodeParameter(selectedNode.id, key, parseFloat(e.target.value))}
                    className="bg-gray-800 text-white text-sm px-2 py-1 rounded w-full"
                  />
                ) : (
                  <input
                    type="text"
                    value={String(value)}
                    onChange={(e) => handleUpdateNodeParameter(selectedNode.id, key, e.target.value)}
                    className="bg-gray-800 text-white text-sm px-2 py-1 rounded w-full"
                  />
                )}
              </div>
            ))}
          </div>
        </div>

        <div>
          <h4 className="text-white font-medium mb-2">Ports</h4>
          
          <div className="space-y-2">
            <div>
              <label className="text-gray-400 text-sm">Inputs</label>
              {selectedNode?.inputs.length ? (
                <div className="space-y-1">
                  {selectedNode.inputs.map((port: any) => (
                    <div key={port.id} className="bg-gray-800 rounded p-2">
                      <p className="text-white text-sm">{port.name}</p>
                      <p className="text-gray-500 text-xs">{port.dataType}</p>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-gray-500 text-sm">No inputs</p>
              )}
            </div>
            
            <div>
              <label className="text-gray-400 text-sm">Outputs</label>
              {selectedNode?.outputs.length ? (
                <div className="space-y-1">
                  {selectedNode.outputs.map((port: any) => (
                    <div key={port.id} className="bg-gray-800 rounded p-2">
                      <p className="text-white text-sm">{port.name}</p>
                      <p className="text-gray-500 text-xs">{port.dataType}</p>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-gray-500 text-sm">No outputs</p>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
