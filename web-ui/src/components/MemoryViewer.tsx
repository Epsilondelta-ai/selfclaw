"use client";

import { useAgentStore } from "@/stores/agentStore";

export function MemoryViewer() {
  const memoryCache = useAgentStore((s) => s.memoryCache);
  const paths = Object.keys(memoryCache);

  return (
    <div>
      <h2 className="mb-2 text-sm font-medium text-zinc-400">Memory</h2>
      {paths.length === 0 ? (
        <p className="text-sm text-zinc-600">
          No memory files loaded yet. Memory will appear here as the agent operates.
        </p>
      ) : (
        <div className="space-y-3">
          {paths.map((path) => (
            <div key={path} className="rounded border border-zinc-800 p-3">
              <h3 className="mb-1 text-xs font-mono text-zinc-500">{path}</h3>
              <pre className="whitespace-pre-wrap text-xs text-zinc-300">
                {memoryCache[path]}
              </pre>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
