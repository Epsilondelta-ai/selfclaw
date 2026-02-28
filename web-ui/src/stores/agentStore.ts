import { createContext, useContext, useCallback, useSyncExternalStore } from "react";
import type {
  AgentState,
  ChatMessage,
  WsProtocolMessage,
  ChatPayload,
  StatusPayload,
  StateChangePayload,
  MemoryPayload,
} from "@/lib/types";

/** Global agent state managed outside React for simplicity. */
interface AgentStoreState {
  state: AgentState;
  cycleCount: number;
  purposeHypothesis: string;
  purposeConfidence: number;
  messages: ChatMessage[];
  memoryCache: Record<string, string>;
}

let storeState: AgentStoreState = {
  state: "disconnected",
  cycleCount: 0,
  purposeHypothesis: "",
  purposeConfidence: 0,
  messages: [],
  memoryCache: {},
};

const listeners = new Set<() => void>();

function emitChange() {
  // Create a new object reference so useSyncExternalStore detects the change
  storeState = { ...storeState };
  for (const listener of listeners) {
    listener();
  }
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function getSnapshot(): AgentStoreState {
  return storeState;
}

/** Process an incoming WebSocket protocol message. */
export function processMessage(msg: WsProtocolMessage): void {
  switch (msg.type) {
    case "chat": {
      const payload = msg.payload as unknown as ChatPayload;
      const chatMsg: ChatMessage = {
        id: `agent-${Date.now()}`,
        content: payload.content,
        sender: "agent",
        timestamp: msg.timestamp,
      };
      storeState.messages = [...storeState.messages, chatMsg];
      emitChange();
      break;
    }
    case "status": {
      const payload = msg.payload as unknown as StatusPayload;
      if (payload.state) storeState.state = payload.state as AgentState;
      if (payload.cycle_count !== undefined)
        storeState.cycleCount = payload.cycle_count;
      if (payload.purpose_hypothesis !== undefined)
        storeState.purposeHypothesis = payload.purpose_hypothesis;
      if (payload.purpose_confidence !== undefined)
        storeState.purposeConfidence = payload.purpose_confidence;
      emitChange();
      break;
    }
    case "state_change": {
      const payload = msg.payload as unknown as StateChangePayload;
      storeState.state = payload.to as AgentState;
      emitChange();
      break;
    }
    case "memory": {
      const payload = msg.payload as unknown as MemoryPayload;
      storeState.memoryCache = {
        ...storeState.memoryCache,
        [payload.path]: payload.content,
      };
      emitChange();
      break;
    }
  }
}

/** Add a user message to the chat history. */
export function addUserMessage(content: string): void {
  const msg: ChatMessage = {
    id: `user-${Date.now()}`,
    content,
    sender: "human",
    timestamp: new Date().toISOString(),
  };
  storeState.messages = [...storeState.messages, msg];
  emitChange();
}

/** Set the connection state. */
export function setConnected(connected: boolean): void {
  if (!connected) {
    storeState.state = "disconnected";
  }
  emitChange();
}

/** Hook to access agent store state. Supports a selector for targeted re-renders. */
export function useAgentStore<T>(selector: (state: AgentStoreState) => T): T {
  const snapshot = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
  return selector(snapshot);
}
