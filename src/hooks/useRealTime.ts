import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

/**
 * Real-time Technology Analysis:
 * 
 * Primary Choice: Server-Sent Events (SSE) for video editing
 * - Preview frame updates (server → client)
 * - Export progress monitoring
 * - Timeline state synchronization
 * - Media processing status updates
 * 
 * WebSocket Use Cases: Future collaboration features
 * - Multi-user editing sessions
 * - Real-time chat and annotations
 * - Remote control capabilities
 * 
 * Benefits: Lower resource usage, simpler implementation, automatic reconnection
 * 
 * Implementation: Hybrid SSE + Tauri commands approach
 * - Primary: WebSocket for bidirectional communication (future collaboration)
 * - Secondary: SSE for unidirectional updates (current video editing)
 * - Fallback: Tauri commands for desktop-native events (desktop environment)
 */

// Types for real-time events
export interface RealTimeEvent {
  type: 'preview_frame' | 'export_progress' | 'timeline_update' | 'media_status' | 'error';
  data: any;
  timestamp: number;
  source: 'sse' | 'websocket' | 'tauri';
}

export interface PreviewFrameEvent {
  frameId: string;
  frameData: string; // Base64 encoded frame
  timestamp: number;
  resolution: { width: number; height: number };
}

export interface ExportProgressEvent {
  exportId: string;
  progress: number;
  currentFrame: number;
  totalFrames: number;
  speed?: number;
  timeRemaining?: number;
  status: 'preparing' | 'rendering' | 'encoding' | 'finalizing' | 'completed' | 'failed';
}

export interface TimelineUpdateEvent {
  projectId: string;
  changes: {
    trackId?: string;
    clipId?: string;
    action: 'add' | 'remove' | 'update' | 'move';
    data: any;
  }[];
}

export interface MediaStatusEvent {
  mediaId: string;
  status: 'analyzing' | 'processing' | 'ready' | 'error';
  progress: number;
  metadata?: any;
  error?: string;
}

interface UseRealTimeOptions {
  projectId?: string;
  enabled?: boolean;
  fallbackToTauri?: boolean;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
}

interface UseRealTimeReturn {
  connected: boolean;
  connectionType: 'sse' | 'websocket' | 'tauri' | 'none';
  lastEvent: RealTimeEvent | null;
  error: string | null;
  reconnect: () => void;
  disconnect: () => void;
  sendEvent: (event: any) => void;
}

export const useRealTime = (options: UseRealTimeOptions = {}): UseRealTimeReturn => {
  const {
    projectId,
    enabled = true,
    fallbackToTauri = true,
    reconnectInterval = 5000,
    maxReconnectAttempts = 5,
  } = options;

  const [connected, setConnected] = useState(false);
  const [connectionType, setConnectionType] = useState<'sse' | 'websocket' | 'tauri' | 'none'>('none');
  const [lastEvent, setLastEvent] = useState<RealTimeEvent | null>(null);
  const [error, setError] = useState<string | null>(null);

  const eventSourceRef = useRef<EventSource | null>(null);
  const webSocketRef = useRef<WebSocket | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Cleanup function
  const cleanup = useCallback(() => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
    }
    if (webSocketRef.current) {
      webSocketRef.current.close();
      webSocketRef.current = null;
    }
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
    setConnected(false);
    setConnectionType('none');
  }, []);

  // Tauri fallback connection
  const connectTauri = useCallback(() => {
    if (!fallbackToTauri) {
      setError('Tauri fallback disabled');
      return;
    }

    try {
      setConnectionType('tauri');
      setConnected(true);
      setError(null);
      reconnectAttemptsRef.current = 0;

      // Set up Tauri event listeners for real-time updates
      // This would be implemented with Tauri's event system
      console.log('Connected via Tauri fallback');
    } catch (err) {
      setError(`Tauri connection failed: ${err}`);
      setConnected(false);
    }
  }, [fallbackToTauri]);

  // SSE Connection
  const connectSSE = useCallback(() => {
    if (!projectId) {
      setError('Project ID required for SSE connection');
      return;
    }

    try {
      const sseUrl = `/api/sse/${projectId}`;
      const eventSource = new EventSource(sseUrl);
      eventSourceRef.current = eventSource;

      eventSource.onopen = () => {
        setConnected(true);
        setConnectionType('sse');
        setError(null);
        reconnectAttemptsRef.current = 0;
        console.log('SSE connection established');
      };

      eventSource.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          const realTimeEvent: RealTimeEvent = {
            type: data.type,
            data: data.data,
            timestamp: data.timestamp || Date.now(),
            source: 'sse',
          };
          setLastEvent(realTimeEvent);
        } catch (err) {
          console.error('Failed to parse SSE event:', err);
        }
      };

      eventSource.onerror = (err) => {
        console.error('SSE connection error:', err);
        eventSource.close();
        
        if (reconnectAttemptsRef.current < maxReconnectAttempts) {
          reconnectAttemptsRef.current++;
          reconnectTimeoutRef.current = setTimeout(() => {
            connectSSE();
          }, reconnectInterval);
        } else {
          // Fallback to Tauri
          cleanup();
          connectTauri();
        }
      };
    } catch (err) {
      setError(`SSE connection failed: ${err}`);
      if (fallbackToTauri) {
        connectTauri();
      }
    }
  }, [projectId, maxReconnectAttempts, reconnectInterval, fallbackToTauri, cleanup, connectTauri]);

  // WebSocket Connection
  const connectWebSocket = useCallback(() => {
    if (!projectId) {
      setError('Project ID required for WebSocket connection');
      return;
    }

    try {
      const wsUrl = `ws://localhost:3000/ws/${projectId}`;
      const ws = new WebSocket(wsUrl);
      webSocketRef.current = ws;

      ws.onopen = () => {
        setConnected(true);
        setConnectionType('websocket');
        setError(null);
        reconnectAttemptsRef.current = 0;
        console.log('WebSocket connection established');
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          const realTimeEvent: RealTimeEvent = {
            type: data.type,
            data: data.data,
            timestamp: data.timestamp || Date.now(),
            source: 'websocket',
          };
          setLastEvent(realTimeEvent);
        } catch (err) {
          console.error('Failed to parse WebSocket event:', err);
        }
      };

      ws.onerror = (err) => {
        console.error('WebSocket connection error:', err);
      };

      ws.onclose = () => {
        setConnected(false);
        
        if (reconnectAttemptsRef.current < maxReconnectAttempts) {
          reconnectAttemptsRef.current++;
          reconnectTimeoutRef.current = setTimeout(() => {
            connectWebSocket();
          }, reconnectInterval);
        } else {
          // Fallback to Tauri
          cleanup();
          connectTauri();
        }
      };
    } catch (err) {
      setError(`WebSocket connection failed: ${err}`);
      if (fallbackToTauri) {
        connectTauri();
      }
    }
  }, [projectId, maxReconnectAttempts, reconnectInterval, fallbackToTauri, cleanup, connectTauri]);

  // Optimistic UI update with validation
  const updateWithOptimism = useCallback(async (
    action: () => Promise<any>,
    rollback: () => void,
    validation?: (result: any) => boolean
  ) => {
    try {
      // Execute the action optimistically
      const result = await action();
      
      // Validate the result if validation function provided
      if (validation && !validation(result)) {
        rollback();
        setError('Optimistic update validation failed');
        return null;
      }
      
      return result;
    } catch (err) {
      rollback();
      setError(`Optimistic update failed: ${err}`);
      return null;
    }
  }, []);

  // Send event via WebSocket or Tauri
  const sendEvent = useCallback((event: any) => {
    if (webSocketRef.current && webSocketRef.current.readyState === WebSocket.OPEN) {
      webSocketRef.current.send(JSON.stringify(event));
    } else if (connectionType === 'tauri') {
      // Fallback to Tauri command
      invoke('send_real_time_event', { event })
        .catch((err) => setError(`Failed to send event via Tauri: ${err}`));
    } else {
      setError('No active connection to send event');
    }
  }, [connectionType]);

  // Manual reconnect
  const reconnect = useCallback(() => {
    cleanup();
    reconnectAttemptsRef.current = 0;
    
    // Try WebSocket first, then SSE, then Tauri fallback
    connectWebSocket();
  }, [cleanup, connectWebSocket]);

  // Initial connection
  useEffect(() => {
    if (!enabled) return;

    // Try WebSocket first for bidirectional communication
    connectWebSocket();

    return () => {
      cleanup();
    };
  }, [enabled, connectWebSocket, cleanup]);

  return {
    connected,
    connectionType,
    lastEvent,
    error,
    reconnect,
    disconnect: cleanup,
    sendEvent,
  };
};

// Specific hooks for different real-time use cases

export const useRealTimePreview = (projectId?: string) => {
  const { lastEvent, connected, connectionType } = useRealTime({ projectId });
  const [currentFrame, setCurrentFrame] = useState<PreviewFrameEvent | null>(null);

  useEffect(() => {
    if (lastEvent?.type === 'preview_frame') {
      setCurrentFrame(lastEvent.data as PreviewFrameEvent);
    }
  }, [lastEvent]);

  return {
    currentFrame,
    connected,
    connectionType,
  };
};

export const useRealTimeExport = (projectId?: string) => {
  const { lastEvent, connected, connectionType } = useRealTime({ projectId });
  const [exportProgress, setExportProgress] = useState<ExportProgressEvent | null>(null);

  useEffect(() => {
    if (lastEvent?.type === 'export_progress') {
      setExportProgress(lastEvent.data as ExportProgressEvent);
    }
  }, [lastEvent]);

  return {
    exportProgress,
    connected,
    connectionType,
  };
};

export const useRealTimeTimeline = (projectId?: string) => {
  const { lastEvent, connected, connectionType, sendEvent } = useRealTime({ projectId });
  const [timelineUpdate, setTimelineUpdate] = useState<TimelineUpdateEvent | null>(null);

  useEffect(() => {
    if (lastEvent?.type === 'timeline_update') {
      setTimelineUpdate(lastEvent.data as TimelineUpdateEvent);
    }
  }, [lastEvent]);

  const sendTimelineUpdate = useCallback((changes: TimelineUpdateEvent['changes']) => {
    sendEvent({
      type: 'timeline_update',
      data: {
        projectId,
        changes,
      },
    });
  }, [projectId, sendEvent]);

  return {
    timelineUpdate,
    connected,
    connectionType,
    sendTimelineUpdate,
  };
};

export const useRealTimeMedia = (projectId?: string) => {
  const { lastEvent, connected, connectionType } = useRealTime({ projectId });
  const [mediaStatus, setMediaStatus] = useState<MediaStatusEvent | null>(null);

  useEffect(() => {
    if (lastEvent?.type === 'media_status') {
      setMediaStatus(lastEvent.data as MediaStatusEvent);
    }
  }, [lastEvent]);

  return {
    mediaStatus,
    connected,
    connectionType,
  };
};
