import { useEffect, useRef, useState, useCallback } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useAuth } from '@/lib/auth';
import { registerPush, loadPendingPushes } from '@/lib/sync-manager';
import {
  applyRemoteUpdate,
  applyRemoteDelete,
  getSyncCursor,
  setSyncCursor,
  getDeviceId,
  exportNoteBlob,
  encodePushFrame,
  encodeDeleteFrame,
  decodePushResponse,
  encodePullRequest,
  decodePullResponse,
} from '@/lib/wasm';

export type SyncStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

interface PendingAcknowledgement {
  resolve: () => void;
  reject: (error: Error) => void;
}

const MAX_RECONNECT_DELAY_MS = 10_000;

export function useSync() {
  const { token, accountId, isAuthenticated } = useAuth();
  const queryClient = useQueryClient();
  const wsRef = useRef<WebSocket | null>(null);
  const connectRef = useRef<() => void>(() => {});
  const reconnectTimerRef = useRef<number | null>(null);
  const reconnectAttemptRef = useRef(0);
  const shouldReconnectRef = useRef(false);
  const pendingAcksRef = useRef<PendingAcknowledgement[]>([]);
  const pullInFlightRef = useRef(false);
  const pullAgainRef = useRef(false);
  const [status, setStatus] = useState<SyncStatus>('disconnected');
  const [lastSeq, setLastSeq] = useState(0);

  const rejectPendingAcknowledgements = useCallback((message: string) => {
    const error = new Error(message);
    for (const acknowledgement of pendingAcksRef.current.splice(0)) {
      acknowledgement.reject(error);
    }
  }, []);

  const pull = useCallback(async () => {
    const ws = wsRef.current;
    if (ws?.readyState !== WebSocket.OPEN) return;
    if (pullInFlightRef.current) {
      pullAgainRef.current = true;
      return;
    }

    pullInFlightRef.current = true;
    try {
      const cursor = await getSyncCursor();
      const pullText = await encodePullRequest(cursor);
      ws.send(pullText);
    } catch (error) {
      pullInFlightRef.current = false;
      throw error;
    }
  }, []);

  const push = useCallback(async (docId: string, noteType: string) => {
    const ws = wsRef.current;
    if (ws?.readyState !== WebSocket.OPEN) {
      throw new Error('sync socket is not connected');
    }

    const deviceId = await getDeviceId();
    const frame = noteType === 'delete'
      ? await encodeDeleteFrame(docId, deviceId)
      : await encodePushFrame(docId, deviceId, noteType, await exportNoteBlob(docId));

    await new Promise<void>((resolve, reject) => {
      const acknowledgement = { resolve, reject };
      pendingAcksRef.current.push(acknowledgement);
      try {
        ws.send(frame.buffer as ArrayBuffer);
      } catch (error) {
        const index = pendingAcksRef.current.indexOf(acknowledgement);
        if (index >= 0) pendingAcksRef.current.splice(index, 1);
        reject(error instanceof Error ? error : new Error(String(error)));
      }
    });
  }, []);

  const connect = useCallback(() => {
    if (!isAuthenticated || !token) return;
    if (wsRef.current && wsRef.current.readyState <= WebSocket.OPEN) return;

    if (reconnectTimerRef.current !== null) {
      window.clearTimeout(reconnectTimerRef.current);
      reconnectTimerRef.current = null;
    }

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
    };

    ws.onmessage = async (event) => {
      if (typeof event.data === 'string') {
        if (event.data === 'unauthorized') {
          setStatus('error');
          shouldReconnectRef.current = false;
          ws.close();
          return;
        }

        if (event.data === 'ready') {
          reconnectAttemptRef.current = 0;
          setStatus('connected');
          void pull();
          return;
        }

        if (event.data.startsWith('update:')) {
          void pull();
          return;
        }

        if (event.data.startsWith('seq:')) {
          let shouldPullAgain = false;
          try {
            const response = await decodePullResponse(event.data);
            for (const entry of response.entries) {
              if (entry.deleted) {
                await applyRemoteDelete(entry.docId);
              } else if (entry.noteType) {
                await applyRemoteUpdate(entry.docId, entry.noteType, entry.loroBlob);
              } else {
                throw new Error(`note update ${entry.docId} is missing its note type`);
              }
            }

            // Advance only after every update was applied successfully.
            await setSyncCursor(response.currentSeq);
            setLastSeq(response.currentSeq);
            if (response.entries.length > 0) {
              await queryClient.invalidateQueries({ queryKey: ['notes'] });
            }
            shouldPullAgain = response.entries.length === 1000 || pullAgainRef.current;
          } catch (error) {
            console.error('pull response handling failed:', error);
          } finally {
            pullInFlightRef.current = false;
            pullAgainRef.current = false;
          }

          if (shouldPullAgain) void pull();
        }
      }

      if (event.data instanceof ArrayBuffer) {
        try {
          await decodePushResponse(new Uint8Array(event.data));
          const acknowledgement = pendingAcksRef.current.shift();
          acknowledgement?.resolve();
          // Acknowledgements are not pull cursors. Pull the committed row instead.
          void pull();
        } catch (error) {
          const acknowledgement = pendingAcksRef.current.shift();
          acknowledgement?.reject(error instanceof Error ? error : new Error(String(error)));
          console.error('push response decode failed:', error);
        }
      }
    };

    ws.onclose = () => {
      if (wsRef.current !== ws) return;

      wsRef.current = null;
      pullInFlightRef.current = false;
      pullAgainRef.current = false;
      rejectPendingAcknowledgements('sync socket closed before acknowledgement');
      registerPush(null);
      setStatus('disconnected');

      if (shouldReconnectRef.current) {
        const delay = Math.min(
          1000 * 2 ** reconnectAttemptRef.current,
          MAX_RECONNECT_DELAY_MS,
        );
        reconnectAttemptRef.current += 1;
        reconnectTimerRef.current = window.setTimeout(() => connectRef.current(), delay);
      }
    };

    ws.onerror = () => {
      if (wsRef.current === ws) {
        setStatus('error');
        ws.close();
      }
    };
  }, [isAuthenticated, token, pull, queryClient, rejectPendingAcknowledgements]);

  connectRef.current = connect;

  const disconnect = useCallback(() => {
    shouldReconnectRef.current = false;
    if (reconnectTimerRef.current !== null) {
      window.clearTimeout(reconnectTimerRef.current);
      reconnectTimerRef.current = null;
    }
    wsRef.current?.close();
    wsRef.current = null;
    rejectPendingAcknowledgements('sync disconnected');
    registerPush(null);
    setStatus('disconnected');
  }, [rejectPendingAcknowledgements]);

  useEffect(() => {
    if (isAuthenticated && accountId) {
      shouldReconnectRef.current = true;
      loadPendingPushes(accountId);
      getSyncCursor().then(setLastSeq);
      connectRef.current();
    }
    return () => {
      disconnect();
      registerPush(null);
    };
  }, [isAuthenticated, accountId, disconnect]);

  useEffect(() => {
    registerPush(status === 'connected' ? push : null);
    return () => registerPush(null);
  }, [status, push]);

  return { status, lastSeq, pull, push, connect, disconnect };
}
