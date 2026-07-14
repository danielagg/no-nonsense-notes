import { NotebookText } from "lucide-react";
import { cn } from "@/lib/utils";

export function Brand({
  compact = false,
  className,
}: {
  compact?: boolean;
  className?: string;
}) {
  return (
    <div className={cn("flex items-center gap-3", className)}>
      <span className="brand-mark grid size-9 shrink-0 place-items-center rounded-md border">
        <NotebookText className="size-4.5" strokeWidth={1.8} />
      </span>
      {!compact && (
        <span className="font-heading text-[13px] font-semibold uppercase tracking-[0.08em]">
          No Nonsense <span className="brand-accent-text">/ Notes</span>
        </span>
      )}
    </div>
  );
}
