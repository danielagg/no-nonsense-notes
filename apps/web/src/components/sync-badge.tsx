import type { SyncStatus } from "@/hooks/use-sync";
import { LoaderCircle, TriangleAlert, WifiOff } from "lucide-react";

const STATUS_CONTENT: Record<
  Exclude<SyncStatus, "connected">,
  { label: string; detail: string }
> = {
  disconnected: {
    label: "You're offline",
    detail: "Changes are saved locally and will sync when the connection returns.",
  },
  connecting: {
    label: "Connecting to sync…",
    detail: "Your changes are safe while we reconnect.",
  },
  error: {
    label: "Sync issue",
    detail: "Your changes are safe locally. We'll keep trying to reconnect.",
  },
};

const STATUS_STYLES: Record<Exclude<SyncStatus, "connected">, string> = {
  disconnected:
    "border-amber-500/25 bg-amber-50 text-amber-950 dark:bg-amber-950 dark:text-amber-100",
  connecting:
    "border-sky-500/25 bg-sky-50 text-sky-950 dark:bg-sky-950 dark:text-sky-100",
  error:
    "border-destructive/25 bg-red-50 text-red-950 dark:bg-red-950 dark:text-red-100",
};

export function SyncStatusBar({ status }: { status: SyncStatus }) {
  if (status === "connected") return null;

  const content = STATUS_CONTENT[status];
  const icon =
    status === "connecting" ? (
      <LoaderCircle className="size-3.5 animate-spin" />
    ) : status === "error" ? (
      <TriangleAlert className="size-3.5" />
    ) : (
      <WifiOff className="size-3.5" />
    );

  return (
    <div
      className={`fixed inset-x-0 top-0 z-50 flex h-10 items-center justify-center border-b px-4 text-xs shadow-sm ${STATUS_STYLES[status]}`}
      role="status"
      aria-live="polite"
      data-sync-status={status}
    >
      <div className="flex min-w-0 items-center gap-2.5">
        <span className="shrink-0" aria-hidden="true">
          {icon}
        </span>
        <span className="shrink-0 font-semibold">{content.label}</span>
        <span className="hidden truncate opacity-75 sm:inline">{content.detail}</span>
      </div>
    </div>
  );
}
