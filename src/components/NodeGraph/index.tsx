'use client';

import React, { useState, useRef, useCallback, useEffect } from 'react';
import { NodeGraphCanvas } from './NodeGraphCanvas';
import { NodeGraphToolbar } from './NodeGraphToolbar';
import { NodeGraphSidebar } from './NodeGraphSidebar';
import { NodeGraphProvider, useNodeGraph } from './NodeGraphContext';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

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

export const NodeGraphEditor: React.FC = () => {
  const [graph, setGraph] = useState<Graph>({
    id: '',
    name: 'Untitled Graph',
    nodes: [],
    connections: []
  });
  
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);
  const [selectedConnection, setSelectedConnection] = useState<Connection | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);
  const [connectionStart, setConnectionStart] = useState<{ nodeId: string; portId: string } | null>(null);
  const [viewport, setViewport] = useState({ x: 0, y: 0, zoom: 1 });
  const [isPanning, setIsPanning] = useState(false);
  const [panStart, setPanStart] = useState({ x: 0, y: 0 });
  
  const canvasRef = useRef<HTMLDivElement>(null);
  const svgRef = useRef<SVGSVGElement>(null);

  // Initialize graph
  useEffect(() => {
    const initializeGraph = async () => {
      try {
        const graphInfo = await invoke<Graph>('get_graph_info');
        console.log('Graph initialized:', graphInfo);
        
        // Load existing graph if available
        if (graphInfo && graphInfo.nodes) {
          setGraph(graphInfo);
        }
      } catch (error) {
        console.error('Failed to initialize graph:', error);
        // Start with empty graph if backend fails
        console.log('Starting with empty graph');
      }
    };
    
    initializeGraph();
  }, []);

  // Handle node creation
  const handleCreateNode = useCallback(async (nodeType: string, position: { x: number; y: number }) => {
    try {
      // Call backend to create node
      const newNode = await invoke<Node>('create_node', {
        nodeType,
        position,
        name: `${nodeType}_${Date.now()}`
      });
      
      // Update local state with the created node
      setGraph(prev => ({
        ...prev,
        nodes: [...prev.nodes, newNode]
      }));
      
      setSelectedNode(newNode);
      console.log('Node created successfully:', newNode);
    } catch (error) {
      console.error('Failed to create node:', error);
      // Fallback to mock implementation if backend fails
      const nodeId = `node_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      
      const mockNode: Node = {
        id: nodeId,
        type: nodeType,
        name: `${nodeType}_${nodeId}`,
        position,
        inputs: [],
        outputs: [],
        parameters: {}
      };
      
      setGraph(prev => ({
        ...prev,
        nodes: [...prev.nodes, mockNode]
      }));
      
      setSelectedNode(mockNode);
    }
  }, []);

  // Handle node deletion
  const handleDeleteNode = useCallback(async (nodeId: string) => {
    try {
      // Call backend to delete node
      await invoke('delete_node', { nodeId });
      
      // Update local state
      setGraph(prev => ({
        ...prev,
        nodes: prev.nodes.filter(n => n.id !== nodeId),
        connections: prev.connections.filter(c => c.sourceNodeId !== nodeId && c.targetNodeId !== nodeId)
      }));
      
      if (selectedNode?.id === nodeId) {
        setSelectedNode(null);
      }
      
      console.log('Node deleted successfully:', nodeId);
    } catch (error) {
      console.error('Failed to delete node:', error);
      // Fallback to local state update if backend fails
      setGraph(prev => ({
        ...prev,
        nodes: prev.nodes.filter(n => n.id !== nodeId),
        connections: prev.connections.filter(c => c.sourceNodeId !== nodeId && c.targetNodeId !== nodeId)
      }));
      
      if (selectedNode?.id === nodeId) {
        setSelectedNode(null);
      }
    }
  }, [selectedNode]);

  // Handle connection creation
  const handleCreateConnection = useCallback(async (sourceNodeId: string, sourcePortId: string, targetNodeId: string, targetPortId: string) => {
    try {
      // Call backend to create connection
      const newConnection = await invoke<Connection>('connect_nodes', {
        sourceNodeId,
        sourcePortId,
        targetNodeId,
        targetPortId
      });
      
      // Update local state with the created connection
      setGraph(prev => ({
        ...prev,
        connections: [...prev.connections, newConnection]
      }));
      
      console.log('Connection created successfully:', newConnection);
    } catch (error) {
      console.error('Failed to create connection:', error);
      // Fallback to mock implementation if backend fails
      const connectionId = `conn_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      
      const mockConnection: Connection = {
        id: connectionId,
        sourceNodeId,
        sourcePortId,
        targetNodeId,
        targetPortId
      };
      
      setGraph(prev => ({
        ...prev,
        connections: [...prev.connections, mockConnection]
      }));
    }
  }, []);

  // Handle connection deletion
  const handleDeleteConnection = useCallback(async (connectionId: string) => {
    try {
      // Call backend to delete connection
      await invoke('disconnect_nodes', { connectionId });
      
      // Update local state
      setGraph(prev => ({
        ...prev,
        connections: prev.connections.filter(c => c.id !== connectionId)
      }));
      
      if (selectedConnection?.id === connectionId) {
        setSelectedConnection(null);
      }
      
      console.log('Connection deleted successfully:', connectionId);
    } catch (error) {
      console.error('Failed to delete connection:', error);
      // Fallback to local state update if backend fails
      setGraph(prev => ({
        ...prev,
        connections: prev.connections.filter(c => c.id !== connectionId)
      }));
      
      if (selectedConnection?.id === connectionId) {
        setSelectedConnection(null);
      }
    }
  }, [selectedConnection]);

  // Handle node parameter updates
  const handleUpdateNodeParameter = useCallback(async (nodeId: string, parameterName: string, value: any) => {
    try {
      // Update local state immediately for responsiveness
      setGraph(prev => ({
        ...prev,
        nodes: prev.nodes.map(node => 
          node.id === nodeId 
            ? { ...node, parameters: { ...node.parameters, [parameterName]: value } }
            : node
        )
      }));
      
      // Backend parameter update would go here when implemented
      console.log('Node parameter updated:', nodeId, parameterName, value);
    } catch (error) {
      console.error('Failed to update node parameter:', error);
    }
  }, []);

  // Handle graph execution
  const handleExecuteGraph = useCallback(async () => {
    try {
      console.log('Executing graph...');
      const result = await invoke('execute_graph', { graphId: graph.id });
      console.log('Graph execution result:', result);
      return result;
    } catch (error) {
      console.error('Failed to execute graph:', error);
      throw error;
    }
  }, [graph.id]);

  // Handle canvas mouse events
  const handleCanvasMouseDown = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    if (e.button === 1 || (e.button === 0 && e.shiftKey)) {
      setIsPanning(true);
      setPanStart({ x: e.clientX - viewport.x, y: e.clientY - viewport.y });
      e.preventDefault();
    } else {
      setSelectedNode(null);
      setSelectedConnection(null);
    }
  }, [viewport]);

  const handleCanvasMouseMove = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    if (isPanning) {
      setViewport(prev => ({
        ...prev,
        x: e.clientX - panStart.x,
        y: e.clientY - panStart.y
      }));
    }
  }, [isPanning, panStart]);

  const handleCanvasMouseUp = useCallback(() => {
    setIsPanning(false);
  }, []);

  // Handle wheel zoom
  const handleWheel = useCallback((e: React.WheelEvent<HTMLDivElement>) => {
    if (e.ctrlKey) {
      e.preventDefault();
      const scaleFactor = e.deltaY > 0 ? 0.9 : 1.1;
      const newZoom = Math.max(0.1, Math.min(5, viewport.zoom * scaleFactor));
      
      // Zoom towards mouse position
      const rect = canvasRef.current?.getBoundingClientRect();
      if (rect) {
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;
        
        const worldX = (mouseX - viewport.x) / viewport.zoom;
        const worldY = (mouseY - viewport.y) / viewport.zoom;
        
        const newViewportX = mouseX - worldX * newZoom;
        const newViewportY = mouseY - worldY * newZoom;
        
        setViewport({
          x: newViewportX,
          y: newViewportY,
          zoom: newZoom
        });
      }
    }
  }, [viewport]);

  const contextValue = {
    graph,
    selectedNode,
    selectedConnection,
    isConnecting,
    connectionStart,
    viewport,
    isPanning,
    canvasRef,
    svgRef,
    setSelectedNode,
    setSelectedConnection,
    setIsConnecting,
    setConnectionStart,
    setViewport,
    handleCreateNode,
    handleDeleteNode,
    handleCreateConnection,
    handleDeleteConnection,
    handleUpdateNodeParameter,
    handleExecuteGraph,
    handleCanvasMouseDown,
    handleCanvasMouseMove,
    handleCanvasMouseUp,
    handleWheel
  };

  return (
    <NodeGraphProvider value={contextValue}>
      <div className="flex h-full bg-[#0d0d0d]">
        <NodeGraphToolbar />
        
        <div className="flex-1 relative overflow-hidden">
          <NodeGraphCanvas />
        </div>
        
        <NodeGraphSidebar />
      </div>
    </NodeGraphProvider>
  );
};

export default NodeGraphEditor;
