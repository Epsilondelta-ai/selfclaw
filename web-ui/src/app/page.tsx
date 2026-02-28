"use client";

import { ChatPanel } from "@/components/ChatPanel";
import { StatusPanel } from "@/components/StatusPanel";
import { MemoryViewer } from "@/components/MemoryViewer";
import { PurposeTracker } from "@/components/PurposeTracker";
import { useWebSocket } from "@/hooks/useWebSocket";
import { useAgentStore } from "@/stores/agentStore";

export default function Home() {
  const { connected, send } = useWebSocket();
  const agentState = useAgentStore((s) => s.state);

  return (
    <div className="flex h-screen flex-col">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-zinc-800 px-6 py-3">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-semibold tracking-tight">SelfClaw</h1>
          <span className="text-xs text-zinc-500">autonomous agent</span>
        </div>
        <div className="flex items-center gap-3">
          <span
            className={`inline-block h-2 w-2 rounded-full ${
              connected ? "bg-emerald-500" : "bg-red-500"
            }`}
          />
          <span className="text-xs text-zinc-400">
            {connected ? "connected" : "disconnected"}
          </span>
          <span className="text-xs text-zinc-600">|</span>
          <span className="text-xs text-zinc-400">{agentState}</span>
        </div>
      </header>

      {/* Main content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left panel: Chat */}
        <div className="flex w-1/2 flex-col border-r border-zinc-800">
          <ChatPanel send={send} />
        </div>

        {/* Right panel: Status + Memory + Purpose */}
        <div className="flex w-1/2 flex-col">
          <div className="border-b border-zinc-800 p-4">
            <PurposeTracker />
          </div>
          <div className="border-b border-zinc-800 p-4">
            <StatusPanel />
          </div>
          <div className="flex-1 overflow-auto p-4">
            <MemoryViewer />
          </div>
        </div>
      </div>
    </div>
  );
}
