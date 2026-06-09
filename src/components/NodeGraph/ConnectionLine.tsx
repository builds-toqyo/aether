'use client';

import React from 'react';

interface ConnectionLineProps {
  start: { x: number; y: number };
  end: { x: number; y: number };
  isPreview?: boolean;
}

export const ConnectionLine: React.FC<ConnectionLineProps> = ({ start, end, isPreview = false }) => {
  const getPath = () => {
    const controlOffset = Math.abs(end.x - start.x) * 0.5;
    const controlX1 = start.x + controlOffset;
    const controlY1 = start.y;
    const controlX2 = end.x - controlOffset;
    const controlY2 = end.y;

    return `M ${start.x} ${start.y} C ${controlX1} ${controlY1}, ${controlX2} ${controlY2}, ${end.x} ${end.y}`;
  };

  return (
    <path
      d={getPath()}
      fill="none"
      stroke={isPreview ? '#60A5FA' : '#6B7280'}
      strokeWidth={2}
      strokeDasharray={isPreview ? '5,5' : ''}
      opacity={isPreview ? 0.7 : 1}
      className="pointer-events-none"
    />
  );
};
