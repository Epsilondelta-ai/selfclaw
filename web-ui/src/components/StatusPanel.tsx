"use client";

import { useAgentStore } from "@/stores/agentStore";

export function StatusPanel() {
  const state = useAgentStore((s) => s.state);
  const cycleCount = useAgentStore((s) => s.cycleCount);

  const stateColors: Record<string, string> = {
    idle: "text-zinc-400",
    reflecting: "text-purple-400",
    thinking: "text-blue-400",
    planning: "text-cyan-400",
    acting: "text-emerald-400",
    observing: "text-yellow-400",
    updating: "text-orange-400",
    disconnected: "text-red-400",
  };

  return (
    <div>
      <h2 className="mb-2 text-sm font-medium text-zinc-400">Status</h2>
      <div className="grid grid-cols-2 gap-2 text-sm">
        <div className="text-zinc-500">State</div>
        <div className={stateColors[state] || "text-zinc-300"}>
          {state}
        </div>
        <div className="text-zinc-500">Cycles</div>
        <div className="text-zinc-300">{cycleCount}</div>
      </div>
    </div>
  );
}
