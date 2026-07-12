import { useEffect, useRef, useState, useCallback } from 'react';
import { useAuth } from '@/lib/auth-context';

export type SyncStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

interface SyncEntry {
  docId: string;
  blob: string; // base64
  globalSeq: number;
}

export function useSync() {
  const { token, isAuthenticated } = useAuth();
  const wsRef = useRef<WebSocket | null>(null);
  const [status, setStatus] = useState<SyncStatus>('disconnected');
  const [lastSeq, setLastSeq] = useState(0);
  const [entries, setEntries] = useState<SyncEntry[]>([]);

  const connect = useCallback(() => {
    if (!isAuthenticated || !token) return;

    setStatus('connecting');
    const serverUrl = import.meta.env.VITE_API_URL;
    const wsUrl = serverUrl
      ? serverUrl.replace(/^http/, 'ws')
      : `${location.protocol === 'https:' ? 'wss:' : 'ws:'}//${location.host}`;
    const ws = new WebSocket(`${wsUrl}/sync`);
    wsRef.current = ws;

    ws.onopen = () => {
      // Send auth token as first message
      ws.send(token);
    };

    ws.onmessage = (ev) => {
      const data = ev.data;
      if (typeof data === 'string') {
        if (data === 'unauthorized') {
          setStatus('error');
          ws.close();
          return;
        }
        // Parse pull response: "seq:N\ndoc_id:base64\n..."
        if (data.startsWith('seq:')) {
          const lines = data.split('\n');
          const seqMatch = lines[0].match(/^seq:(\d+)$/);
          if (seqMatch) {
            setLastSeq(parseInt(seqMatch[1], 10));
          }
          const newEntries: SyncEntry[] = [];
          for (let i = 1; i < lines.length; i++) {
            const colonIdx = lines[i].indexOf(':');
            if (colonIdx > 0) {
              newEntries.push({
                docId: lines[i].substring(0, colonIdx),
                blob: lines[i].substring(colonIdx + 1),
                globalSeq: 0,
              });
            }
          }
          if (newEntries.length > 0) {
            setEntries((prev) => [...prev, ...newEntries]);
          }
        }
      }
      // Binary push response: 8 bytes global_seq LE
      if (data instanceof ArrayBuffer) {
        const view = new DataView(data);
        if (data.byteLength === 8) {
          const seq = Number(view.getBigUint64(0, true));
          setLastSeq(seq);
        }
      }
    };

    ws.onclose = () => {
      setStatus('disconnected');
      wsRef.current = null;
    };

    ws.onerror = () => {
      setStatus('error');
    };
  }, [token, isAuthenticated]);

  const disconnect = useCallback(() => {
    wsRef.current?.close();
    wsRef.current = null;
  }, []);

  const pull = useCallback((since: number) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(`pull:${since}`);
    }
  }, []);

  const push = useCallback((docId: string, deviceId: string, blob: Uint8Array) => {
    if (wsRef.current?.readyState !== WebSocket.OPEN) return;

    // Binary format: [version:1][type:1][doc_id:16][device_id:16][blob_len:4][blob:N]
    const docIdBytes = hexToBytes(docId);
    const deviceIdBytes = hexToBytes(deviceId);
    const payload = new Uint8Array(2 + 16 + 16 + 4 + blob.length);
    payload[0] = 1; // version
    payload[1] = 1; // push
    payload.set(docIdBytes, 2);
    payload.set(deviceIdBytes, 18);
    new DataView(payload.buffer).setUint32(34, blob.length, true);
    payload.set(blob, 38);
    wsRef.current.send(payload);
  }, []);

  // Auto-connect when authenticated
  useEffect(() => {
    if (isAuthenticated) {
      connect();
    }
    return () => disconnect();
  }, [isAuthenticated, connect, disconnect]);

  return { status, lastSeq, entries, connect, disconnect, pull, push };
}

function hexToBytes(hex: string): Uint8Array {
  const clean = hex.replace(/-/g, '');
  const bytes = new Uint8Array(16);
  for (let i = 0; i < 16; i++) {
    bytes[i] = parseInt(clean.substring(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}
