"use client";

import { useState, useRef, useEffect } from "react";
import { useAgentStore } from "@/stores/agentStore";

interface ChatPanelProps {
  send: (content: string) => void;
}

export function ChatPanel({ send }: ChatPanelProps) {
  const messages = useAgentStore((s) => s.messages);
  const [input, setInput] = useState("");
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!input.trim()) return;
    send(input.trim());
    setInput("");
  }

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-zinc-800 px-4 py-2">
        <h2 className="text-sm font-medium text-zinc-400">Chat</h2>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-auto p-4 space-y-3">
        {messages.length === 0 && (
          <p className="text-sm text-zinc-600">
            No messages yet. Say something to SelfClaw.
          </p>
        )}
        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`flex ${
              msg.sender === "human" ? "justify-end" : "justify-start"
            }`}
          >
            <div
              className={`max-w-[80%] rounded-lg px-3 py-2 text-sm ${
                msg.sender === "human"
                  ? "bg-blue-600 text-white"
                  : msg.sender === "agent"
                  ? "bg-zinc-800 text-zinc-200"
                  : "bg-zinc-900 text-zinc-500 italic"
              }`}
            >
              <p>{msg.content}</p>
              <span className="mt-1 block text-[10px] opacity-50">
                {new Date(msg.timestamp).toLocaleTimeString()}
              </span>
            </div>
          </div>
        ))}
        <div ref={bottomRef} />
      </div>

      {/* Input */}
      <form onSubmit={handleSubmit} className="border-t border-zinc-800 p-3">
        <div className="flex gap-2">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Talk to SelfClaw..."
            className="flex-1 rounded-lg border border-zinc-700 bg-zinc-900 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-600 focus:border-blue-500 focus:outline-none"
          />
          <button
            type="submit"
            className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 transition-colors"
          >
            Send
          </button>
        </div>
      </form>
    </div>
  );
}
