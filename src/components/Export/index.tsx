import React, { useState, useCallback, useEffect } from 'react';
import { Download, X, Play, Settings, Save, Trash2, Plus, Film, FileVideo, Music, Image } from 'lucide-react';
import { useTauriAPI } from '../../hooks/useTauriAPI';
import { useDialog } from '../../hooks/useDialog';
import { useProgress } from '../../hooks/useProgress';
import Dialog from '../ui/Dialog';
import ProgressBar from '../ui/ProgressBar';

export interface ExportPreset {
  id: string;
  name: string;
  format: 'mp4' | 'mov' | 'avi' | 'mkv' | 'webm' | 'mp3' | 'wav' | 'png' | 'jpg' | 'tiff';
  codec: 'h264' | 'h265' | 'prores' | 'dnxhd' | 'aac' | 'mp3' | 'pcm' | 'png' | 'jpeg';
  quality: 'low' | 'medium' | 'high' | 'custom';
  resolution?: '720p' | '1080p' | '4K' | '8K' | 'original';
  framerate?: number;
  bitrate?: number;
  audioBitrate?: number;
  container: string;
  isCustom: boolean;
  description?: string;
}

export interface ExportSettings {
  preset: ExportPreset;
  customSettings: {
    resolution: { width: number; height: number };
    framerate: number;
    bitrate: number;
    audioBitrate: number;
    codec: string;
    container: string;
    quality: number;
    keyframeInterval: number;
    gopSize: number;
    profile: 'baseline' | 'main' | 'high';
    level: string;
    pixelFormat: string;
    colorSpace: string;
    range: 'limited' | 'full';
  };
  outputSettings: {
    filename: string;
    outputPath: string;
    format: string;
    includeAudio: boolean;
    includeSubtitles: boolean;
    burnSubtitles: boolean;
    chapters: boolean;
    metadata: boolean;
  };
  advancedSettings: {
    multiPass: boolean;
    hardwareAcceleration: boolean;
    gpuType: 'none' | 'nvidia' | 'amd' | 'intel';
    threads: number;
    memoryUsage: number;
    tempDirectory: string;
  };
}

const defaultPresets: ExportPreset[] = [
  {
    id: 'youtube-1080p',
    name: 'YouTube 1080p',
    format: 'mp4',
    codec: 'h264',
    quality: 'high',
    resolution: '1080p',
    framerate: 30,
    bitrate: 8000000,
    audioBitrate: 192000,
    container: 'mp4',
    isCustom: false,
    description: 'Optimized for YouTube upload at 1080p'
  },
  {
    id: 'youtube-4k',
    name: 'YouTube 4K',
    format: 'mp4',
    codec: 'h265',
    quality: 'high',
    resolution: '4K',
    framerate: 30,
    bitrate: 45000000,
    audioBitrate: 384000,
    container: 'mp4',
    isCustom: false,
    description: 'Optimized for YouTube upload at 4K'
  },
  {
    id: 'vimeo-1080p',
    name: 'Vimeo 1080p',
    format: 'mov',
    codec: 'h264',
    quality: 'high',
    resolution: '1080p',
    framerate: 30,
    bitrate: 10000000,
    audioBitrate: 320000,
    container: 'mov',
    isCustom: false,
    description: 'Optimized for Vimeo upload at 1080p'
  },
  {
    id: 'proxy-720p',
    name: 'Proxy 720p',
    format: 'mp4',
    codec: 'h264',
    quality: 'medium',
    resolution: '720p',
    framerate: 30,
    bitrate: 2000000,
    audioBitrate: 128000,
    container: 'mp4',
    isCustom: false,
    description: 'Lightweight proxy files for editing'
  },
  {
    id: 'master-prores',
    name: 'Master ProRes',
    format: 'mov',
    codec: 'prores',
    quality: 'high',
    resolution: 'original',
    framerate: 30,
    bitrate: 220000000,
    audioBitrate: 480000,
    container: 'mov',
    isCustom: false,
    description: 'High-quality ProRes for post-production'
  },
  {
    id: 'web-optimized',
    name: 'Web Optimized',
    format: 'webm',
    codec: 'h265',
    quality: 'medium',
    resolution: '1080p',
    framerate: 30,
    bitrate: 5000000,
    audioBitrate: 128000,
    container: 'webm',
    isCustom: false,
    description: 'Optimized for web streaming'
  }
];

const Export: React.FC = () => {
  const { isOpen, open, close } = useDialog();
  const { 
    isRunning: isExporting, 
    progress: exportProgress, 
    status: exportStatus, 
    currentFrame,
    totalFrames,
    speed: exportSpeed,
    timeRemaining,
    start: startExport,
    complete: completeExport,
    error: setExportError,
    update: updateProgress,
    reset: resetProgress
  } = useProgress();
  
  const [presets, setPresets] = useState<ExportPreset[]>(defaultPresets);
  const [selectedPreset, setSelectedPreset] = useState<ExportPreset>(defaultPresets[0]);
  const [settings, setSettings] = useState<ExportSettings>({
    preset: defaultPresets[0],
    customSettings: {
      resolution: { width: 1920, height: 1080 },
      framerate: 30,
      bitrate: 8000000,
      audioBitrate: 192000,
      codec: 'h264',
      container: 'mp4',
      quality: 23,
      keyframeInterval: 60,
      gopSize: 60,
      profile: 'high',
      level: '4.1',
      pixelFormat: 'yuv420p',
      colorSpace: 'bt709',
      range: 'limited',
    },
    outputSettings: {
      filename: 'export',
      outputPath: '~/Desktop',
      format: 'mp4',
      includeAudio: true,
      includeSubtitles: false,
      burnSubtitles: false,
      chapters: true,
      metadata: true,
    },
    advancedSettings: {
      multiPass: true,
      hardwareAcceleration: true,
      gpuType: 'nvidia',
      threads: 0,
      memoryUsage: 2048,
      tempDirectory: '/tmp',
    },
  });
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [showPresetDialog, setShowPresetDialog] = useState(false);
  const [editingPreset, setEditingPreset] = useState<ExportPreset | null>(null);
  
  const { rendering } = useTauriAPI();

  const handlePresetSelect = useCallback((preset: ExportPreset) => {
    setSelectedPreset(preset);
    setSettings(prev => ({
      ...prev,
      preset,
      customSettings: {
        ...prev.customSettings,
        codec: preset.codec,
        container: preset.container,
        bitrate: preset.bitrate || 8000000,
        audioBitrate: preset.audioBitrate || 192000,
        framerate: preset.framerate || 30,
        resolution: preset.resolution === 'original' 
          ? { width: 1920, height: 1080 }
          : preset.resolution === '720p'
            ? { width: 1280, height: 720 }
            : preset.resolution === '1080p'
              ? { width: 1920, height: 1080 }
              : preset.resolution === '4K'
                ? { width: 3840, height: 2160 }
                : { width: 7680, height: 4320 },
      },
      outputSettings: {
        ...prev.outputSettings,
        format: preset.format,
      },
    }));
  }, []);

  const handleExport = useCallback(async () => {
    startExport('preparing');
    
    try {
      const exportRequest = {
        settings: {
          format: settings.preset.format,
          codec: settings.customSettings.codec,
          resolution: settings.customSettings.resolution,
          framerate: settings.customSettings.framerate,
          bitrate: settings.customSettings.bitrate,
          audioBitrate: settings.customSettings.audioBitrate,
          quality: settings.customSettings.quality,
          container: settings.customSettings.container,
          keyframeInterval: settings.customSettings.keyframeInterval,
          profile: settings.customSettings.profile,
          level: settings.customSettings.level,
          pixelFormat: settings.customSettings.pixelFormat,
          colorSpace: settings.customSettings.colorSpace,
          range: settings.customSettings.range,
        },
        output: {
          filename: settings.outputSettings.filename,
          outputPath: settings.outputSettings.outputPath,
          includeAudio: settings.outputSettings.includeAudio,
          includeSubtitles: settings.outputSettings.includeSubtitles,
          burnSubtitles: settings.outputSettings.burnSubtitles,
          chapters: settings.outputSettings.chapters,
          metadata: settings.outputSettings.metadata,
        },
        advanced: {
          multiPass: settings.advancedSettings.multiPass,
          hardwareAcceleration: settings.advancedSettings.hardwareAcceleration,
          gpuType: settings.advancedSettings.gpuType,
          threads: settings.advancedSettings.threads,
          memoryUsage: settings.advancedSettings.memoryUsage,
          tempDirectory: settings.advancedSettings.tempDirectory,
        },
      };

      const result = await rendering.start_job(exportRequest);
      
      // Simulate export progress
      updateProgress({ status: 'processing', currentFrame: 0, totalFrames: 1000 });
      
      const progressInterval = setInterval(() => {
        const currentProgress = exportProgress + Math.random() * 5;
        if (currentProgress >= 100) {
          clearInterval(progressInterval);
          updateProgress({ status: 'finalizing' });
          setTimeout(() => {
            completeExport();
          }, 1000);
        } else {
          const frame = Math.floor((currentProgress / 100) * 1000);
          const speed = Math.random() * 30 + 10;
          const timeRemaining = Math.floor((100 - currentProgress) * 2);
          
          updateProgress({
            progress: currentProgress,
            currentFrame: frame,
            speed: speed,
            timeRemaining: timeRemaining,
          });
        }
      }, 500);
      
    } catch (error) {
      setExportError(error instanceof Error ? error.message : 'Export failed');
    }
  }, [settings, rendering, startExport, updateProgress, completeExport, setExportError]);

  const handleSavePreset = useCallback(() => {
    if (!editingPreset) return;
    
    const newPreset: ExportPreset = {
      ...editingPreset,
      id: `custom_${Date.now()}`,
      isCustom: true,
    };
    
    setPresets(prev => [...prev, newPreset]);
    setShowPresetDialog(false);
    setEditingPreset(null);
  }, [editingPreset]);

  const handleDeletePreset = useCallback((presetId: string) => {
    setPresets(prev => prev.filter(p => p.id !== presetId));
    if (selectedPreset?.id === presetId) {
      setSelectedPreset(defaultPresets[0]);
      handlePresetSelect(defaultPresets[0]);
    }
  }, [selectedPreset, handlePresetSelect]);

  const formatBitrate = (bitrate: number) => {
    if (bitrate >= 1000000) {
      return `${(bitrate / 1000000).toFixed(1)} Mbps`;
    }
    return `${(bitrate / 1000).toFixed(0)} kbps`;
  };

  const formatTime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);
    
    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${minutes}:${secs.toString().padStart(2, '0')}`;
  };

  const getFormatIcon = (format: string) => {
    switch (format) {
      case 'mp4':
      case 'mov':
      case 'avi':
      case 'mkv':
      case 'webm':
        return <FileVideo size={20} />;
      case 'mp3':
      case 'wav':
        return <Music size={20} />;
      case 'png':
      case 'jpg':
      case 'tiff':
        return <Image size={20} />;
      default:
        return <Film size={20} />;
    }
  };

  if (!isOpen) {
    return (
      <button
        onClick={open}
        className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors"
      >
        <Download size={20} />
        Export
      </button>
    );
  }

  return (
    <Dialog
      isOpen={isOpen}
      onClose={close}
      title="Export Media"
      maxWidth="5xl"
      disabled={isExporting}
    >

        {/* Main Content */}
        <div className="flex-1 flex overflow-hidden">
          {/* Presets Panel */}
          <div className="w-80 border-r border-gray-700 flex flex-col">
            <div className="p-4 border-b border-gray-700">
              <div className="flex justify-between items-center mb-3">
                <h3 className="text-lg font-semibold text-white">Presets</h3>
                <button
                  onClick={() => setShowPresetDialog(true)}
                  className="text-gray-400 hover:text-white transition-colors"
                >
                  <Plus size={20} />
                </button>
              </div>
              
              <div className="space-y-2 max-h-96 overflow-y-auto">
                {presets.map((preset) => (
                  <div
                    key={preset.id}
                    className={`p-3 rounded-lg cursor-pointer transition-colors ${
                      selectedPreset?.id === preset.id
                        ? 'bg-blue-900 bg-opacity-30 border border-blue-500'
                        : 'bg-gray-800 hover:bg-gray-700 border border-gray-600'
                    }`}
                    onClick={() => handlePresetSelect(preset)}
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        {getFormatIcon(preset.format)}
                        <div>
                          <div className="text-sm font-medium text-white">{preset.name}</div>
                          <div className="text-xs text-gray-400">{preset.description}</div>
                        </div>
                      </div>
                      {preset.isCustom && (
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleDeletePreset(preset.id);
                          }}
                          className="text-gray-400 hover:text-red-500 transition-colors"
                        >
                          <Trash2 size={16} />
                        </button>
                      )}
                    </div>
                    <div className="mt-2 text-xs text-gray-400">
                      {preset.resolution} • {formatBitrate(preset.bitrate || 8000000)} • {preset.codec.toUpperCase()}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Output Settings */}
            <div className="p-4 flex-1 overflow-y-auto">
              <h3 className="text-lg font-semibold text-white mb-3">Output Settings</h3>
              
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">Filename</label>
                  <input
                    type="text"
                    value={settings.outputSettings.filename}
                    onChange={(e) => setSettings(prev => ({
                      ...prev,
                      outputSettings: { ...prev.outputSettings, filename: e.target.value }
                    }))}
                    className="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white"
                  />
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">Output Path</label>
                  <input
                    type="text"
                    value={settings.outputSettings.outputPath}
                    onChange={(e) => setSettings(prev => ({
                      ...prev,
                      outputSettings: { ...prev.outputSettings, outputPath: e.target.value }
                    }))}
                    className="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white"
                  />
                </div>
                
                <div className="space-y-2">
                  <label className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={settings.outputSettings.includeAudio}
                      onChange={(e) => setSettings(prev => ({
                        ...prev,
                        outputSettings: { ...prev.outputSettings, includeAudio: e.target.checked }
                      }))}
                      className="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded"
                    />
                    <span className="text-sm text-gray-300">Include Audio</span>
                  </label>
                  
                  <label className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={settings.outputSettings.includeSubtitles}
                      onChange={(e) => setSettings(prev => ({
                        ...prev,
                        outputSettings: { ...prev.outputSettings, includeSubtitles: e.target.checked }
                      }))}
                      className="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded"
                    />
                    <span className="text-sm text-gray-300">Include Subtitles</span>
                  </label>
                  
                  <label className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={settings.outputSettings.chapters}
                      onChange={(e) => setSettings(prev => ({
                        ...prev,
                        outputSettings: { ...prev.outputSettings, chapters: e.target.checked }
                      }))}
                      className="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded"
                    />
                    <span className="text-sm text-gray-300">Include Chapters</span>
                  </label>
                  
                  <label className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={settings.outputSettings.metadata}
                      onChange={(e) => setSettings(prev => ({
                        ...prev,
                        outputSettings: { ...prev.outputSettings, metadata: e.target.checked }
                      }))}
                      className="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded"
                    />
                    <span className="text-sm text-gray-300">Include Metadata</span>
                  </label>
                </div>
              </div>
            </div>
          </div>

          {/* Settings Panel */}
          <div className="flex-1 flex flex-col">
            <div className="p-4 border-b border-gray-700">
              <div className="flex justify-between items-center mb-4">
                <h3 className="text-lg font-semibold text-white">Export Settings</h3>
                <button
                  onClick={() => setShowAdvanced(!showAdvanced)}
                  className="flex items-center gap-2 text-sm text-gray-400 hover:text-white transition-colors"
                >
                  <Settings size={16} />
                  {showAdvanced ? 'Simple' : 'Advanced'}
                </button>
              </div>
              
              {/* Basic Settings */}
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">Codec</label>
                  <select
                    value={settings.customSettings.codec}
                    onChange={(e) => setSettings(prev => ({
                      ...prev,
                      customSettings: { ...prev.customSettings, codec: e.target.value }
                    }))}
                    className="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white"
                  >
                    <option value="h264">H.264</option>
                    <option value="h265">H.265</option>
                    <option value="prores">ProRes</option>
                    <option value="dnxhd">DNxHD</option>
                  </select>
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">Container</label>
                  <select
                    value={settings.customSettings.container}
                    onChange={(e) => setSettings(prev => ({
                      ...prev,
                      customSettings: { ...prev.customSettings, container: e.target.value }
                    }))}
                    className="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white"
                  >
                    <option value="mp4">MP4</option>
                    <option value="mov">MOV</option>
                    <option value="avi">AVI</option>
                    <option value="mkv">MKV</option>
                    <option value="webm">WebM</option>
                  </select>
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">Resolution</label>
                  <select
                    value={`${settings.customSettings.resolution.width}x${settings.customSettings.resolution.height}`}
                    onChange={(e) => {
                      const [width, height] = e.target.value.split('x').map(Number);
                      setSettings(prev => ({
                        ...prev,
                        customSettings: { ...prev.customSettings, resolution: { width, height } }
                      }));
                    }}
                    className="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white"
                  >
                    <option value="1280x720">720p</option>
                    <option value="1920x1080">1080p</option>
                    <option value="3840x2160">4K</option>
                    <option value="7680x4320">8K</option>
                  </select>
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">Frame Rate</label>
                  <select
                    value={settings.customSettings.framerate}
                    onChange={(e) => setSettings(prev => ({
                      ...prev,
                      customSettings: { ...prev.customSettings, framerate: Number(e.target.value) }
                    }))}
                    className="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white"
                  >
                    <option value="24">24 fps</option>
                    <option value="25">25 fps</option>
                    <option value="30">30 fps</option>
                    <option value="50">50 fps</option>
                    <option value="60">60 fps</option>
                  </select>
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">
                    Video Bitrate: {formatBitrate(settings.customSettings.bitrate)}
                  </label>
                  <input
                    type="range"
                    min="1000000"
                    max="100000000"
                    step="1000000"
                    value={settings.customSettings.bitrate}
                    onChange={(e) => setSettings(prev => ({
                      ...prev,
                      customSettings: { ...prev.customSettings, bitrate: Number(e.target.value) }
                    }))}
                    className="w-full"
                  />
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">
                    Audio Bitrate: {formatBitrate(settings.customSettings.audioBitrate)}
                  </label>
                  <input
                    type="range"
                    min="64000"
                    max="512000"
                    step="32000"
                    value={settings.customSettings.audioBitrate}
                    onChange={(e) => setSettings(prev => ({
                      ...prev,
                      customSettings: { ...prev.customSettings, audioBitrate: Number(e.target.value) }
                    }))}
                    className="w-full"
                  />
                </div>
              </div>
              
              {/* Advanced Settings */}
              {showAdvanced && (
                <div className="mt-4 space-y-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-300 mb-1">Profile</label>
                      <select
                        value={settings.customSettings.profile}
                        onChange={(e) => setSettings(prev => ({
                          ...prev,
                          customSettings: { ...prev.customSettings, profile: e.target.value as any }
                        }))}
                        className="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white"
                      >
                        <option value="baseline">Baseline</option>
                        <option value="main">Main</option>
                        <option value="high">High</option>
                      </select>
                    </div>
                    
                    <div>
                      <label className="block text-sm font-medium text-gray-300 mb-1">Level</label>
                      <select
                        value={settings.customSettings.level}
                        onChange={(e) => setSettings(prev => ({
                          ...prev,
                          customSettings: { ...prev.customSettings, level: e.target.value }
                        }))}
                        className="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white"
                      >
                        <option value="3.1">3.1</option>
                        <option value="4.0">4.0</option>
                        <option value="4.1">4.1</option>
                        <option value="5.1">5.1</option>
                        <option value="5.2">5.2</option>
                      </select>
                    </div>
                    
                    <div>
                      <label className="block text-sm font-medium text-gray-300 mb-1">Pixel Format</label>
                      <select
                        value={settings.customSettings.pixelFormat}
                        onChange={(e) => setSettings(prev => ({
                          ...prev,
                          customSettings: { ...prev.customSettings, pixelFormat: e.target.value }
                        }))}
                        className="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white"
                      >
                        <option value="yuv420p">YUV 4:2:0</option>
                        <option value="yuv422p">YUV 4:2:2</option>
                        <option value="yuv444p">YUV 4:4:4</option>
                        <option value="rgb24">RGB 24-bit</option>
                      </select>
                    </div>
                    
                    <div>
                      <label className="block text-sm font-medium text-gray-300 mb-1">Color Space</label>
                      <select
                        value={settings.customSettings.colorSpace}
                        onChange={(e) => setSettings(prev => ({
                          ...prev,
                          customSettings: { ...prev.customSettings, colorSpace: e.target.value }
                        }))}
                        className="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white"
                      >
                        <option value="bt709">BT.709</option>
                        <option value="bt2020">BT.2020</option>
                        <option value="srgb">sRGB</option>
                        <option value="rec601">Rec.601</option>
                      </select>
                    </div>
                  </div>
                  
                  <div className="space-y-2">
                    <label className="flex items-center gap-2">
                      <input
                        type="checkbox"
                        checked={settings.advancedSettings.multiPass}
                        onChange={(e) => setSettings(prev => ({
                          ...prev,
                          advancedSettings: { ...prev.advancedSettings, multiPass: e.target.checked }
                        }))}
                        className="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded"
                      />
                      <span className="text-sm text-gray-300">Multi-pass Encoding</span>
                    </label>
                    
                    <label className="flex items-center gap-2">
                      <input
                        type="checkbox"
                        checked={settings.advancedSettings.hardwareAcceleration}
                        onChange={(e) => setSettings(prev => ({
                          ...prev,
                          advancedSettings: { ...prev.advancedSettings, hardwareAcceleration: e.target.checked }
                        }))}
                        className="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded"
                      />
                      <span className="text-sm text-gray-300">Hardware Acceleration</span>
                    </label>
                  </div>
                </div>
              )}
            </div>
            
            {/* Progress Display */}
            {isExporting && (
              <div className="p-4 border-b border-gray-700">
                <ProgressBar
                  progress={exportProgress}
                  status={exportStatus}
                  currentFrame={currentFrame}
                  totalFrames={totalFrames}
                  speed={exportSpeed}
                  timeRemaining={timeRemaining}
                  showDetails={true}
                />
              </div>
            )}
            
            {/* Preview */}
            <div className="flex-1 p-4">
              <h3 className="text-lg font-semibold text-white mb-3">Preview</h3>
              <div className="bg-black rounded-lg h-64 flex items-center justify-center">
                <div className="text-center text-gray-400">
                  <Film size={48} />
                  <div className="mt-2">Export preview</div>
                  <div className="text-xs mt-1">
                    {settings.customSettings.resolution.width}x{settings.customSettings.resolution.height} @ {settings.customSettings.framerate}fps
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="border-t border-gray-700 p-4">
          <div className="flex justify-between items-center">
            <div className="text-sm text-gray-400">
              {selectedPreset && `Selected: ${selectedPreset.name}`}
            </div>
            
            <div className="flex gap-3">
              <button
                onClick={close}
                disabled={isExporting}
                className="px-4 py-2 bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
              >
                Cancel
              </button>
              
              <button
                onClick={handleExport}
                disabled={isExporting}
                className="px-4 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-700 disabled:cursor-not-allowed text-white rounded-lg transition-colors flex items-center gap-2"
              >
                {isExporting && (
                  <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                )}
                {isExporting ? 'Exporting...' : 'Start Export'}
              </button>
            </div>
          </div>
        </div>
    </Dialog>
  );
};

export default Export;
