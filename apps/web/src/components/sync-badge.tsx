import { useSync, type SyncStatus } from "@/hooks/use-sync";

const STATUS_LABELS: Record<SyncStatus, string> = {
  disconnected: "Offline",
  connecting: "Connecting...",
  connected: "Synced",
  error: "Sync error",
};

const STATUS_COLORS: Record<SyncStatus, string> = {
  disconnected: "bg-muted-foreground/50",
  connecting: "bg-amber-500",
  connected: "bg-emerald-500",
  error: "bg-destructive",
};

export function SyncBadge() {
  const { status } = useSync();

  return (
    <div className="fixed bottom-4 right-4 z-50 sm:bottom-6 sm:right-6">
      <div className="flex items-center gap-2 rounded-full border bg-card/90 px-3 py-2 text-[11px] font-medium shadow-lg shadow-foreground/[0.07] backdrop-blur-xl">
        <span
          className={`size-1.5 rounded-full ${STATUS_COLORS[status]} ${status === "connecting" ? "animate-pulse" : ""}`}
        />
        <span className="text-foreground/70">{STATUS_LABELS[status]}</span>
      </div>
    </div>
  );
}
