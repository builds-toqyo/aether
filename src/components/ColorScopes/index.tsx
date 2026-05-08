import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Play, Pause, RefreshCw, AlertTriangle } from 'lucide-react';
import { WaveformScope } from './WaveformScope';
import { Vectorscope } from './Vectorscope';
import { Histogram } from './Histogram';
import { ScopeControls } from './ScopeControls';
import { useTauriAPI } from '../../hooks/useTauriAPI';
import './ColorScopes.css';

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
      
      const response = await preview.get_scope_data();
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
    <div className="color-scopes">
      <div className="color-scopes-header">
        <h3>Color Scopes</h3>
        <div className="scope-controls">
          <button
            className={`pause-button ${isPaused ? 'paused' : ''}`}
            onClick={handlePauseToggle}
            title={isPaused ? 'Resume Updates' : 'Pause Updates'}
          >
            {isPaused ? <Play size={16} /> : <Pause size={16} />}
          </button>
          <button
            className="refresh-button"
            onClick={handleRefresh}
            title="Refresh Scopes"
            disabled={isLoading}
          >
            <RefreshCw size={16} className={isLoading ? 'spinning' : ''} />
          </button>
        </div>
      </div>

      <ScopeControls
        settings={settings}
        onSettingsChange={handleSettingsChange}
      />

      {error && (
        <div className="scope-error">
          <AlertTriangle size={16} className="error-icon" />
          {error}
        </div>
      )}

      <div className="scopes-container">
        {settings.enabled.waveform && (
          <div className="scope-panel">
            <WaveformScope
              data={scopeData.waveform}
              settings={settings.waveform}
              isLoading={isLoading}
            />
          </div>
        )}

        {settings.enabled.vectorscope && (
          <div className="scope-panel">
            <Vectorscope
              data={scopeData.vectorscope}
              settings={settings.vectorscope}
              isLoading={isLoading}
            />
          </div>
        )}

        {settings.enabled.histogram && (
          <div className="scope-panel">
            <Histogram
              data={scopeData.histogram}
              settings={settings.histogram}
              isLoading={isLoading}
            />
          </div>
        )}

        {!settings.enabled.waveform && !settings.enabled.vectorscope && !settings.enabled.histogram && (
          <div className="no-scopes-enabled">
            <p>No scopes enabled. Enable scopes in the controls above.</p>
          </div>
        )}
      </div>

      {isLoading && (
        <div className="scope-loading-overlay">
          <div className="loading-spinner"></div>
        </div>
      )}
    </div>
  );
};

export default ColorScopes;
