'use client';

import React from 'react';
import { useNodeGraph } from './NodeGraphContext';

export const NodeGraphSidebar: React.FC = () => {
  const { selectedNode, selectedConnection, handleUpdateNodeParameter, graph } = useNodeGraph();

  if (!selectedNode && !selectedConnection) {
    return (
      <div className="w-80 bg-[#1a1a1a] border-l border-[#222] p-4">
        <div className="text-[#555] text-center mt-8">
          <svg className="w-16 h-16 mx-auto mb-4 text-[#333]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p className="text-sm">Select a node or connection to view properties</p>
        </div>
      </div>
    );
  }

  if (selectedConnection) {
    return (
      <div className="w-80 bg-[#1a1a1a] border-l border-[#222] p-4">
        <h3 className="text-[#e0e0e0] font-semibold mb-4 text-sm">Connection Properties</h3>
        
        <div className="space-y-4">
          <div>
            <label className="text-[#777] text-xs">Connection ID</label>
            <p className="text-[#ccc] text-sm font-mono break-all">{selectedConnection.id}</p>
          </div>
          
          <div>
            <label className="text-[#777] text-xs">Source</label>
            <p className="text-[#ccc] text-sm">
              {graph.nodes.find(n => n.id === selectedConnection.sourceNodeId)?.name || 'Unknown'}
            </p>
            <p className="text-[#555] text-xs">
              Port: {selectedConnection.sourcePortId}
            </p>
          </div>
          
          <div>
            <label className="text-[#777] text-xs">Target</label>
            <p className="text-[#ccc] text-sm">
              {graph.nodes.find(n => n.id === selectedConnection.targetNodeId)?.name || 'Unknown'}
            </p>
            <p className="text-[#555] text-xs">
              Port: {selectedConnection.targetPortId}
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="w-80 bg-[#1a1a1a] border-l border-[#222] p-4">
      <h3 className="text-[#e0e0e0] font-semibold mb-4 text-sm">Node Properties</h3>

      <div className="space-y-4">
        <div>
          <label className="text-[#777] text-xs">Node Type</label>
          <p className="text-[#ccc] text-sm capitalize">{selectedNode?.type}</p>
        </div>

        <div>
          <label className="text-[#777] text-xs">Node ID</label>
          <p className="text-[#ccc] text-sm font-mono break-all">{selectedNode?.id}</p>
        </div>

        <div>
          <label className="text-[#777] text-xs">Position</label>
          <div className="flex space-x-2">
            <div>
              <label className="text-[#555] text-xs">X</label>
              <input
                type="number"
                value={selectedNode?.position?.x || 0}
                onChange={(e) => {
                  // Handle position update
                }}
                className="bg-[#252525] border border-[#333] text-[#ccc] text-sm px-2 py-1 rounded w-20 outline-none focus:border-blue-500"
              />
            </div>
            <div>
              <label className="text-[#555] text-xs">Y</label>
              <input
                type="number"
                value={selectedNode?.position?.y || 0}
                onChange={(e) => {
                  // Handle position update
                }}
                className="bg-[#252525] border border-[#333] text-[#ccc] text-sm px-2 py-1 rounded w-20 outline-none focus:border-blue-500"
              />
            </div>
          </div>
        </div>

        <div>
          <h4 className="text-[#ccc] font-medium mb-2 text-sm">Parameters</h4>
          <div className="space-y-2">
            {Object.entries(selectedNode?.parameters ?? {}).map(([key, value]) => (
              <div key={key}>
                <label className="text-[#777] text-xs capitalize">{key}</label>
                {typeof value === 'boolean' ? (
                  <input
                    type="checkbox"
                    checked={value as boolean}
                    onChange={(e) => selectedNode && handleUpdateNodeParameter(selectedNode.id, key, e.target.checked)}
                    className="accent-blue-500 rounded"
                  />
                ) : typeof value === 'number' ? (
                  <input
                    type="number"
                    value={value as number}
                    onChange={(e) => selectedNode && handleUpdateNodeParameter(selectedNode.id, key, parseFloat(e.target.value))}
                    className="bg-[#252525] border border-[#333] text-[#ccc] text-sm px-2 py-1 rounded w-full outline-none focus:border-blue-500"
                  />
                ) : (
                  <input
                    type="text"
                    value={String(value)}
                    onChange={(e) => selectedNode && handleUpdateNodeParameter(selectedNode.id, key, e.target.value)}
                    className="bg-[#252525] border border-[#333] text-[#ccc] text-sm px-2 py-1 rounded w-full outline-none focus:border-blue-500"
                  />
                )}
              </div>
            ))}
          </div>
        </div>

        <div>
          <h4 className="text-[#ccc] font-medium mb-2 text-sm">Ports</h4>

          <div className="space-y-2">
            <div>
              <label className="text-[#777] text-xs">Inputs</label>
              {(selectedNode?.inputs ?? []).length ? (
                <div className="space-y-1">
                  {(selectedNode?.inputs ?? []).map((port: any) => (
                    <div key={port.id} className="bg-[#252525] border border-[#333] rounded p-2">
                      <p className="text-[#ccc] text-sm">{port.name}</p>
                      <p className="text-[#555] text-xs">{port.dataType}</p>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-[#555] text-sm">No inputs</p>
              )}
            </div>

            <div>
              <label className="text-[#777] text-xs">Outputs</label>
              {(selectedNode?.outputs ?? []).length ? (
                <div className="space-y-1">
                  {(selectedNode?.outputs ?? []).map((port: any) => (
                    <div key={port.id} className="bg-[#252525] border border-[#333] rounded p-2">
                      <p className="text-[#ccc] text-sm">{port.name}</p>
                      <p className="text-[#555] text-xs">{port.dataType}</p>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-[#555] text-sm">No outputs</p>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
