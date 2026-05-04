'use client';

import React from 'react';
import { usePreview } from './PreviewContext';
import { X, ZoomIn, ZoomOut, RotateCw, Eye, EyeOff } from 'lucide-react';

export const PreviewSettings: React.FC = () => {
  const { 
    state, 
    handleQualityChange, 
    handleZoomIn, 
    handleZoomOut, 
    handleZoomReset,
    handleSettingsToggle 
  } = usePreview();

  return (
    <div className="absolute bottom-20 left-4 right-4 bg-gray-900 bg-opacity-95 backdrop-blur-sm rounded-lg p-4 border border-gray-700">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-white font-semibold">Preview Settings</h3>
        <button
          onClick={handleSettingsToggle}
          className="p-1 hover:bg-gray-800 rounded text-white transition-colors"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="text-gray-400 text-sm block mb-2">Quality</label>
          <select
            value={state.quality}
            onChange={(e) => handleQualityChange(e.target.value as 'low' | 'medium' | 'high' | 'auto')}
            className="w-full bg-gray-800 text-white px-3 py-2 rounded border border-gray-700 text-sm"
          >
            <option value="auto">Auto</option>
            <option value="low">Low</option>
            <option value="medium">Medium</option>
            <option value="high">High</option>
          </select>
        </div>

        <div>
          <label className="text-gray-400 text-sm block mb-2">Zoom</label>
          <div className="flex items-center space-x-2">
            <button
              onClick={handleZoomOut}
              className="p-1 bg-gray-800 hover:bg-gray-700 rounded text-white transition-colors"
            >
              <ZoomOut className="w-4 h-4" />
            </button>
            <span className="text-white text-sm min-w-[60px] text-center">
              {Math.round(state.zoom * 100)}%
            </span>
            <button
              onClick={handleZoomIn}
              className="p-1 bg-gray-800 hover:bg-gray-700 rounded text-white transition-colors"
            >
              <ZoomIn className="w-4 h-4" />
            </button>
            <button
              onClick={handleZoomReset}
              className="p-1 bg-gray-800 hover:bg-gray-700 rounded text-white transition-colors"
            >
              <RotateCw className="w-4 h-4" />
            </button>
          </div>
        </div>

        <div>
          <label className="text-gray-400 text-sm block mb-2">Display Options</label>
          <div className="space-y-2">
            <label className="flex items-center space-x-2 text-white text-sm">
              <input
                type="checkbox"
                defaultChecked={true}
                className="bg-gray-800 rounded"
              />
              <span>Show Safe Area</span>
            </label>
            <label className="flex items-center space-x-2 text-white text-sm">
              <input
                type="checkbox"
                defaultChecked={false}
                className="bg-gray-800 rounded"
              />
              <span>Show Grid</span>
            </label>
            <label className="flex items-center space-x-2 text-white text-sm">
              <input
                type="checkbox"
                defaultChecked={true}
                className="bg-gray-800 rounded"
              />
              <span>Show Frame Info</span>
            </label>
          </div>
        </div>

        <div>
          <label className="text-gray-400 text-sm block mb-2">Color Space</label>
          <select
            defaultValue="sRGB"
            className="w-full bg-gray-800 text-white px-3 py-2 rounded border border-gray-700 text-sm"
          >
            <option value="sRGB">sRGB</option>
            <option value="Rec.709">Rec.709</option>
            <option value="Rec.2020">Rec.2020</option>
            <option value="ACES">ACES</option>
          </select>
        </div>

        <div>
          <label className="text-gray-400 text-sm block mb-2">Gamma</label>
          <select
            defaultValue="2.4"
            className="w-full bg-gray-800 text-white px-3 py-2 rounded border border-gray-700 text-sm"
          >
            <option value="2.2">2.2</option>
            <option value="2.4">2.4</option>
            <option value="2.6">2.6</option>
            <option value="linear">Linear</option>
          </select>
        </div>

        <div>
          <label className="text-gray-400 text-sm block mb-2">HDR Mode</label>
          <select
            defaultValue="off"
            className="w-full bg-gray-800 text-white px-3 py-2 rounded border border-gray-700 text-sm"
          >
            <option value="off">Off</option>
            <option value="on">On</option>
            <option value="auto">Auto</option>
          </select>
        </div>
      </div>

      <div className="mt-4 pt-4 border-t border-gray-700">
        <div className="grid grid-cols-2 gap-4 text-xs text-gray-400">
          <div>
            <span className="block">Frame Rate:</span>
            <span className="text-white">30 fps</span>
          </div>
          <div>
            <span className="block">Resolution:</span>
            <span className="text-white">1920×1080</span>
          </div>
          <div>
            <span className="block">Bit Depth:</span>
            <span className="text-white">8-bit</span>
          </div>
          <div>
            <span className="block">Color Profile:</span>
            <span className="text-white">sRGB</span>
          </div>
        </div>
      </div>
    </div>
  );
};
