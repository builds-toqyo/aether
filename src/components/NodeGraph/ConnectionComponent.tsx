'use client';

import React from 'react';
import { useNodeGraph } from './NodeGraphContext';

interface ConnectionComponentProps {
  connection: {
    id: string;
    sourceNodeId: string;
    sourcePortId: string;
    targetNodeId: string;
    targetPortId: string;
  };
  sourceNode: any;
  targetNode: any;
  isSelected: boolean;
}

export const ConnectionComponent: React.FC<ConnectionComponentProps> = ({
  connection,
  sourceNode,
  targetNode,
  isSelected
}) => {
  const { setSelectedConnection, handleDeleteConnection } = useNodeGraph();

  const getConnectionPath = () => {
    if (!sourceNode || !targetNode) return '';

    const sourcePort = sourceNode.outputs.find((p: any) => p.id === connection.sourcePortId);
    const targetPort = targetNode.inputs.find((p: any) => p.id === connection.targetPortId);

    if (!sourcePort || !targetPort) return '';

    const startX = sourceNode.position.x + sourcePort.position.x;
    const startY = sourceNode.position.y + sourcePort.position.y;
    const endX = targetNode.position.x + targetPort.position.x;
    const endY = targetNode.position.y + targetPort.position.y;

    // Create a curved path
    const controlOffset = Math.abs(endX - startX) * 0.5;
    const controlX1 = startX + controlOffset;
    const controlY1 = startY;
    const controlX2 = endX - controlOffset;
    const controlY2 = endY;

    return `M ${startX} ${startY} C ${controlX1} ${controlY1}, ${controlX2} ${controlY2}, ${endX} ${endY}`;
  };

  const handleConnectionClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setSelectedConnection(connection);
  };

  const handleDeleteClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    handleDeleteConnection(connection.id);
  };

  return (
    <g>
      <path
        d={getConnectionPath()}
        fill="none"
        stroke={isSelected ? '#60A5FA' : '#6B7280'}
        strokeWidth={isSelected ? 3 : 2}
        className="cursor-pointer hover:stroke-blue-400 transition-colors"
        onClick={handleConnectionClick}
      />

      {targetNode && (
        <polygon
          points="0,-5 10,0 0,5"
          fill={isSelected ? '#60A5FA' : '#6B7280'}
          transform={`translate(${targetNode.position.x + 5}, ${targetNode.position.y + 20})`}
        />
      )}
      
      {isSelected && (
        <g
          onClick={handleDeleteClick}
          className="cursor-pointer"
        >
          <circle
            cx={(sourceNode?.position.x || 0) + 100}
            cy={(sourceNode?.position.y || 0) + 50}
            r="8"
            fill="#EF4444"
            className="hover:fill-red-600 transition-colors"
          />
          <line
            x1={(sourceNode?.position.x || 0) + 96}
            y1={(sourceNode?.position.y || 0) + 46}
            x2={(sourceNode?.position.x || 0) + 104}
            y2={(sourceNode?.position.y || 0) + 54}
            stroke="white"
            strokeWidth="2"
          />
          <line
            x1={(sourceNode?.position.x || 0) + 104}
            y1={(sourceNode?.position.y || 0) + 46}
            x2={(sourceNode?.position.x || 0) + 96}
            y2={(sourceNode?.position.y || 0) + 54}
            stroke="white"
            strokeWidth="2"
          />
        </g>
      )}
    </g>
  );
};
