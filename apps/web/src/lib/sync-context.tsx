import { createContext, useContext, useCallback, type ReactNode } from 'react';
import { useSync, type SyncStatus } from '@/hooks/use-sync';
import { exportSnapshot } from '@/lib/loro-store';

interface SyncState {
  status: SyncStatus;
  lastSeq: number;
  pushNote: (noteId: string) => void;
  pushDelete: (noteId: string) => void;
  pullAll: () => void;
}

const SyncContext = createContext<SyncState | null>(null);

export function SyncProvider({
  children,
  onSyncComplete,
}: {
  children: ReactNode;
  onSyncComplete?: () => void;
}) {
  const { status, lastSeq, pull, push, pushDelete } = useSync(onSyncComplete);

  const pushNote = useCallback(
    (noteId: string) => {
      const snapshot = exportSnapshot(noteId);
      if (snapshot) {
        push(noteId, snapshot);
      }
    },
    [push],
  );

  const pullAll = useCallback(() => {
    pull(0);
  }, [pull]);

  return (
    <SyncContext.Provider value={{ status, lastSeq, pushNote, pushDelete, pullAll }}>
      {children}
    </SyncContext.Provider>
  );
}

export function useSyncState() {
  const ctx = useContext(SyncContext);
  if (!ctx) throw new Error('useSyncState must be used within SyncProvider');
  return ctx;
}
