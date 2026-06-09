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
    <div className="bg-gray-900 border-t border-gray-700 p-4">
      <div className="mb-5">
        <h5 className="mb-3 text-sm font-semibold text-white border-b border-gray-700 pb-1.5">Enabled Scopes</h5>
        <div className="flex flex-wrap gap-4">
          <label className="flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={settings.enabled.waveform}
              onChange={() => handleToggleScope('waveform')}
              className="hidden"
            />
            <div className="relative w-11 h-6 bg-gray-600 border border-gray-500 rounded-full mr-2 transition-all duration-200">
              <div className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform duration-200 ${settings.enabled.waveform ? 'translate-x-5' : ''}`}></div>
            </div>
            <span className="text-sm text-gray-300">Waveform</span>
          </label>
          
          <label className="flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={settings.enabled.vectorscope}
              onChange={() => handleToggleScope('vectorscope')}
              className="hidden"
            />
            <div className="relative w-11 h-6 bg-gray-600 border border-gray-500 rounded-full mr-2 transition-all duration-200">
              <div className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform duration-200 ${settings.enabled.vectorscope ? 'translate-x-5' : ''}`}></div>
            </div>
            <span className="text-sm text-gray-300">Vectorscope</span>
          </label>
          
          <label className="flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={settings.enabled.histogram}
              onChange={() => handleToggleScope('histogram')}
              className="hidden"
            />
            <div className="relative w-11 h-6 bg-gray-600 border border-gray-500 rounded-full mr-2 transition-all duration-200">
              <div className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform duration-200 ${settings.enabled.histogram ? 'translate-x-5' : ''}`}></div>
            </div>
            <span className="text-sm text-gray-300">Histogram</span>
          </label>
        </div>
      </div>

      {settings.enabled.waveform && (
        <div className="mb-5">
          <h5 className="mb-3 text-sm font-semibold text-white border-b border-gray-700 pb-1.5">Waveform Settings</h5>
          <div className="flex items-center gap-3 mb-3">
            <label className="min-w-20 text-sm text-gray-300">Mode:</label>
            <select
              value={settings.waveform.mode}
              onChange={(e) => handleWaveformModeChange(e.target.value as ScopeSettings['waveform']['mode'])}
              className="flex-1 max-w-[200px] bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm text-white"
            >
              <option value="luma">Luma</option>
              <option value="rgb">RGB</option>
              <option value="parade">Parade</option>
            </select>
          </div>
          
          <div className="flex items-center gap-3 mb-3">
            <label className="min-w-20 text-sm text-gray-300">Intensity:</label>
            <input
              type="range"
              min="0.1"
              max="2.0"
              step="0.1"
              value={settings.waveform.intensity}
              onChange={(e) => onSettingsChange({
                waveform: { ...settings.waveform, intensity: parseFloat(e.target.value) }
              })}
              className="flex-1 max-w-[200px] h-1.5 bg-gray-700 rounded outline-none"
            />
            <span className="min-w-10 text-xs text-gray-600 text-right">{settings.waveform.intensity.toFixed(1)}</span>
          </div>
          
          <label className="flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={settings.waveform.grid}
              onChange={(e) => onSettingsChange({
                waveform: { ...settings.waveform, grid: e.target.checked }
              })}
              className="hidden"
            />
            <div className="relative w-11 h-6 bg-gray-600 border border-gray-500 rounded-full mr-2 transition-all duration-200">
              <div className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform duration-200 ${settings.waveform.grid ? 'translate-x-5' : ''}`}></div>
            </div>
            <span className="text-sm text-gray-300">Grid</span>
          </label>
        </div>
      )}

      {settings.enabled.vectorscope && (
        <div className="mb-5">
          <h5 className="mb-3 text-sm font-semibold text-white border-b border-gray-700 pb-1.5">Vectorscope Settings</h5>
          <div className="flex items-center gap-3 mb-3">
            <label className="min-w-20 text-sm text-gray-300">Intensity:</label>
            <input
              type="range"
              min="0.1"
              max="2.0"
              step="0.1"
              value={settings.vectorscope.intensity}
              onChange={(e) => handleVectorscopeSettingChange('intensity', parseFloat(e.target.value))}
              className="flex-1 max-w-[200px] h-1.5 bg-gray-700 rounded outline-none"
            />
            <span className="min-w-10 text-xs text-gray-600 text-right">{settings.vectorscope.intensity.toFixed(1)}</span>
          </div>
          
          <div className="flex items-center gap-3 mb-3">
            <label className="min-w-20 text-sm text-gray-300">Chroma Scale:</label>
            <input
              type="range"
              min="0.5"
              max="2.0"
              step="0.1"
              value={settings.vectorscope.chromaScale}
              onChange={(e) => handleVectorscopeSettingChange('chromaScale', parseFloat(e.target.value))}
              className="flex-1 max-w-[200px] h-1.5 bg-gray-700 rounded outline-none"
            />
            <span className="min-w-10 text-xs text-gray-600 text-right">{(settings.vectorscope.chromaScale * 100).toFixed(0)}%</span>
          </div>
          
          <div className="flex flex-wrap gap-4 mb-3">
            <label className="flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.vectorscope.targets}
                onChange={(e) => handleVectorscopeSettingChange('targets', e.target.checked)}
                className="hidden"
              />
              <div className="relative w-11 h-6 bg-gray-600 border border-gray-500 rounded-full mr-2 transition-all duration-200">
                <div className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform duration-200 ${settings.vectorscope.targets ? 'translate-x-5' : ''}`}></div>
              </div>
              <span className="text-sm text-gray-300">Show Targets</span>
            </label>
            
            <label className="flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.vectorscope.grid}
                onChange={(e) => handleVectorscopeSettingChange('grid', e.target.checked)}
                className="hidden"
              />
              <div className="relative w-11 h-6 bg-gray-600 border border-gray-500 rounded-full mr-2 transition-all duration-200">
                <div className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform duration-200 ${settings.vectorscope.grid ? 'translate-x-5' : ''}`}></div>
              </div>
              <span className="text-sm text-gray-300">Grid</span>
            </label>
          </div>
        </div>
      )}

      {settings.enabled.histogram && (
        <div className="mb-5">
          <h5 className="mb-3 text-sm font-semibold text-white border-b border-gray-700 pb-1.5">Histogram Settings</h5>
          <div className="flex items-center gap-3 mb-3">
            <label className="min-w-20 text-sm text-gray-300">Mode:</label>
            <select
              value={settings.histogram.mode}
              onChange={(e) => handleHistogramModeChange(e.target.value as ScopeSettings['histogram']['mode'])}
              className="flex-1 max-w-[200px] bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm text-white"
            >
              <option value="rgb">RGB</option>
              <option value="luma">Luma</option>
              <option value="all">All</option>
            </select>
          </div>
          
          <div className="flex flex-wrap gap-4 mb-3">
            <label className="flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.histogram.logarithmic}
                onChange={(e) => onSettingsChange({
                  histogram: { ...settings.histogram, logarithmic: e.target.checked }
                })}
                className="hidden"
              />
              <div className="relative w-11 h-6 bg-gray-600 border border-gray-500 rounded-full mr-2 transition-all duration-200">
                <div className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform duration-200 ${settings.histogram.logarithmic ? 'translate-x-5' : ''}`}></div>
              </div>
              <span className="text-sm text-gray-300">Logarithmic</span>
            </label>
            
            <label className="flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.histogram.grid}
                onChange={(e) => onSettingsChange({
                  histogram: { ...settings.histogram, grid: e.target.checked }
                })}
                className="hidden"
              />
              <div className="relative w-11 h-6 bg-gray-600 border border-gray-500 rounded-full mr-2 transition-all duration-200">
                <div className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform duration-200 ${settings.histogram.grid ? 'translate-x-5' : ''}`}></div>
              </div>
              <span className="text-sm text-gray-300">Grid</span>
            </label>
          </div>
        </div>
      )}

      <div className="mb-5">
        <h5 className="mb-3 text-sm font-semibold text-white border-b border-gray-700 pb-1.5">Performance</h5>
        <div className="flex items-center gap-3">
          <label className="min-w-20 text-sm text-gray-300">Update Rate:</label>
          <select
            value={settings.updateRate}
            onChange={(e) => handleUpdateRateChange(parseInt(e.target.value))}
            className="flex-1 max-w-[200px] bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm text-white"
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
