import React from 'react';
import { Check, AlertTriangle, Loader2 } from 'lucide-react';

interface ProgressBarProps {
  progress: number;
  status: 'idle' | 'preparing' | 'processing' | 'finalizing' | 'completed' | 'error';
  showDetails?: boolean;
  currentFile?: string;
  currentFrame?: number;
  totalFrames?: number;
  speed?: number;
  timeRemaining?: number;
  error?: string;
}

const ProgressBar: React.FC<ProgressBarProps> = ({
  progress,
  status,
  showDetails = false,
  currentFile,
  currentFrame,
  totalFrames,
  speed,
  timeRemaining,
  error,
}) => {
  const getStatusIcon = () => {
    switch (status) {
      case 'preparing':
      case 'processing':
      case 'finalizing':
        return <Loader2 size={16} className="animate-spin text-blue-500" />;
      case 'completed':
        return <Check size={16} className="text-green-500" />;
      case 'error':
        return <AlertTriangle size={16} className="text-red-500" />;
      default:
        return null;
    }
  };

  const getStatusText = () => {
    switch (status) {
      case 'preparing':
        return 'Preparing...';
      case 'processing':
        return 'Processing...';
      case 'finalizing':
        return 'Finalizing...';
      case 'completed':
        return 'Completed!';
      case 'error':
        return 'Error!';
      default:
        return '';
    }
  };

  const formatTime = (seconds: number) => {
    if (!seconds || seconds <= 0) return '--';
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);
    
    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${minutes}:${secs.toString().padStart(2, '0')}`;
  };

  const getProgressColor = () => {
    switch (status) {
      case 'error':
        return 'bg-red-600';
      case 'completed':
        return 'bg-green-600';
      default:
        return 'bg-blue-600';
    }
  };

  return (
    <div className="space-y-2">
      {/* Status */}
      <div className="flex items-center gap-2">
        {getStatusIcon()}
        <span className="text-sm text-gray-300">{getStatusText()}</span>
      </div>

      {/* Progress Bar */}
      <div className="w-full bg-gray-700 rounded-full h-2">
        <div
          className={`${getProgressColor()} h-2 rounded-full transition-all duration-300`}
          style={{ width: `${progress}%` }}
        />
      </div>

      {/* Details */}
      {showDetails && (
        <div className="flex justify-between text-xs text-gray-400">
          <span>{Math.round(progress)}%</span>
          {currentFrame && totalFrames && (
            <span>{currentFrame} / {totalFrames} frames</span>
          )}
          {speed && <span>{speed.toFixed(1)} fps</span>}
          {timeRemaining && <span>{formatTime(timeRemaining)} remaining</span>}
        </div>
      )}

      {/* Current File */}
      {currentFile && (
        <div className="text-xs text-gray-400 truncate">{currentFile}</div>
      )}

      {/* Error Message */}
      {error && (
        <div className="text-xs text-red-400">{error}</div>
      )}
    </div>
  );
};

export default ProgressBar;
