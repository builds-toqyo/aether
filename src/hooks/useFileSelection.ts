import { useState, useCallback } from 'react';

export interface FileItem {
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
  status: 'pending' | 'processing' | 'completed' | 'error';
  error?: string;
}

export const useFileSelection = () => {
  const [files, setFiles] = useState<FileItem[]>([]);
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());

  const addFiles = useCallback((newFiles: FileItem[]) => {
    setFiles(prev => [...prev, ...newFiles]);
    setSelectedFiles(prev => {
      const newSet = new Set(prev);
      newFiles.forEach(file => newSet.add(file.id));
      return newSet;
    });
  }, []);

  const removeFile = useCallback((fileId: string) => {
    setFiles(prev => prev.filter(file => file.id !== fileId));
    setSelectedFiles(prev => {
      const newSet = new Set(prev);
      newSet.delete(fileId);
      return newSet;
    });
  }, []);

  const toggleFileSelection = useCallback((fileId: string) => {
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

  const selectAll = useCallback(() => {
    const allFileIds = new Set(files.map(file => file.id));
    setSelectedFiles(allFileIds);
    setFiles(prev => prev.map(file => ({ ...file, selected: true })));
  }, [files]);

  const deselectAll = useCallback(() => {
    setSelectedFiles(new Set());
    setFiles(prev => prev.map(file => ({ ...file, selected: false })));
  }, []);

  const updateFileStatus = useCallback((fileId: string, status: FileItem['status'], error?: string) => {
    setFiles(prev => prev.map(file => 
      file.id === fileId ? { ...file, status, error } : file
    ));
  }, []);

  const updateFileMetadata = useCallback((fileId: string, metadata: Partial<FileItem>) => {
    setFiles(prev => prev.map(file => 
      file.id === fileId ? { ...file, ...metadata } : file
    ));
  }, []);

  const clearFiles = useCallback(() => {
    setFiles([]);
    setSelectedFiles(new Set());
  }, []);

  const getSelectedFiles = useCallback(() => {
    return files.filter(file => selectedFiles.has(file.id));
  }, [files, selectedFiles]);

  const getFilesByStatus = useCallback((status: FileItem['status']) => {
    return files.filter(file => file.status === status);
  }, [files]);

  return {
    files,
    selectedFiles,
    selectedCount: selectedFiles.size,
    addFiles,
    removeFile,
    toggleFileSelection,
    selectAll,
    deselectAll,
    updateFileStatus,
    updateFileMetadata,
    clearFiles,
    getSelectedFiles,
    getFilesByStatus,
  };
};
