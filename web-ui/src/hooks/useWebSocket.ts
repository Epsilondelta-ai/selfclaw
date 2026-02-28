"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import type { WsProtocolMessage } from "@/lib/types";
import { processMessage, setConnected, addUserMessage } from "@/stores/agentStore";

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:3000";
const RECONNECT_DELAY_MS = 3000;
const MAX_RECONNECT_DELAY_MS = 30000;

interface UseWebSocketReturn {
  connected: boolean;
  send: (content: string) => void;
}

export function useWebSocket(): UseWebSocketReturn {
  const [connected, setConnectedState] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectDelayRef = useRef(RECONNECT_DELAY_MS);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const mountedRef = useRef(true);

  const connect = useCallback(() => {
    if (!mountedRef.current) return;
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    try {
      const ws = new WebSocket(WS_URL);

      ws.onopen = () => {
        if (!mountedRef.current) return;
        setConnectedState(true);
        setConnected(true);
        reconnectDelayRef.current = RECONNECT_DELAY_MS;
      };

      ws.onclose = () => {
        if (!mountedRef.current) return;
        setConnectedState(false);
        setConnected(false);
        wsRef.current = null;

        // Schedule reconnect with exponential backoff
        const delay = reconnectDelayRef.current;
        reconnectDelayRef.current = Math.min(
          delay * 2,
          MAX_RECONNECT_DELAY_MS
        );
        reconnectTimeoutRef.current = setTimeout(connect, delay);
      };

      ws.onerror = () => {
        // onclose will fire after onerror
      };

      ws.onmessage = (event) => {
        try {
          const msg: WsProtocolMessage = JSON.parse(event.data);
          processMessage(msg);
        } catch {
          // Ignore malformed messages
        }
      };

      wsRef.current = ws;
    } catch {
      // Connection failed, will retry via onclose
    }
  }, []);

  useEffect(() => {
    mountedRef.current = true;
    connect();

    return () => {
      mountedRef.current = false;
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, [connect]);

  const send = useCallback((content: string) => {
    if (!content.trim()) return;

    // Add to local chat history
    addUserMessage(content);

    // Send over WebSocket
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      const msg: WsProtocolMessage = {
        type: "chat",
        payload: { content, sender: "web-user" },
        timestamp: new Date().toISOString(),
      };
      wsRef.current.send(JSON.stringify(msg));
    }
  }, []);

  return { connected, send };
}
