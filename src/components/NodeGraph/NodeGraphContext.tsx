'use client';

import React, { createContext, useContext, useState } from 'react';

// Types
interface Node {
  id: string;
  type: string;
  name: string;
  position: { x: number; y: number };
  inputs: NodePort[];
  outputs: NodePort[];
  parameters: Record<string, any>;
}

interface NodePort {
  id: string;
  name: string;
  type: 'input' | 'output';
  dataType: string;
  position: { x: number; y: number };
}

interface Connection {
  id: string;
  sourceNodeId: string;
  sourcePortId: string;
  targetNodeId: string;
  targetPortId: string;
}

interface Graph {
  id: string;
  name: string;
  nodes: Node[];
  connections: Connection[];
}

interface Viewport {
  x: number;
  y: number;
  zoom: number;
}

interface NodeGraphContextType {
  graph: Graph;
  selectedNode: Node | null;
  selectedConnection: Connection | null;
  isConnecting: boolean;
  connectionStart: { nodeId: string; portId: string } | null;
  viewport: Viewport;
  isPanning: boolean;
  canvasRef: React.RefObject<HTMLDivElement | null>;
  svgRef: React.RefObject<SVGSVGElement | null>;
  setSelectedNode: (node: Node | null) => void;
  setSelectedConnection: (connection: Connection | null) => void;
  setIsConnecting: (connecting: boolean) => void;
  setConnectionStart: (start: { nodeId: string; portId: string } | null) => void;
  setViewport: (viewport: Viewport | ((prev: Viewport) => Viewport)) => void;
  handleCreateNode: (nodeType: string, position: { x: number; y: number }) => Promise<void>;
  handleDeleteNode: (nodeId: string) => Promise<void>;
  handleCreateConnection: (sourceNodeId: string, sourcePortId: string, targetNodeId: string, targetPortId: string) => Promise<void>;
  handleDeleteConnection: (connectionId: string) => Promise<void>;
  handleUpdateNodeParameter: (nodeId: string, parameterName: string, value: any) => Promise<void>;
  handleExecuteGraph: () => Promise<any>;
  handleCanvasMouseDown: (e: React.MouseEvent<HTMLDivElement>) => void;
  handleCanvasMouseMove: (e: React.MouseEvent<HTMLDivElement>) => void;
  handleCanvasMouseUp: () => void;
  handleWheel: (e: React.WheelEvent<HTMLDivElement>) => void;
}

const NodeGraphContext = createContext<NodeGraphContextType | undefined>(undefined);

export const useNodeGraph = () => {
  const context = useContext(NodeGraphContext);
  if (!context) {
    throw new Error('useNodeGraph must be used within a NodeGraphProvider');
  }
  return context;
};

interface NodeGraphProviderProps {
  children: React.ReactNode;
  value: NodeGraphContextType;
}

export const NodeGraphProvider: React.FC<NodeGraphProviderProps> = ({ children, value }) => {
  return (
    <NodeGraphContext.Provider value={value}>
      {children}
    </NodeGraphContext.Provider>
  );
};

export type {
  Node,
  NodePort,
  Connection,
  Graph,
  Viewport,
  NodeGraphContextType
};
