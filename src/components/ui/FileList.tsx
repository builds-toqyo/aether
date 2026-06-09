import React from 'react';
import { FileVideo, FileImage, FileAudio, X, Play, Check, AlertTriangle, Loader2 } from 'lucide-react';
import { FileItem } from '../../hooks/useFileSelection';

interface FileListProps {
  files: FileItem[];
  selectedCount: number;
  onToggleSelection: (fileId: string) => void;
  onRemove: (fileId: string) => void;
  onPreview: (file: FileItem) => void;
  previewFileId?: string;
}

const FileList: React.FC<FileListProps> = ({
  files,
  selectedCount,
  onToggleSelection,
  onRemove,
  onPreview,
  previewFileId,
}) => {
  const getFileIcon = (type: FileItem['type']) => {
    switch (type) {
      case 'video': return <FileVideo size={20} />;
      case 'image': return <FileImage size={20} />;
      case 'audio': return <FileAudio size={20} />;
    }
  };

  const getStatusIcon = (status: FileItem['status']) => {
    switch (status) {
      case 'completed':
        return <Check size={16} className="text-green-500" />;
      case 'error':
        return <AlertTriangle size={16} className="text-red-500" />;
      case 'processing':
        return <Loader2 size={16} className="animate-spin text-blue-500" />;
      default:
        return null;
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

  if (files.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-gray-500">
        No files selected
      </div>
    );
  }

  return (
    <div className="p-2">
      {files.map((file) => (
        <div
          key={file.id}
          className={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-colors ${
            file.selected ? 'bg-blue-900 bg-opacity-30' : 'hover:bg-gray-800'
          } ${previewFileId === file.id ? 'ring-2 ring-blue-500' : ''}`}
        >
          <input
            type="checkbox"
            checked={file.selected}
            onChange={() => onToggleSelection(file.id)}
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
            {getStatusIcon(file.status)}
            <button
              onClick={() => onPreview(file)}
              className="text-gray-400 hover:text-white transition-colors"
            >
              <Play size={16} />
            </button>
            <button
              onClick={() => onRemove(file.id)}
              className="text-gray-400 hover:text-red-500 transition-colors"
            >
              <X size={16} />
            </button>
          </div>
        </div>
      ))}
      
      {selectedCount > 0 && (
        <div className="mt-2 p-2 bg-gray-800 rounded text-xs text-gray-400">
          {selectedCount} file{selectedCount > 1 ? 's' : ''} selected
        </div>
      )}
    </div>
  );
};

export default FileList;
