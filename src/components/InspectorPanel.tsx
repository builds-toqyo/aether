"use client";

import React from "react";
import { Type, Hash, Maximize, Volume2, Settings } from "lucide-react";

export const InspectorPanel: React.FC = () => {
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
