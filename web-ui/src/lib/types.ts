/** Message types for the WebSocket protocol. */
export type WsMessageType = "chat" | "status" | "memory" | "state_change";

/** A structured message sent over the WebSocket connection. */
export interface WsProtocolMessage {
  type: WsMessageType;
  payload: Record<string, unknown>;
  timestamp: string;
}

/** Chat message payload. */
export interface ChatPayload {
  content: string;
  sender?: string;
}

/** Status payload. */
export interface StatusPayload {
  state: string;
  cycle_count?: number;
  purpose_hypothesis?: string;
  purpose_confidence?: number;
}

/** Memory payload. */
export interface MemoryPayload {
  path: string;
  content: string;
}

/** State change payload. */
export interface StateChangePayload {
  from: string;
  to: string;
}

/** A chat message in the UI. */
export interface ChatMessage {
  id: string;
  content: string;
  sender: "human" | "agent" | "system";
  timestamp: string;
}

/** Agent state as tracked by the frontend. */
export type AgentState =
  | "idle"
  | "reflecting"
  | "thinking"
  | "planning"
  | "acting"
  | "observing"
  | "updating"
  | "disconnected";
