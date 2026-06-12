"use client";

import React, { useState } from 'react';
import { TimelineEditor } from '@/components/timeline';
import { PreviewWindow } from '@/components/Preview';
import { NodeGraphEditor } from '@/components/NodeGraph';
import { ColorScopes } from '@/components/ColorScopes';
import Project from '@/components/Project';
import { Film, Play, GitBranch, Palette, FolderOpen, Settings } from 'lucide-react';

type ViewTab = 'edit' | 'preview' | 'node' | 'color' | 'project';

const tabs: { id: ViewTab; label: string; icon: React.ReactNode }[] = [
  { id: 'edit', label: 'Edit', icon: <Film size={16} /> },
  { id: 'preview', label: 'Preview', icon: <Play size={16} /> },
  { id: 'node', label: 'Nodes', icon: <GitBranch size={16} /> },
  { id: 'color', label: 'Color', icon: <Palette size={16} /> },
  { id: 'project', label: 'Project', icon: <FolderOpen size={16} /> },
];

export default function Home() {
  const [activeTab, setActiveTab] = useState<ViewTab>('edit');

  return (
    <div className="h-screen flex flex-col bg-gray-950 text-white overflow-hidden">
      <header className="h-11 bg-gray-900 border-b border-gray-800 flex items-center px-3 shrink-0 select-none">
        <div className="flex items-center gap-3">
          <div className="w-6 h-6 bg-blue-600 rounded flex items-center justify-center">
            <Film size={14} className="text-white" />
          </div>
          <span className="font-semibold text-sm tracking-wide">Aether</span>
        </div>

        <nav className="flex items-center gap-1 ml-6">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded text-xs font-medium transition-colors ${
                activeTab === tab.id
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-400 hover:text-white hover:bg-gray-800'
              }`}
            >
              {tab.icon}
              {tab.label}
            </button>
          ))}
        </nav>

        <div className="ml-auto flex items-center gap-2">
          <button className="p-1.5 text-gray-400 hover:text-white hover:bg-gray-800 rounded transition-colors">
            <Settings size={16} />
          </button>
        </div>
      </header>

      <main className="flex-1 overflow-hidden">
        {activeTab === 'edit' && (
          <div className="h-full flex">
            <div className="w-72 bg-gray-900 border-r border-gray-800 overflow-y-auto">
              <Project />
            </div>

            <div className="flex-1 flex flex-col min-w-0">
              <div className="flex-1 min-h-0 bg-black relative">
                <PreviewWindow />
              </div>

              <div className="h-72 border-t border-gray-800 shrink-0">
                <TimelineEditor />
              </div>
            </div>
          </div>
        )}

        {activeTab === 'preview' && (
          <div className="h-full bg-black">
            <PreviewWindow />
          </div>
        )}

        {activeTab === 'node' && (
          <div className="h-full">
            <NodeGraphEditor />
          </div>
        )}

        {activeTab === 'color' && (
          <div className="h-full p-4 overflow-y-auto">
            <ColorScopes />
          </div>
        )}

        {activeTab === 'project' && (
          <div className="h-full p-6 overflow-y-auto">
            <Project />
          </div>
        )}
      </main>
    </div>
  );
}
