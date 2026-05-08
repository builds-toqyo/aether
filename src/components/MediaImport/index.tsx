import React, { useState, useCallback, useRef } from 'react';
import { Upload, FileVideo, FileImage, FileAudio, X, Check, AlertTriangle, Settings, Play, Pause } from 'lucide-react';
import { useTauriAPI } from '../../hooks/useTauriAPI';

export interface MediaFile {
  id: string;
  name: string;
  path: string;
  type: 'video' | 'image' | 'audio';
  size: number;
  duration?: number;
  resolution?: { width: number; height: number };
  fps?: number;
  codec?: string;
  format?: string;
  bitrate?: number;
  thumbnail?: string;
  selected: boolean;
  importStatus: 'pending' | 'importing' | 'completed' | 'error';
  error?: string;
}

export interface ProxySettings {
  enabled: boolean;
  resolution: '720p' | '1080p' | '4K';
  codec: 'h264' | 'h265' | 'prores';
  quality: 'low' | 'medium' | 'high';
  generateAudioProxy: boolean;
}

export interface ImportSettings {
  createProxy: boolean;
  proxySettings: ProxySettings;
  analyzeMedia: boolean;
  generateThumbnails: boolean;
  extractMetadata: boolean;
}

const MediaImport: React.FC = () => {
  const [isOpen, setIsOpen] = useState(false);
  const [files, setFiles] = useState<MediaFile[]>([]);
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [isImporting, setIsImporting] = useState(false);
  const [importProgress, setImportProgress] = useState(0);
  const [currentImportFile, setCurrentImportFile] = useState<string>('');
  const [settings, setSettings] = useState<ImportSettings>({
    createProxy: true,
    proxySettings: {
      enabled: true,
      resolution: '1080p',
      codec: 'h264',
      quality: 'medium',
      generateAudioProxy: true,
    },
    analyzeMedia: true,
    generateThumbnails: true,
    extractMetadata: true,
  });
  const [previewFile, setPreviewFile] = useState<MediaFile | null>(null);
  const [isPreviewPlaying, setIsPreviewPlaying] = useState(false);
  
  const fileInputRef = useRef<HTMLInputElement>(null);
  const previewVideoRef = useRef<HTMLVideoElement>(null);
  const { editing } = useTauriAPI();

  const handleFileSelect = useCallback(async (event: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = Array.from(event.target.files || []);
    
    const newFiles: MediaFile[] = selectedFiles.map((file, index) => {
      const extension = file.name.split('.').pop()?.toLowerCase();
      let type: 'video' | 'image' | 'audio' = 'video';
      
      if (extension && ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'tiff', 'webp'].includes(extension)) {
        type = 'image';
      } else if (extension && ['mp3', 'wav', 'aac', 'flac', 'ogg', 'm4a'].includes(extension)) {
        type = 'audio';
      }
      
      return {
        id: `file_${Date.now()}_${index}`,
        name: file.name,
        path: (file as any).path || file.name,
        type,
        size: file.size,
        selected: true,
        importStatus: 'pending',
      };
    });
    
    setFiles(prev => [...prev, ...newFiles]);
    setSelectedFiles(prev => {
      const newSet = new Set(prev);
      newFiles.forEach(file => newSet.add(file.id));
      return newSet;
    });
  }, []);

  const handleFileToggle = useCallback((fileId: string) => {
    setSelectedFiles(prev => {
      const newSet = new Set(prev);
      if (newSet.has(fileId)) {
        newSet.delete(fileId);
      } else {
        newSet.add(fileId);
      }
      return newSet;
    });
    
    setFiles(prev => prev.map(file => 
      file.id === fileId ? { ...file, selected: !file.selected } : file
    ));
  }, []);

  const handleRemoveFile = useCallback((fileId: string) => {
    setFiles(prev => prev.filter(file => file.id !== fileId));
    setSelectedFiles(prev => {
      const newSet = new Set(prev);
      newSet.delete(fileId);
      return newSet;
    });
    
    if (previewFile?.id === fileId) {
      setPreviewFile(null);
    }
  }, [previewFile]);

  const handlePreview = useCallback((file: MediaFile) => {
    setPreviewFile(file);
    setIsPreviewPlaying(false);
  }, []);

  const togglePreviewPlayback = useCallback(() => {
    if (previewVideoRef.current) {
      if (isPreviewPlaying) {
        previewVideoRef.current.pause();
      } else {
        previewVideoRef.current.play();
      }
      setIsPreviewPlaying(!isPreviewPlaying);
    }
  }, [isPreviewPlaying]);

  const handleImport = useCallback(async () => {
    if (selectedFiles.size === 0) return;
    
    setIsImporting(true);
    setImportProgress(0);
    
    const filesToImport = files.filter(file => selectedFiles.has(file.id));
    
    try {
      for (let i = 0; i < filesToImport.length; i++) {
        const file = filesToImport[i];
        setCurrentImportFile(file.name);
        
        setFiles(prev => prev.map(f => 
          f.id === file.id ? { ...f, importStatus: 'importing' } : f
        ));
        
        try {
          const result = await editing.import_media({
            files: [file.path],
            settings: settings.createProxy ? settings.proxySettings : undefined,
            analyze: settings.analyzeMedia,
            generateThumbnails: settings.generateThumbnails,
            extractMetadata: settings.extractMetadata,
          });
          
          setFiles(prev => prev.map(f => 
            f.id === file.id ? { 
              ...f, 
              importStatus: 'completed',
              ...result[0]
            } : f
          ));
        } catch (error) {
          setFiles(prev => prev.map(f => 
            f.id === file.id ? { 
              ...f, 
              importStatus: 'error',
              error: error instanceof Error ? error.message : 'Import failed'
            } : f
          ));
        }
        
        setImportProgress(((i + 1) / filesToImport.length) * 100);
      }
    } finally {
      setIsImporting(false);
      setCurrentImportFile('');
    }
  }, [selectedFiles, files, settings, editing]);

  const getFileIcon = (type: 'video' | 'image' | 'audio') => {
    switch (type) {
      case 'video': return <FileVideo size={20} />;
      case 'image': return <FileImage size={20} />;
      case 'audio': return <FileAudio size={20} />;
    }
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatDuration = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);
    
    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${minutes}:${secs.toString().padStart(2, '0')}`;
  };

  if (!isOpen) {
    return (
      <button
        onClick={() => setIsOpen(true)}
        className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
      >
        <Upload size={20} />
        Import Media
      </button>
    );
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-900 border border-gray-700 rounded-lg w-full max-w-6xl max-h-[90vh] flex flex-col">
        <div className="flex justify-between items-center p-4 border-b border-gray-700">
          <h2 className="text-xl font-semibold text-white">Import Media</h2>
          <button
            onClick={() => setIsOpen(false)}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <X size={24} />
          </button>
        </div>
        <div className="flex-1 flex overflow-hidden">
          <div className="flex-1 flex flex-col border-r border-gray-700">
            <div className="p-4 border-b border-gray-700">
              <input
                ref={fileInputRef}
                type="file"
                multiple
                accept="video/*,image/*,audio/*"
                onChange={handleFileSelect}
                className="hidden"
              />
              <button
                onClick={() => fileInputRef.current?.click()}
                className="w-full flex items-center justify-center gap-2 p-4 border-2 border-dashed border-gray-600 rounded-lg hover:border-gray-500 transition-colors"
              >
                <Upload size={24} />
                <span className="text-gray-300">Select files to import</span>
              </button>
            </div>

            <div className="flex-1 overflow-y-auto">
              {files.length === 0 ? (
                <div className="flex items-center justify-center h-full text-gray-500">
                  No files selected
                </div>
              ) : (
                <div className="p-2">
                  {files.map((file) => (
                    <div
                      key={file.id}
                      className={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-colors ${
                        file.selected ? 'bg-blue-900 bg-opacity-30' : 'hover:bg-gray-800'
                      } ${previewFile?.id === file.id ? 'ring-2 ring-blue-500' : ''}`}
                    >
                      <input
                        type="checkbox"
                        checked={file.selected}
                        onChange={() => handleFileToggle(file.id)}
                        className="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded focus:ring-blue-500"
                      />
                      <div className="flex-shrink-0 text-gray-400">
                        {getFileIcon(file.type)}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="text-sm text-white truncate">{file.name}</div>
                        <div className="text-xs text-gray-400">
                          {formatFileSize(file.size)}
                          {file.duration && ` • ${formatDuration(file.duration)}`}
                          {file.resolution && ` • ${file.resolution.width}x${file.resolution.height}`}
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        {file.importStatus === 'completed' && (
                          <Check size={16} className="text-green-500" />
                        )}
                        {file.importStatus === 'error' && (
                          <AlertTriangle size={16} className="text-red-500" />
                        )}
                        {file.importStatus === 'importing' && (
                          <div className="w-4 h-4 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
                        )}
                        <button
                          onClick={() => handlePreview(file)}
                          className="text-gray-400 hover:text-white transition-colors"
                        >
                          <Play size={16} />
                        </button>
                        <button
                          onClick={() => handleRemoveFile(file.id)}
                          className="text-gray-400 hover:text-red-500 transition-colors"
                        >
                          <X size={16} />
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Preview Panel */}
          <div className="w-96 flex flex-col">
            {previewFile ? (
              <>
                <div className="p-4 border-b border-gray-700">
                  <h3 className="text-lg font-semibold text-white mb-2">Preview</h3>
                  <div className="text-sm text-gray-300 truncate">{previewFile.name}</div>
                </div>
                
                <div className="flex-1 p-4 flex items-center justify-center bg-black">
                  {previewFile.type === 'video' ? (
                    <div className="relative">
                      <video
                        ref={previewVideoRef}
                        src={previewFile.path}
                        className="max-w-full max-h-full"
                        controls={false}
                      />
                      <button
                        onClick={togglePreviewPlayback}
                        className="absolute bottom-4 right-4 p-2 bg-black bg-opacity-50 rounded-full text-white hover:bg-opacity-70 transition-colors"
                      >
                        {isPreviewPlaying ? <Pause size={20} /> : <Play size={20} />}
                      </button>
                    </div>
                  ) : previewFile.type === 'image' ? (
                    <img
                      src={previewFile.path}
                      alt={previewFile.name}
                      className="max-w-full max-h-full object-contain"
                    />
                  ) : (
                    <div className="text-center text-gray-400">
                      <FileAudio size={48} />
                      <div className="mt-2">Audio preview not available</div>
                    </div>
                  )}
                </div>

                {/* Metadata */}
                <div className="p-4 border-t border-gray-700 max-h-64 overflow-y-auto">
                  <h4 className="text-sm font-semibold text-white mb-2">Metadata</h4>
                  <div className="space-y-1 text-xs text-gray-300">
                    <div>Type: {previewFile.type}</div>
                    <div>Size: {formatFileSize(previewFile.size)}</div>
                    {previewFile.duration && <div>Duration: {formatDuration(previewFile.duration)}</div>}
                    {previewFile.resolution && (
                      <div>Resolution: {previewFile.resolution.width}x{previewFile.resolution.height}</div>
                    )}
                    {previewFile.fps && <div>FPS: {previewFile.fps}</div>}
                    {previewFile.codec && <div>Codec: {previewFile.codec}</div>}
                    {previewFile.format && <div>Format: {previewFile.format}</div>}
                    {previewFile.bitrate && <div>Bitrate: {previewFile.bitrate}</div>}
                  </div>
                </div>
              </>
            ) : (
              <div className="flex-1 flex items-center justify-center text-gray-500">
                Select a file to preview
              </div>
            )}
          </div>
        </div>

        {/* Settings Panel */}
        <div className="border-t border-gray-700 p-4">
          <div className="flex items-center gap-2 mb-4">
            <Settings size={20} className="text-gray-400" />
            <h3 className="text-lg font-semibold text-white">Import Settings</h3>
          </div>
          
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-3">
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={settings.analyzeMedia}
                  onChange={(e) => setSettings(prev => ({ ...prev, analyzeMedia: e.target.checked }))}
                  className="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded"
                />
                <span className="text-sm text-gray-300">Analyze media</span>
              </label>
              
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={settings.generateThumbnails}
                  onChange={(e) => setSettings(prev => ({ ...prev, generateThumbnails: e.target.checked }))}
                  className="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded"
                />
                <span className="text-sm text-gray-300">Generate thumbnails</span>
              </label>
              
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={settings.extractMetadata}
                  onChange={(e) => setSettings(prev => ({ ...prev, extractMetadata: e.target.checked }))}
                  className="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded"
                />
                <span className="text-sm text-gray-300">Extract metadata</span>
              </label>
            </div>
            
            <div className="space-y-3">
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={settings.createProxy}
                  onChange={(e) => setSettings(prev => ({ ...prev, createProxy: e.target.checked }))}
                  className="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded"
                />
                <span className="text-sm text-gray-300">Create proxy files</span>
              </label>
              
              {settings.createProxy && (
                <div className="pl-6 space-y-2">
                  <select
                    value={settings.proxySettings.resolution}
                    onChange={(e) => setSettings(prev => ({
                      ...prev,
                      proxySettings: { ...prev.proxySettings, resolution: e.target.value as any }
                    }))}
                    className="w-full bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm text-white"
                  >
                    <option value="720p">720p</option>
                    <option value="1080p">1080p</option>
                    <option value="4K">4K</option>
                  </select>
                  
                  <select
                    value={settings.proxySettings.codec}
                    onChange={(e) => setSettings(prev => ({
                      ...prev,
                      proxySettings: { ...prev.proxySettings, codec: e.target.value as any }
                    }))}
                    className="w-full bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm text-white"
                  >
                    <option value="h264">H.264</option>
                    <option value="h265">H.265</option>
                    <option value="prores">ProRes</option>
                  </select>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="border-t border-gray-700 p-4">
          <div className="flex justify-between items-center">
            <div className="text-sm text-gray-400">
              {selectedFiles.size > 0 && `${selectedFiles.size} file${selectedFiles.size > 1 ? 's' : ''} selected`}
            </div>
            
            <div className="flex gap-3">
              <button
                onClick={() => setIsOpen(false)}
                className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg transition-colors"
              >
                Cancel
              </button>
              
              <button
                onClick={handleImport}
                disabled={selectedFiles.size === 0 || isImporting}
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed text-white rounded-lg transition-colors flex items-center gap-2"
              >
                {isImporting && (
                  <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                )}
                {isImporting ? `Importing... ${Math.round(importProgress)}%` : 'Import Selected'}
              </button>
            </div>
          </div>
          
          {isImporting && (
            <div className="mt-3">
              <div className="text-xs text-gray-400 mb-1">{currentImportFile}</div>
              <div className="w-full bg-gray-700 rounded-full h-2">
                <div
                  className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${importProgress}%` }}
                />
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default MediaImport;
