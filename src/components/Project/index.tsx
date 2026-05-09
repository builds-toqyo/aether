import React, { useState, useCallback, useRef } from 'react';
import { FolderOpen, Save, Plus, Settings, Clock, FileVideo, FileImage, Music, Search, Grid, List, MoreHorizontal } from 'lucide-react';
import { useDialog } from '../../hooks/useDialog';
import Dialog from '../ui/Dialog';

export interface Project {
  id: string;
  name: string;
  path: string;
  createdAt: Date;
  lastModified: Date;
  thumbnail?: string;
  duration?: number;
  mediaCount: number;
  settings: ProjectSettings;
}

export interface ProjectSettings {
  resolution: { width: number; height: number };
  framerate: number;
  audioSampleRate: number;
  autoSave: boolean;
  autoSaveInterval: number;
  proxyEnabled: boolean;
  proxyResolution: '720p' | '1080p' | '4K';
  defaultExportFormat: 'mp4' | 'mov' | 'avi' | 'mkv' | 'webm';
}

export interface MediaAsset {
  id: string;
  name: string;
  type: 'video' | 'image' | 'audio';
  path: string;
  size: number;
  duration?: number;
  resolution?: { width: number; height: number };
  thumbnail?: string;
  addedAt: Date;
}

const Project: React.FC = () => {
  const { isOpen, open, close } = useDialog();
  const [projects, setProjects] = useState<Project[]>([]);
  const [currentProject, setCurrentProject] = useState<Project | null>(null);
  const [recentProjects, setRecentProjects] = useState<Project[]>([]);
  const [mediaAssets, setMediaAssets] = useState<MediaAsset[]>([]);
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [searchQuery, setSearchQuery] = useState('');
  const [showSettings, setShowSettings] = useState(false);
  const [selectedAssets, setSelectedAssets] = useState<Set<string>>(new Set());

  const handleNewProject = useCallback(() => {
    const newProject: Project = {
      id: `project_${Date.now()}`,
      name: 'Untitled Project',
      path: '',
      createdAt: new Date(),
      lastModified: new Date(),
      mediaCount: 0,
      settings: {
        resolution: { width: 1920, height: 1080 },
        framerate: 30,
        audioSampleRate: 48000,
        autoSave: true,
        autoSaveInterval: 300,
        proxyEnabled: false,
        proxyResolution: '1080p',
        defaultExportFormat: 'mp4',
      },
    };
    setCurrentProject(newProject);
    open();
  }, [open]);

  const handleOpenProject = useCallback(() => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.aether,.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        try {
          const text = await file.text();
          const project: Project = JSON.parse(text);
          project.path = file.name;
          
          setCurrentProject(project);
          setRecentProjects(prev => {
            const filtered = prev.filter(p => p.id !== project.id);
            return [project, ...filtered.slice(0, 4)];
          });
          open();
        } catch (error) {
          console.error('Failed to load project:', error);
        }
      }
    };
    input.click();
  }, [open]);

  const handleSaveProject = useCallback(() => {
    if (!currentProject) return;
    
    const updatedProject = {
      ...currentProject,
      lastModified: new Date(),
      mediaCount: mediaAssets.length,
    };
    setCurrentProject(updatedProject);
    
    // Create and download the project file
    const projectJson = JSON.stringify(updatedProject, null, 2);
    const blob = new Blob([projectJson], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    
    const a = document.createElement('a');
    a.href = url;
    a.download = `${currentProject.name.replace(/\s+/g, '_')}.aether`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }, [currentProject, mediaAssets]);

  const handleAssetSelection = useCallback((assetId: string) => {
    setSelectedAssets(prev => {
      const newSet = new Set(prev);
      if (newSet.has(assetId)) {
        newSet.delete(assetId);
      } else {
        newSet.add(assetId);
      }
      return newSet;
    });
  }, []);

  const filteredAssets = mediaAssets.filter(asset =>
    asset.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const getAssetIcon = (type: 'video' | 'image' | 'audio') => {
    switch (type) {
      case 'video': return <FileVideo size={20} />;
      case 'image': return <FileImage size={20} />;
      case 'audio': return <Music size={20} />;
    }
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatDate = (date: Date) => {
    return new Date(date).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  if (!isOpen) {
    return (
      <div className="flex flex-col gap-4">
        <div className="flex gap-3">
          <button
            onClick={handleNewProject}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
          >
            <Plus size={20} />
            New Project
          </button>
          <button
            onClick={handleOpenProject}
            className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors"
          >
            <FolderOpen size={20} />
            Open Project
          </button>
        </div>

        {recentProjects.length > 0 && (
          <div className="mt-6">
            <h3 className="text-lg font-semibold text-white mb-3 flex items-center gap-2">
              <Clock size={20} />
              Recent Projects
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {recentProjects.map(project => (
                <div
                  key={project.id}
                  onClick={() => setCurrentProject(project)}
                  className="bg-gray-800 border border-gray-700 rounded-lg p-4 hover:bg-gray-750 cursor-pointer transition-colors"
                >
                  <div className="aspect-video bg-gray-900 rounded-lg mb-3 flex items-center justify-center">
                    <FileVideo size={48} className="text-gray-600" />
                  </div>
                  <h4 className="text-white font-medium truncate">{project.name}</h4>
                  <p className="text-gray-400 text-sm mt-1">{project.path}</p>
                  <div className="flex justify-between text-xs text-gray-500 mt-2">
                    <span>{formatDate(project.lastModified)}</span>
                    <span>{project.mediaCount} items</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    );
  }

  return (
    <Dialog
      isOpen={isOpen}
      onClose={close}
      title={currentProject?.name || 'Project'}
      maxWidth="6xl"
      disabled={false}
    >
      <div className="flex-1 flex overflow-hidden">
        <div className="w-64 border-r border-gray-700 flex flex-col">
          <div className="p-4 border-b border-gray-700">
            <h3 className="text-lg font-semibold text-white mb-3">Project Info</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-400">Created:</span>
                <span className="text-white">{currentProject && formatDate(currentProject.createdAt)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">Modified:</span>
                <span className="text-white">{currentProject && formatDate(currentProject.lastModified)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">Media:</span>
                <span className="text-white">{mediaAssets.length} items</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">Size:</span>
                <span className="text-white">
                  {formatFileSize(mediaAssets.reduce((acc, asset) => acc + asset.size, 0))}
                </span>
              </div>
            </div>
          </div>

          <div className="p-4 border-b border-gray-700">
            <button
              onClick={() => setShowSettings(!showSettings)}
              className="flex items-center gap-2 text-gray-300 hover:text-white transition-colors w-full"
            >
              <Settings size={20} />
              <span>Project Settings</span>
            </button>
          </div>

          {showSettings && currentProject && (
            <div className="p-4 border-b border-gray-700 space-y-3">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Resolution</label>
                <select
                  value={`${currentProject.settings.resolution.width}x${currentProject.settings.resolution.height}`}
                  className="w-full bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm text-white"
                >
                  <option value="1920x1080">1920x1080 (1080p)</option>
                  <option value="1280x720">1280x720 (720p)</option>
                  <option value="3840x2160">3840x2160 (4K)</option>
                  <option value="7680x4320">7680x4320 (8K)</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Frame Rate</label>
                <select
                  value={currentProject.settings.framerate}
                  className="w-full bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm text-white"
                >
                  <option value="24">24 fps</option>
                  <option value="25">25 fps</option>
                  <option value="30">30 fps</option>
                  <option value="60">60 fps</option>
                </select>
              </div>
              <div>
                <label className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={currentProject.settings.autoSave}
                    className="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded"
                  />
                  <span className="text-sm text-gray-300">Auto Save</span>
                </label>
              </div>
              <div>
                <label className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={currentProject.settings.proxyEnabled}
                    className="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded"
                  />
                  <span className="text-sm text-gray-300">Proxy Files</span>
                </label>
              </div>
            </div>
          )}
        </div>
        <div className="flex-1 flex flex-col">
          <div className="p-4 border-b border-gray-700">
            <div className="flex items-center gap-3">
              <div className="flex-1 relative">
                <Search size={20} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
                <input
                  type="text"
                  placeholder="Search media..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full bg-gray-800 border border-gray-600 rounded-lg pl-10 pr-4 py-2 text-white placeholder-gray-400"
                />
              </div>
              <div className="flex gap-2">
                <button
                  onClick={() => setViewMode('grid')}
                  className={`p-2 rounded ${viewMode === 'grid' ? 'bg-blue-600' : 'bg-gray-700 hover:bg-gray-600'} text-white transition-colors`}
                >
                  <Grid size={20} />
                </button>
                <button
                  onClick={() => setViewMode('list')}
                  className={`p-2 rounded ${viewMode === 'list' ? 'bg-blue-600' : 'bg-gray-700 hover:bg-gray-600'} text-white transition-colors`}
                >
                  <List size={20} />
                </button>
              </div>
            </div>
          </div>

          <div className="flex-1 overflow-y-auto p-4">
            {filteredAssets.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-full text-gray-500">
                <FileVideo size={48} />
                <div className="mt-4">No media assets yet</div>
                <div className="text-sm">Import media to get started</div>
              </div>
            ) : viewMode === 'grid' ? (
              <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                {filteredAssets.map(asset => (
                  <div
                    key={asset.id}
                    onClick={() => handleAssetSelection(asset.id)}
                    className={`bg-gray-800 border rounded-lg overflow-hidden cursor-pointer transition-colors ${
                      selectedAssets.has(asset.id) ? 'border-blue-500 bg-blue-900 bg-opacity-20' : 'border-gray-700 hover:border-gray-600'
                    }`}
                  >
                    <div className="aspect-video bg-gray-900 flex items-center justify-center">
                      {asset.thumbnail ? (
                        <img src={asset.thumbnail} alt={asset.name} className="w-full h-full object-cover" />
                      ) : (
                        <div className="text-gray-600">{getAssetIcon(asset.type)}</div>
                      )}
                    </div>
                    <div className="p-3">
                      <div className="text-sm text-white truncate">{asset.name}</div>
                      <div className="text-xs text-gray-400 mt-1">
                        {formatFileSize(asset.size)}
                        {asset.duration && ` • ${Math.floor(asset.duration / 60)}:${(asset.duration % 60).toFixed(2).padStart(5, '0')}`}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="space-y-2">
                {filteredAssets.map(asset => (
                  <div
                    key={asset.id}
                    onClick={() => handleAssetSelection(asset.id)}
                    className={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-colors ${
                      selectedAssets.has(asset.id) ? 'bg-blue-900 bg-opacity-30' : 'bg-gray-800 hover:bg-gray-750'
                    }`}
                  >
                    <div className="flex-shrink-0 text-gray-400">
                      {getAssetIcon(asset.type)}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="text-sm text-white truncate">{asset.name}</div>
                      <div className="text-xs text-gray-400">
                        {formatFileSize(asset.size)}
                        {asset.duration && ` • ${Math.floor(asset.duration / 60)}:${(asset.duration % 60).toFixed(2).padStart(5, '0')}`}
                      </div>
                    </div>
                    <button className="text-gray-400 hover:text-white">
                      <MoreHorizontal size={20} />
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
      <div className="border-t border-gray-700 p-4">
        <div className="flex justify-between items-center">
          <div className="text-sm text-gray-400">
            {selectedAssets.size > 0 && `${selectedAssets.size} item${selectedAssets.size > 1 ? 's' : ''} selected`}
          </div>
          <div className="flex gap-3">
            <button
              onClick={handleSaveProject}
              className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
            >
              <Save size={20} />
              Save Project
            </button>
          </div>
        </div>
      </div>
    </Dialog>
  );
};

export default Project;
