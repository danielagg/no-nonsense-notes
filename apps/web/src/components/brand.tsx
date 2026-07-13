import { SquarePen } from "lucide-react";
import { cn } from "@/lib/utils";

export function Brand({
  compact = false,
  className,
}: {
  compact?: boolean;
  className?: string;
}) {
  return (
    <div className={cn("flex items-center gap-2.5", className)}>
      <span className="grid size-9 shrink-0 place-items-center rounded-xl bg-primary text-primary-foreground shadow-sm shadow-primary/20">
        <SquarePen className="size-4.5" strokeWidth={2.2} />
      </span>
      {!compact && (
        <span className="font-heading text-[15px] font-semibold tracking-[-0.02em]">
          No Nonsense Notes
        </span>
      )}
    </div>
  );
}
