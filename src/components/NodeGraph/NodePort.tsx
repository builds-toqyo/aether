'use client';

import React from 'react';

interface NodePortProps {
  port: {
    id: string;
    name: string;
    type: 'input' | 'output';
    dataType: string;
    position: { x: number; y: number };
  };
  onPortClick: (portId: string, e: React.MouseEvent) => void;
}

export const NodePort: React.FC<NodePortProps> = ({ port, onPortClick }) => {
  const getPortColor = (dataType: string) => {
    const colors = {
      image: 'bg-blue-500',
      video: 'bg-green-500',
      audio: 'bg-purple-500',
      number: 'bg-orange-500',
      vector: 'bg-pink-500',
      boolean: 'bg-[#777]'
    };
    return colors[dataType as keyof typeof colors] || 'bg-[#777]';
  };

  const isInput = port.type === 'input';

  return (
    <div
      className={`flex items-center space-x-2 group cursor-pointer hover:bg-[#333] rounded px-1 py-1 transition-colors`}
      onClick={(e) => onPortClick(port.id, e)}
    >
      <div
        className={`w-3 h-3 rounded-full border-2 border-[#444] ${getPortColor(port.dataType)} ${
          isInput ? 'order-first' : 'order-last'
        }`}
      />

      <span className="text-xs text-[#999] group-hover:text-[#e0e0e0] transition-colors">
        {port.name}
      </span>
    </div>
  );
};
