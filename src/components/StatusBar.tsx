"use client";

import React from "react";
import { HardDrive, Cpu, Wifi } from "lucide-react";

export const StatusBar: React.FC = () => {
  return (
    <div className="h-6 bg-[#141414] border-t border-[#1f1f1f] flex items-center px-3 text-[10px] text-[#555] shrink-0">
      <div className="flex items-center gap-4">
        <span className="flex items-center gap-1">
          <Wifi size={10} />
          Ready
        </span>
        <span className="flex items-center gap-1">
          <Cpu size={10} />
          GPU: Metal
        </span>
        <span className="flex items-center gap-1">
          <HardDrive size={10} />
          1920 x 1080 @ 30fps
        </span>
      </div>
      <div className="ml-auto flex items-center gap-3">
        <span>v0.1.0</span>
      </div>
    </div>
  );
};

export default StatusBar;
