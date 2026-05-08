import React from 'react';
import { ScopeSettings } from './index';

interface ScopeControlsProps {
  settings: ScopeSettings;
  onSettingsChange: (newSettings: Partial<ScopeSettings>) => void;
}

export const ScopeControls: React.FC<ScopeControlsProps> = ({
  settings,
  onSettingsChange,
}) => {
  const handleToggleScope = (scope: keyof typeof settings.enabled) => {
    onSettingsChange({
      enabled: {
        ...settings.enabled,
        [scope]: !settings.enabled[scope],
      },
    });
  };

  const handleWaveformModeChange = (mode: ScopeSettings['waveform']['mode']) => {
    onSettingsChange({
      waveform: {
        ...settings.waveform,
        mode,
      },
    });
  };

  const handleVectorscopeSettingChange = (key: keyof ScopeSettings['vectorscope'], value: any) => {
    onSettingsChange({
      vectorscope: {
        ...settings.vectorscope,
        [key]: value,
      },
    });
  };

  const handleHistogramModeChange = (mode: ScopeSettings['histogram']['mode']) => {
    onSettingsChange({
      histogram: {
        ...settings.histogram,
        mode,
      },
    });
  };

  const handleUpdateRateChange = (rate: number) => {
    onSettingsChange({ updateRate: rate });
  };

  return (
    <div className="scope-controls">
      <div className="controls-section">
        <h5>Enabled Scopes</h5>
        <div className="scope-toggles">
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={settings.enabled.waveform}
              onChange={() => handleToggleScope('waveform')}
            />
            <span className="toggle-slider"></span>
            <span className="toggle-label">Waveform</span>
          </label>
          
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={settings.enabled.vectorscope}
              onChange={() => handleToggleScope('vectorscope')}
            />
            <span className="toggle-slider"></span>
            <span className="toggle-label">Vectorscope</span>
          </label>
          
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={settings.enabled.histogram}
              onChange={() => handleToggleScope('histogram')}
            />
            <span className="toggle-slider"></span>
            <span className="toggle-label">Histogram</span>
          </label>
        </div>
      </div>

      {settings.enabled.waveform && (
        <div className="controls-section">
          <h5>Waveform Settings</h5>
          <div className="control-group">
            <label>Mode:</label>
            <select
              value={settings.waveform.mode}
              onChange={(e) => handleWaveformModeChange(e.target.value as ScopeSettings['waveform']['mode'])}
            >
              <option value="luma">Luma</option>
              <option value="rgb">RGB</option>
              <option value="parade">Parade</option>
            </select>
          </div>
          
          <div className="control-group">
            <label>Intensity:</label>
            <input
              type="range"
              min="0.1"
              max="2.0"
              step="0.1"
              value={settings.waveform.intensity}
              onChange={(e) => onSettingsChange({
                waveform: { ...settings.waveform, intensity: parseFloat(e.target.value) }
              })}
            />
            <span className="range-value">{settings.waveform.intensity.toFixed(1)}</span>
          </div>
          
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={settings.waveform.grid}
              onChange={(e) => onSettingsChange({
                waveform: { ...settings.waveform, grid: e.target.checked }
              })}
            />
            <span className="toggle-slider"></span>
            <span className="toggle-label">Grid</span>
          </label>
        </div>
      )}

      {settings.enabled.vectorscope && (
        <div className="controls-section">
          <h5>Vectorscope Settings</h5>
          <div className="control-group">
            <label>Intensity:</label>
            <input
              type="range"
              min="0.1"
              max="2.0"
              step="0.1"
              value={settings.vectorscope.intensity}
              onChange={(e) => handleVectorscopeSettingChange('intensity', parseFloat(e.target.value))}
            />
            <span className="range-value">{settings.vectorscope.intensity.toFixed(1)}</span>
          </div>
          
          <div className="control-group">
            <label>Chroma Scale:</label>
            <input
              type="range"
              min="0.5"
              max="2.0"
              step="0.1"
              value={settings.vectorscope.chromaScale}
              onChange={(e) => handleVectorscopeSettingChange('chromaScale', parseFloat(e.target.value))}
            />
            <span className="range-value">{(settings.vectorscope.chromaScale * 100).toFixed(0)}%</span>
          </div>
          
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={settings.vectorscope.targets}
              onChange={(e) => handleVectorscopeSettingChange('targets', e.target.checked)}
            />
            <span className="toggle-slider"></span>
            <span className="toggle-label">Show Targets</span>
          </label>
          
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={settings.vectorscope.grid}
              onChange={(e) => handleVectorscopeSettingChange('grid', e.target.checked)}
            />
            <span className="toggle-slider"></span>
            <span className="toggle-label">Grid</span>
          </label>
        </div>
      )}

      {settings.enabled.histogram && (
        <div className="controls-section">
          <h5>Histogram Settings</h5>
          <div className="control-group">
            <label>Mode:</label>
            <select
              value={settings.histogram.mode}
              onChange={(e) => handleHistogramModeChange(e.target.value as ScopeSettings['histogram']['mode'])}
            >
              <option value="rgb">RGB</option>
              <option value="luma">Luma</option>
              <option value="all">All</option>
            </select>
          </div>
          
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={settings.histogram.logarithmic}
              onChange={(e) => onSettingsChange({
                histogram: { ...settings.histogram, logarithmic: e.target.checked }
              })}
            />
            <span className="toggle-slider"></span>
            <span className="toggle-label">Logarithmic</span>
          </label>
          
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={settings.histogram.grid}
              onChange={(e) => onSettingsChange({
                histogram: { ...settings.histogram, grid: e.target.checked }
              })}
            />
            <span className="toggle-slider"></span>
            <span className="toggle-label">Grid</span>
          </label>
        </div>
      )}

      <div className="controls-section">
        <h5>Performance</h5>
        <div className="control-group">
          <label>Update Rate:</label>
          <select
            value={settings.updateRate}
            onChange={(e) => handleUpdateRateChange(parseInt(e.target.value))}
          >
            <option value={15}>15 FPS</option>
            <option value={30}>30 FPS</option>
            <option value={60}>60 FPS</option>
            <option value={0}>Paused</option>
          </select>
        </div>
      </div>
    </div>
  );
};
