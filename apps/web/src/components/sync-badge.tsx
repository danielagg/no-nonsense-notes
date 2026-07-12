import { useSync, type SyncStatus } from '@/hooks/use-sync';

const STATUS_LABELS: Record<SyncStatus, string> = {
  disconnected: 'Offline',
  connecting: 'Connecting...',
  connected: 'Synced',
  error: 'Sync error',
};

const STATUS_COLORS: Record<SyncStatus, string> = {
  disconnected: 'bg-muted-foreground/30',
  connecting: 'bg-yellow-500/30',
  connected: 'bg-green-500/30',
  error: 'bg-destructive/30',
};

export function SyncBadge() {
  const { status, lastSeq, pull } = useSync();

  return (
    <div className="fixed bottom-4 right-4 z-50">
      <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-card border shadow-sm text-xs">
        <span className={`w-2 h-2 rounded-full ${STATUS_COLORS[status]}`} />
        <span className="text-muted-foreground">{STATUS_LABELS[status]}</span>
        {lastSeq > 0 && (
          <button
            onClick={() => pull(lastSeq)}
            className="text-muted-foreground hover:text-foreground underline"
            title="Pull latest"
          >
            seq:{lastSeq}
          </button>
        )}
      </div>
    </div>
  );
}
