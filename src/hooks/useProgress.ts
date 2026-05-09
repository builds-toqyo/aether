import { useState, useCallback, useRef } from 'react';

export interface ProgressState {
  isRunning: boolean;
  progress: number;
  status: 'idle' | 'preparing' | 'processing' | 'finalizing' | 'completed' | 'error';
  currentFile?: string;
  currentFrame?: number;
  totalFrames?: number;
  speed?: number;
  timeRemaining?: number;
  error?: string;
}

export const useProgress = () => {
  const [state, setState] = useState<ProgressState>({
    isRunning: false,
    progress: 0,
    status: 'idle',
  });

  const intervalRef = useRef<NodeJS.Timeout | null>(null);

  const start = useCallback((initialStatus: ProgressState['status'] = 'preparing') => {
    setState({
      isRunning: true,
      progress: 0,
      status: initialStatus,
    });
  }, []);

  const update = useCallback((updates: Partial<ProgressState>) => {
    setState(prev => ({ ...prev, ...updates }));
  }, []);

  const increment = useCallback((amount: number = 1) => {
    setState(prev => ({
      ...prev,
      progress: Math.min(100, prev.progress + amount),
    }));
  }, []);

  const complete = useCallback(() => {
    setState({
      isRunning: false,
      progress: 100,
      status: 'completed',
    });
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  }, []);

  const error = useCallback((errorMessage: string) => {
    setState({
      isRunning: false,
      progress: 0,
      status: 'error',
      error: errorMessage,
    });
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  }, []);

  const reset = useCallback(() => {
    setState({
      isRunning: false,
      progress: 0,
      status: 'idle',
    });
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  }, []);

  const simulateProgress = useCallback((duration: number = 5000, onUpdate?: (progress: number) => void) => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
    }

    const interval = 50; // Update every 50ms
    const steps = duration / interval;
    let currentStep = 0;

    intervalRef.current = setInterval(() => {
      currentStep++;
      const progress = (currentStep / steps) * 100;
      
      setState(prev => ({
        ...prev,
        progress: Math.min(100, progress),
        speed: Math.random() * 30 + 10,
        timeRemaining: Math.max(0, (duration - currentStep * interval) / 1000),
      }));

      if (onUpdate) {
        onUpdate(progress);
      }

      if (currentStep >= steps) {
        complete();
      }
    }, interval);
  }, [complete]);

  return {
    ...state,
    start,
    update,
    increment,
    complete,
    error,
    reset,
    simulateProgress,
  };
};
