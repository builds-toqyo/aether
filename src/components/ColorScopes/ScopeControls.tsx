import React from 'react';
import { ScopeSettings } from './index';

interface ScopeControlsProps {
  settings: ScopeSettings;
  onSettingsChange: (newSettings: Partial<ScopeSettings>) => void;
}

const Toggle: React.FC<{
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
}> = ({ checked, onChange, label }) => (
  <label className="flex items-center cursor-pointer">
    <input
      type="checkbox"
      checked={checked}
      onChange={(e) => onChange(e.target.checked)}
      className="hidden"
    />
    <div
      className={`relative w-9 h-5 rounded-full mr-2 transition-colors duration-200 border ${
        checked ? 'bg-blue-500 border-blue-500' : 'bg-[#333] border-[#444]'
      }`}
    >
      <div
        className={`absolute top-0.5 w-4 h-4 rounded-full transition-transform duration-200 ${
          checked ? 'translate-x-[18px] bg-white' : 'translate-x-0.5 bg-[#999]'
        }`}
      ></div>
    </div>
    <span className="text-[12px] text-[#ccc]">{label}</span>
  </label>
);

const Slider: React.FC<{
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  display: string;
  onChange: (value: number) => void;
}> = ({ label, value, min, max, step, display, onChange }) => (
  <div className="flex items-center gap-3 mb-2.5">
    <label className="min-w-24 text-[12px] text-[#999]">{label}:</label>
    <input
      type="range"
      min={min}
      max={max}
      step={step}
      value={value}
      onChange={(e) => onChange(parseFloat(e.target.value))}
      className="flex-1 max-w-[200px] h-1.5 accent-blue-500 bg-[#333] rounded outline-none"
    />
    <span className="min-w-10 text-[11px] text-blue-400 text-right tabular-nums">{display}</span>
  </div>
);

const selectClass =
  'flex-1 max-w-[200px] bg-[#252525] border border-[#333] rounded px-2 py-1 text-[12px] text-white outline-none focus:border-blue-500 transition-colors';

const sectionTitle =
  'mb-2.5 text-[11px] font-semibold uppercase tracking-wider text-[#777] border-b border-[#222] pb-1.5';

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
    <div className="bg-[#1a1a1a] border-b border-[#222] px-4 py-3">
      <div className="mb-4">
        <h5 className={sectionTitle}>Enabled Scopes</h5>
        <div className="flex flex-wrap gap-5">
          <Toggle
            checked={settings.enabled.waveform}
            onChange={() => handleToggleScope('waveform')}
            label="Waveform"
          />
          <Toggle
            checked={settings.enabled.vectorscope}
            onChange={() => handleToggleScope('vectorscope')}
            label="Vectorscope"
          />
          <Toggle
            checked={settings.enabled.histogram}
            onChange={() => handleToggleScope('histogram')}
            label="Histogram"
          />
        </div>
      </div>

      {settings.enabled.waveform && (
        <div className="mb-4">
          <h5 className={sectionTitle}>Waveform</h5>
          <div className="flex items-center gap-3 mb-2.5">
            <label className="min-w-24 text-[12px] text-[#999]">Mode:</label>
            <select
              value={settings.waveform.mode}
              onChange={(e) => handleWaveformModeChange(e.target.value as ScopeSettings['waveform']['mode'])}
              className={selectClass}
            >
              <option value="luma">Luma</option>
              <option value="rgb">RGB</option>
              <option value="parade">Parade</option>
            </select>
          </div>
          <Slider
            label="Intensity"
            value={settings.waveform.intensity}
            min={0.1}
            max={2.0}
            step={0.1}
            display={settings.waveform.intensity.toFixed(1)}
            onChange={(v) =>
              onSettingsChange({ waveform: { ...settings.waveform, intensity: v } })
            }
          />
          <Toggle
            checked={settings.waveform.grid}
            onChange={(c) =>
              onSettingsChange({ waveform: { ...settings.waveform, grid: c } })
            }
            label="Grid"
          />
        </div>
      )}

      {settings.enabled.vectorscope && (
        <div className="mb-4">
          <h5 className={sectionTitle}>Vectorscope</h5>
          <Slider
            label="Intensity"
            value={settings.vectorscope.intensity}
            min={0.1}
            max={2.0}
            step={0.1}
            display={settings.vectorscope.intensity.toFixed(1)}
            onChange={(v) => handleVectorscopeSettingChange('intensity', v)}
          />
          <Slider
            label="Chroma Scale"
            value={settings.vectorscope.chromaScale}
            min={0.5}
            max={2.0}
            step={0.1}
            display={`${(settings.vectorscope.chromaScale * 100).toFixed(0)}%`}
            onChange={(v) => handleVectorscopeSettingChange('chromaScale', v)}
          />
          <div className="flex flex-wrap gap-5">
            <Toggle
              checked={settings.vectorscope.targets}
              onChange={(c) => handleVectorscopeSettingChange('targets', c)}
              label="Show Targets"
            />
            <Toggle
              checked={settings.vectorscope.grid}
              onChange={(c) => handleVectorscopeSettingChange('grid', c)}
              label="Grid"
            />
          </div>
        </div>
      )}

      {settings.enabled.histogram && (
        <div className="mb-4">
          <h5 className={sectionTitle}>Histogram</h5>
          <div className="flex items-center gap-3 mb-2.5">
            <label className="min-w-24 text-[12px] text-[#999]">Mode:</label>
            <select
              value={settings.histogram.mode}
              onChange={(e) => handleHistogramModeChange(e.target.value as ScopeSettings['histogram']['mode'])}
              className={selectClass}
            >
              <option value="rgb">RGB</option>
              <option value="luma">Luma</option>
              <option value="all">All</option>
            </select>
          </div>
          <div className="flex flex-wrap gap-5">
            <Toggle
              checked={settings.histogram.logarithmic}
              onChange={(c) =>
                onSettingsChange({ histogram: { ...settings.histogram, logarithmic: c } })
              }
              label="Logarithmic"
            />
            <Toggle
              checked={settings.histogram.grid}
              onChange={(c) =>
                onSettingsChange({ histogram: { ...settings.histogram, grid: c } })
              }
              label="Grid"
            />
          </div>
        </div>
      )}

      <div className="mb-1">
        <h5 className={sectionTitle}>Performance</h5>
        <div className="flex items-center gap-3">
          <label className="min-w-24 text-[12px] text-[#999]">Update Rate:</label>
          <select
            value={settings.updateRate}
            onChange={(e) => handleUpdateRateChange(parseInt(e.target.value))}
            className={selectClass}
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
