import { useEffect, useRef, useState, useCallback } from 'react';
import { useAuth } from '@/lib/auth-context';
import { importSnapshot, deleteNote, getDeviceId } from '@/lib/loro-store';

export type SyncStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export function useSync(onSyncComplete?: () => void) {
  const { token, isAuthenticated } = useAuth();
  const wsRef = useRef<WebSocket | null>(null);
  const [status, setStatus] = useState<SyncStatus>('disconnected');
  const [lastSeq, setLastSeq] = useState(() => {
    const stored = localStorage.getItem('nnn-last-seq');
    return stored ? parseInt(stored, 10) : 0;
  });

  const persistSeq = useCallback((seq: number) => {
    setLastSeq(seq);
    localStorage.setItem('nnn-last-seq', String(seq));
  }, []);

  const connect = useCallback(() => {
    if (!isAuthenticated || !token) return;
    if (wsRef.current && wsRef.current.readyState <= WebSocket.OPEN) return;

    setStatus('connecting');
    const serverUrl = import.meta.env.VITE_API_URL;
    const wsUrl = serverUrl
      ? serverUrl.replace(/^http/, 'ws')
      : `${location.protocol === 'https:' ? 'wss:' : 'ws:'}//${location.host}`;
    const ws = new WebSocket(`${wsUrl}/sync`);
    wsRef.current = ws;

    ws.onopen = () => {
      ws.send(token);
      setStatus('connected');
      // Auto-pull everything from seq 0 on connect
      const stored = localStorage.getItem('nnn-last-seq');
      const fromSeq = stored ? parseInt(stored, 10) : 0;
      ws.send(`pull:${fromSeq}`);
    };

    ws.onmessage = (ev) => {
      const data = ev.data;

      // Text messages: pull responses
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
            persistSeq(parseInt(seqMatch[1], 10));
          }

          // Import each entry's snapshot into local store
          let imported = false;
          for (let i = 1; i < lines.length; i++) {
            const colonIdx = lines[i].indexOf(':');
            if (colonIdx > 0) {
              const docId = lines[i].substring(0, colonIdx);
              const b64 = lines[i].substring(colonIdx + 1);
              const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
              if (bytes.length === 0) {
                deleteNote(docId);
              } else {
                importSnapshot(docId, bytes);
              }
              imported = true;
            }
          }

          if (imported && onSyncComplete) {
            onSyncComplete();
          }
        }
      }

      // Binary push response: 8 bytes global_seq LE
      if (data instanceof ArrayBuffer) {
        const view = new DataView(data);
        if (data.byteLength === 8) {
          const seq = Number(view.getBigUint64(0, true));
          persistSeq(seq);
        }
      }
    };

    ws.onclose = () => {
      if (wsRef.current === ws) {
        setStatus('disconnected');
        wsRef.current = null;
      }
    };

    ws.onerror = () => {
      if (wsRef.current === ws) {
        setStatus('error');
      }
    };
  }, [token, isAuthenticated, persistSeq, onSyncComplete]);

  const disconnect = useCallback(() => {
    wsRef.current?.close();
    wsRef.current = null;
  }, []);

  const pull = useCallback((since: number) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(`pull:${since}`);
    }
  }, []);

  const push = useCallback((docId: string, snapshot: Uint8Array) => {
    if (wsRef.current?.readyState !== WebSocket.OPEN) return;

    const deviceId = getDeviceId();
    const docIdBytes = hexToBytes(docId);
    const deviceIdBytes = hexToBytes(deviceId);

    // Binary format: [version:1][type:1][doc_id:16][device_id:16][blob_len:4][blob:N]
    const payload = new Uint8Array(2 + 16 + 16 + 4 + snapshot.length);
    payload[0] = 1; // version
    payload[1] = 1; // push
    payload.set(docIdBytes, 2);
    payload.set(deviceIdBytes, 18);
    new DataView(payload.buffer).setUint32(34, snapshot.length, true);
    payload.set(snapshot, 38);
    wsRef.current.send(payload);
  }, []);

  const pushDelete = useCallback((docId: string) => {
    if (wsRef.current?.readyState !== WebSocket.OPEN) return;

    const deviceId = getDeviceId();
    const docIdBytes = hexToBytes(docId);
    const deviceIdBytes = hexToBytes(deviceId);

    // Same format as push but blob_len=0, no blob bytes
    const payload = new Uint8Array(2 + 16 + 16 + 4);
    payload[0] = 1; // version
    payload[1] = 1; // push
    payload.set(docIdBytes, 2);
    payload.set(deviceIdBytes, 18);
    new DataView(payload.buffer).setUint32(34, 0, true);
    wsRef.current.send(payload);
  }, []);

  // Auto-connect when authenticated
  useEffect(() => {
    if (isAuthenticated) {
      connect();
    }
    return () => disconnect();
  }, [isAuthenticated, connect, disconnect]);

  return { status, lastSeq, connect, disconnect, pull, push, pushDelete };
}

function hexToBytes(hex: string): Uint8Array {
  const clean = hex.replace(/-/g, '');
  const bytes = new Uint8Array(16);
  for (let i = 0; i < 16; i++) {
    bytes[i] = parseInt(clean.substring(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}
