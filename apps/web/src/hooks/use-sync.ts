import { useEffect, useRef, useState, useCallback } from 'react';
import { useAuth } from '@/lib/auth-context';
import { registerPush } from '@/lib/sync-manager';
import {
  applyRemoteUpdate,
  getSyncCursor,
  setSyncCursor,
  getDeviceId,
  exportNoteBlob,
  encodePushFrame,
  decodePushResponse,
  encodePullRequest,
  decodePullResponse,
} from '@/lib/wasm';

export type SyncStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export function useSync() {
  const { token, isAuthenticated } = useAuth();
  const wsRef = useRef<WebSocket | null>(null);
  const [status, setStatus] = useState<SyncStatus>('disconnected');
  const [lastSeq, setLastSeq] = useState(0);

  const pull = useCallback(async () => {
    if (wsRef.current?.readyState !== WebSocket.OPEN) return;

    const cursor = await getSyncCursor();
    const pullText = await encodePullRequest(cursor);
    wsRef.current.send(pullText);
  }, []);

  const push = useCallback(async (docId: string, noteType: string) => {
    if (wsRef.current?.readyState !== WebSocket.OPEN) return;

    const deviceId = await getDeviceId();
    const loroBlob = await exportNoteBlob(docId);
    const frame = await encodePushFrame(docId, deviceId, noteType, loroBlob);
    wsRef.current.send(frame.buffer as ArrayBuffer);
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
    ws.binaryType = 'arraybuffer';
    wsRef.current = ws;

    ws.onopen = () => {
      ws.send(token);
      setStatus('connected');
      pull();
    };

    ws.onmessage = async (ev) => {
      if (typeof ev.data === 'string') {
        if (ev.data === 'unauthorized') {
          setStatus('error');
          ws.close();
          return;
        }
        if (ev.data.startsWith('seq:')) {
          try {
            const response = await decodePullResponse(ev.data);
            await setSyncCursor(response.currentSeq);
            setLastSeq(response.currentSeq);

            for (const entry of response.entries) {
              await applyRemoteUpdate(entry.docId, entry.noteType, entry.loroBlob);
            }
          } catch (err) {
            console.error('pull response decode failed:', err);
          }
        }
      }

      if (ev.data instanceof ArrayBuffer) {
        try {
          const seq = await decodePushResponse(new Uint8Array(ev.data));
          await setSyncCursor(seq);
          setLastSeq(seq);
        } catch (err) {
          console.error('push response decode failed:', err);
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
  }, [token, isAuthenticated, pull]);

  const disconnect = useCallback(() => {
    wsRef.current?.close();
    wsRef.current = null;
  }, []);

  useEffect(() => {
    if (isAuthenticated) {
      getSyncCursor().then(setLastSeq);
      connect();
    }
    return () => {
      disconnect();
      registerPush(null);
    };
  }, [isAuthenticated, connect, disconnect]);

  useEffect(() => {
    if (status === 'connected') {
      registerPush(push);
    } else {
      registerPush(null);
    }
  }, [status, push]);

  return { status, lastSeq, pull, push, connect, disconnect };
}
