"use client";

import React, { useState, useEffect } from "react";
import { Type, Hash, Maximize, Volume2, Settings, GitBranch, Link, Plus } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

export const InspectorPanel: React.FC = () => {
  const [selectedClipId, setSelectedClipId] = useState<string | null>(null);
  const [availableNodes, setAvailableNodes] = useState<any[]>([]);
  const [selectedNode, setSelectedNode] = useState<string>("");
  const [selectedOutputPin, setSelectedOutputPin] = useState<string>("");
  const [selectedEffect, setSelectedEffect] = useState<string>("");
  const [bindings, setBindings] = useState<any[]>([]);

  useEffect(() => {
    // Fetch available nodes from node graph
    const fetchNodes = async () => {
      try {
        const graphInfo = await invoke<any>('get_graph_info');
        if (graphInfo && graphInfo.nodes) {
          setAvailableNodes(graphInfo.nodes);
        }
      } catch (err) {
        console.error('Failed to fetch nodes:', err);
      }
    };
    fetchNodes();
  }, []);

  const handleBindNode = async () => {
    if (!selectedClipId || !selectedNode || !selectedOutputPin || !selectedEffect) {
      return;
    }

    try {
      await invoke('timeline_bind_node_to_effect', {
        request: {
          clip_id: selectedClipId,
          effect_id: selectedEffect,
          node_id: selectedNode,
          output_pin_name: selectedOutputPin,
        }
      });
      setBindings([...bindings, { node_id: selectedNode, output_pin: selectedOutputPin, effect_id: selectedEffect }]);
    } catch (err) {
      console.error('Failed to bind node:', err);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="h-8 flex items-center px-3 border-b border-[#1f1f1f] bg-[#161616]">
        <Settings size={12} className="text-[#555] mr-2" />
        <span className="text-[11px] font-semibold text-[#888] uppercase tracking-wider">
          Inspector
        </span>
      </div>

      <div className="flex-1 overflow-y-auto p-3 space-y-4">
        <InspectorSection title="Transform" icon={<Maximize size={12} />}>
          <InspectorField label="Position X" value="960" />
          <InspectorField label="Position Y" value="540" />
          <InspectorField label="Scale" value="1.0" />
          <InspectorField label="Rotation" value="0°" />
        </InspectorSection>

        <InspectorSection title="Opacity" icon={<Type size={12} />}>
          <div className="flex items-center gap-2">
            <div className="flex-1 h-1.5 bg-[#222] rounded-full overflow-hidden">
              <div className="h-full w-[100%] bg-blue-600 rounded-full" />
            </div>
            <span className="text-[11px] text-[#888] w-8 text-right">100%</span>
          </div>
        </InspectorSection>

        <InspectorSection title="Audio" icon={<Volume2 size={12} />}>
          <InspectorField label="Volume" value="0 dB" />
          <InspectorField label="Pan" value="Center" />
        </InspectorSection>

        <InspectorSection title="Node Effects" icon={<GitBranch size={12} />}>
          <div className="space-y-2">
            <div className="space-y-1.5">
              <label className="text-[10px] text-[#555]">Node</label>
              <select
                value={selectedNode}
                onChange={(e) => setSelectedNode(e.target.value)}
                className="w-full bg-[#1a1a1a] border border-[#333] rounded px-2 py-1 text-[11px] text-[#ccc] outline-none focus:border-blue-500"
              >
                <option value="">Select node...</option>
                {availableNodes.map((node: any) => (
                  <option key={node.id} value={node.id}>{node.name}</option>
                ))}
              </select>
            </div>

            <div className="space-y-1.5">
              <label className="text-[10px] text-[#555]">Output Pin</label>
              <select
                value={selectedOutputPin}
                onChange={(e) => setSelectedOutputPin(e.target.value)}
                className="w-full bg-[#1a1a1a] border border-[#333] rounded px-2 py-1 text-[11px] text-[#ccc] outline-none focus:border-blue-500"
              >
                <option value="">Select output...</option>
                <option value="output">Output</option>
                <option value="image">Image</option>
                <option value="color">Color</option>
              </select>
            </div>

            <div className="space-y-1.5">
              <label className="text-[10px] text-[#555]">Effect</label>
              <select
                value={selectedEffect}
                onChange={(e) => setSelectedEffect(e.target.value)}
                className="w-full bg-[#1a1a1a] border border-[#333] rounded px-2 py-1 text-[11px] text-[#ccc] outline-none focus:border-blue-500"
              >
                <option value="">Select effect...</option>
                <option value="color_correction">Color Correction</option>
                <option value="blur">Blur</option>
                <option value="transform">Transform</option>
              </select>
            </div>

            <button
              onClick={handleBindNode}
              disabled={!selectedNode || !selectedOutputPin || !selectedEffect}
              className="flex items-center justify-center gap-1.5 w-full px-2 py-1.5 bg-blue-600/20 hover:bg-blue-600/30 border border-blue-600/30 rounded text-[11px] text-blue-400 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
            >
              <Link size={12} />
              Bind Node to Effect
            </button>

            {bindings.length > 0 && (
              <div className="pt-2 border-t border-[#222]">
                <div className="text-[10px] text-[#555] mb-1.5">Active Bindings</div>
                {bindings.map((binding, idx) => (
                  <div key={idx} className="flex items-center gap-1.5 px-2 py-1 bg-[#1a1a1a] rounded text-[10px] text-[#888]">
                    <GitBranch size={10} />
                    <span className="truncate">{binding.node_id}</span>
                    <span className="text-[#444]">→</span>
                    <span className="truncate">{binding.effect_id}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </InspectorSection>
      </div>
    </div>
  );
};

function InspectorSection({
  title,
  icon,
  children,
}: {
  title: string;
  icon: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <div>
      <div className="flex items-center gap-1.5 mb-2">
        <span className="text-[#555]">{icon}</span>
        <span className="text-[11px] font-semibold text-[#777]">{title}</span>
      </div>
      <div className="space-y-1.5 pl-1">{children}</div>
    </div>
  );
}

function InspectorField({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-[10px] text-[#555]">{label}</span>
      <span className="text-[11px] text-[#999] font-mono">{value}</span>
    </div>
  );
}

export default InspectorPanel;
