"use client";

import { useAgentStore } from "@/stores/agentStore";

export function PurposeTracker() {
  const hypothesis = useAgentStore((s) => s.purposeHypothesis);
  const confidence = useAgentStore((s) => s.purposeConfidence);

  return (
    <div>
      <h2 className="mb-2 text-sm font-medium text-zinc-400">
        Purpose Hypothesis
      </h2>
      {hypothesis ? (
        <div>
          <p className="text-sm text-zinc-200">{hypothesis}</p>
          <div className="mt-2 flex items-center gap-2">
            <div className="h-1.5 flex-1 rounded-full bg-zinc-800">
              <div
                className="h-1.5 rounded-full bg-blue-500 transition-all"
                style={{ width: `${Math.round(confidence * 100)}%` }}
              />
            </div>
            <span className="text-xs text-zinc-500">
              {Math.round(confidence * 100)}%
            </span>
          </div>
        </div>
      ) : (
        <p className="text-sm italic text-zinc-600">
          No purpose hypothesis yet. SelfClaw is still searching.
        </p>
      )}
    </div>
  );
}
