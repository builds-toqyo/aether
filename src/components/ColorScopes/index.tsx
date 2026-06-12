import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Play, Pause, RefreshCw, AlertTriangle } from 'lucide-react';
import { WaveformScope } from './WaveformScope';
import { Vectorscope } from './Vectorscope';
import { Histogram } from './Histogram';
import { ScopeControls } from './ScopeControls';
import { useTauriAPI } from '../../hooks/useTauriAPI';

export interface ScopeData {
  waveform?: {
    luma: number[][];
    rgb: {
      red: number[][];
      green: number[][];
      blue: number[][];
    };
  };
  vectorscope?: {
    points: Array<{ x: number; y: number; intensity: number }>;
    targets: Array<{
      name: string;
      x: number;
      y: number;
      color: string;
      radius: number;
    }>;
  };
  histogram?: {
    red: number[];
    green: number[];
    blue: number[];
    luma: number[];
  };
}

export interface ScopeSettings {
  enabled: {
    waveform: boolean;
    vectorscope: boolean;
    histogram: boolean;
  };
  waveform: {
    mode: 'luma' | 'rgb' | 'parade';
    intensity: number;
    grid: boolean;
  };
  vectorscope: {
    intensity: number;
    targets: boolean;
    grid: boolean;
    chromaScale: number;
  };
  histogram: {
    mode: 'rgb' | 'luma' | 'all';
    logarithmic: boolean;
    grid: boolean;
  };
  updateRate: number; // updates per second
}

const defaultSettings: ScopeSettings = {
  enabled: {
    waveform: true,
    vectorscope: true,
    histogram: false,
  },
  waveform: {
    mode: 'luma',
    intensity: 1.0,
    grid: true,
  },
  vectorscope: {
    intensity: 1.0,
    targets: true,
    grid: true,
    chromaScale: 1.0,
  },
  histogram: {
    mode: 'rgb',
    logarithmic: false,
    grid: true,
  },
  updateRate: 30,
};

export const ColorScopes: React.FC = () => {
  const [scopeData, setScopeData] = useState<ScopeData>({});
  const [settings, setSettings] = useState<ScopeSettings>(defaultSettings);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isPaused, setIsPaused] = useState(false);
  
  const { preview } = useTauriAPI();
  const updateIntervalRef = useRef<NodeJS.Timeout | null>(null);

  // Fetch scope data from the backend
  const fetchScopeData = useCallback(async () => {
    if (isPaused) return;
    
    try {
      setIsLoading(true);
      setError(null);
      
      const response = await preview.get_performance_stats();
      setScopeData(response.data);
    } catch (err) {
      console.error('Failed to fetch scope data:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch scope data');
    } finally {
      setIsLoading(false);
    }
  }, [preview, isPaused]);

  // Set up automatic updates
  useEffect(() => {
    if (updateIntervalRef.current) {
      clearInterval(updateIntervalRef.current);
    }

    if (!isPaused && settings.updateRate > 0) {
      updateIntervalRef.current = setInterval(fetchScopeData, 1000 / settings.updateRate);
    }

    return () => {
      if (updateIntervalRef.current) {
        clearInterval(updateIntervalRef.current);
      }
    };
  }, [fetchScopeData, settings.updateRate, isPaused]);

  // Initial data fetch
  useEffect(() => {
    fetchScopeData();
  }, [fetchScopeData]);

  const handleSettingsChange = useCallback((newSettings: Partial<ScopeSettings>) => {
    setSettings(prev => ({ ...prev, ...newSettings }));
  }, []);

  const handlePauseToggle = useCallback(() => {
    setIsPaused(prev => !prev);
  }, []);

  const handleRefresh = useCallback(() => {
    fetchScopeData();
  }, [fetchScopeData]);

  return (
    <div className="flex flex-col bg-gray-900 border border-gray-700 rounded-lg overflow-hidden font-sans text-white">
      <div className="flex justify-between items-center p-3 bg-gray-800 border-b border-gray-700">
        <h3 className="text-base font-semibold text-white">Color Scopes</h3>
        <div className="flex gap-2">
          <button
            className={`flex items-center justify-center min-w-8 h-8 bg-gray-600 border border-gray-500 rounded text-white cursor-pointer transition-all duration-200 hover:bg-gray-500 hover:border-gray-400 ${isPaused ? 'bg-green-600 border-green-500' : ''}`}
            onClick={handlePauseToggle}
            title={isPaused ? 'Resume Updates' : 'Pause Updates'}
          >
            {isPaused ? <Play size={16} /> : <Pause size={16} />}
          </button>
          <button
            className="flex items-center justify-center min-w-8 h-8 bg-gray-600 border border-gray-500 rounded text-white cursor-pointer transition-all duration-200 hover:bg-gray-500 hover:border-gray-400 disabled:opacity-50 disabled:cursor-not-allowed"
            onClick={handleRefresh}
            title="Refresh Scopes"
            disabled={isLoading}
          >
            <RefreshCw size={16} className={isLoading ? 'animate-spin' : ''} />
          </button>
        </div>
      </div>

      <ScopeControls
        settings={settings}
        onSettingsChange={handleSettingsChange}
      />

      {error && (
        <div className="flex items-center gap-2 p-3 bg-red-600 text-white text-sm">
          <AlertTriangle size={16} className="flex-shrink-0" />
          {error}
        </div>
      )}

      <div className="flex flex-wrap gap-4 p-4 min-h-[200px]">
        {settings.enabled.waveform && (
          <div className="flex-1 min-w-[300px] bg-gray-800 border border-gray-700 rounded-md overflow-hidden">
            <WaveformScope
              data={scopeData.waveform}
              settings={settings.waveform}
              isLoading={isLoading}
            />
          </div>
        )}
        {settings.enabled.vectorscope && (
          <div className="flex-1 min-w-[300px] bg-gray-800 border border-gray-700 rounded-md overflow-hidden">
            <Vectorscope
              data={scopeData.vectorscope}
              settings={settings.vectorscope}
              isLoading={isLoading}
            />
          </div>
        )}
        {settings.enabled.histogram && (
          <div className="flex-1 min-w-[300px] bg-gray-800 border border-gray-700 rounded-md overflow-hidden">
            <Histogram
              data={scopeData.histogram}
              settings={settings.histogram}
              isLoading={isLoading}
            />
          </div>
        )}
        {!settings.enabled.waveform && !settings.enabled.vectorscope && !settings.enabled.histogram && (
          <div className="flex items-center justify-center p-10 text-gray-500 italic">
            No scopes enabled. Enable scopes using the controls below.
          </div>
        )}
      </div>
    </div>
  );
};

export default ColorScopes;
