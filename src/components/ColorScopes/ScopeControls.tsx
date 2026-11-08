import React from 'react';
import { ScopeSettings } from './index';

interface ScopeControlsProps {
  settings: ScopeSettings;
  onSettingsChange: (newSettings: Partial<ScopeSettings>) => void;
}

const Pill: React.FC<{
  active: boolean;
  onClick: () => void;
  label: string;
}> = ({ active, onClick, label }) => (
  <button
    onClick={onClick}
    className={`px-2.5 py-1 rounded text-[11px] font-medium border transition-colors ${
      active
        ? 'bg-blue-600/15 text-blue-400 border-blue-500/40'
        : 'bg-[#252525] text-[#777] border-[#333] hover:text-[#aaa] hover:border-[#444]'
    }`}
  >
    {label}
  </button>
);

const Check: React.FC<{
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
}> = ({ checked, onChange, label }) => (
  <label className="flex items-center gap-1.5 cursor-pointer">
    <input
      type="checkbox"
      checked={checked}
      onChange={(e) => onChange(e.target.checked)}
      className="w-3 h-3 accent-blue-500"
    />
    <span className="text-[11px] text-[#999]">{label}</span>
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
  <div className="flex items-center gap-1.5">
    <span className="text-[11px] text-[#777]">{label}</span>
    <input
      type="range"
      min={min}
      max={max}
      step={step}
      value={value}
      onChange={(e) => onChange(parseFloat(e.target.value))}
      className="w-20 h-1 accent-blue-500 bg-[#333] rounded outline-none"
    />
    <span className="text-[10px] text-blue-400 tabular-nums w-8">{display}</span>
  </div>
);

const SettingsRow: React.FC<{ label: string; children: React.ReactNode }> = ({
  label,
  children,
}) => (
  <div className="flex items-center gap-4 flex-wrap">
    <span className="w-[76px] shrink-0 text-[10px] font-semibold uppercase tracking-wider text-[#555]">
      {label}
    </span>
    {children}
  </div>
);

const selectClass =
  'bg-[#252525] border border-[#333] rounded px-1.5 py-0.5 text-[11px] text-[#ccc] outline-none focus:border-blue-500 transition-colors';

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

  return (
    <div className="bg-[#1a1a1a] border-b border-[#222] px-3 py-2 space-y-1.5 shrink-0">
      <div className="flex items-center gap-2 flex-wrap">
        <Pill
          active={settings.enabled.waveform}
          onClick={() => handleToggleScope('waveform')}
          label="Waveform"
        />
        <Pill
          active={settings.enabled.vectorscope}
          onClick={() => handleToggleScope('vectorscope')}
          label="Vectorscope"
        />
        <Pill
          active={settings.enabled.histogram}
          onClick={() => handleToggleScope('histogram')}
          label="Histogram"
        />
        <div className="ml-auto flex items-center gap-1.5">
          <span className="text-[10px] text-[#555] uppercase tracking-wider">Rate</span>
          <select
            value={settings.updateRate}
            onChange={(e) => onSettingsChange({ updateRate: parseInt(e.target.value) })}
            className={selectClass}
          >
            <option value={15}>15 FPS</option>
            <option value={30}>30 FPS</option>
            <option value={60}>60 FPS</option>
            <option value={0}>Paused</option>
          </select>
        </div>
      </div>

      {settings.enabled.waveform && (
        <SettingsRow label="Waveform">
          <select
            value={settings.waveform.mode}
            onChange={(e) =>
              onSettingsChange({
                waveform: { ...settings.waveform, mode: e.target.value as ScopeSettings['waveform']['mode'] },
              })
            }
            className={selectClass}
          >
            <option value="luma">Luma</option>
            <option value="rgb">RGB</option>
            <option value="parade">Parade</option>
          </select>
          <Slider
            label="Intensity"
            value={settings.waveform.intensity}
            min={0.1}
            max={2.0}
            step={0.1}
            display={settings.waveform.intensity.toFixed(1)}
            onChange={(v) => onSettingsChange({ waveform: { ...settings.waveform, intensity: v } })}
          />
          <Check
            checked={settings.waveform.grid}
            onChange={(c) => onSettingsChange({ waveform: { ...settings.waveform, grid: c } })}
            label="Grid"
          />
        </SettingsRow>
      )}

      {settings.enabled.vectorscope && (
        <SettingsRow label="Vectorscope">
          <Slider
            label="Intensity"
            value={settings.vectorscope.intensity}
            min={0.1}
            max={2.0}
            step={0.1}
            display={settings.vectorscope.intensity.toFixed(1)}
            onChange={(v) =>
              onSettingsChange({ vectorscope: { ...settings.vectorscope, intensity: v } })
            }
          />
          <Slider
            label="Chroma"
            value={settings.vectorscope.chromaScale}
            min={0.5}
            max={2.0}
            step={0.1}
            display={`${(settings.vectorscope.chromaScale * 100).toFixed(0)}%`}
            onChange={(v) =>
              onSettingsChange({ vectorscope: { ...settings.vectorscope, chromaScale: v } })
            }
          />
          <Check
            checked={settings.vectorscope.targets}
            onChange={(c) =>
              onSettingsChange({ vectorscope: { ...settings.vectorscope, targets: c } })
            }
            label="Targets"
          />
          <Check
            checked={settings.vectorscope.grid}
            onChange={(c) => onSettingsChange({ vectorscope: { ...settings.vectorscope, grid: c } })}
            label="Grid"
          />
        </SettingsRow>
      )}

      {settings.enabled.histogram && (
        <SettingsRow label="Histogram">
          <select
            value={settings.histogram.mode}
            onChange={(e) =>
              onSettingsChange({
                histogram: { ...settings.histogram, mode: e.target.value as ScopeSettings['histogram']['mode'] },
              })
            }
            className={selectClass}
          >
            <option value="rgb">RGB</option>
            <option value="luma">Luma</option>
            <option value="all">All</option>
          </select>
          <Check
            checked={settings.histogram.logarithmic}
            onChange={(c) =>
              onSettingsChange({ histogram: { ...settings.histogram, logarithmic: c } })
            }
            label="Log"
          />
          <Check
            checked={settings.histogram.grid}
            onChange={(c) => onSettingsChange({ histogram: { ...settings.histogram, grid: c } })}
            label="Grid"
          />
        </SettingsRow>
      )}
    </div>
  );
};
