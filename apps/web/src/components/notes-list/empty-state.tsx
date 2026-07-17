import { ListChecks, NotebookPen, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";

export function EmptyState({
  onCreateNote,
  onCreateList,
}: {
  onCreateNote: () => void;
  onCreateList: () => void;
}) {
  return (
    <div className="flex min-h-[360px] flex-col items-center justify-center rounded-lg border border-dashed border-primary/25 bg-card/45 px-6 text-center">
      <span className="grid size-14 place-items-center rounded-md border border-primary/25 bg-primary/[0.06] text-primary shadow-[0_0_30px_color-mix(in_oklch,var(--primary)_8%,transparent)]">
        <NotebookPen className="size-6" />
      </span>
      <p className="mt-6 font-heading text-[10px] font-semibold uppercase tracking-[0.16em] text-primary">Buffer empty</p>
      <h2 className="mt-2 font-heading text-lg font-semibold tracking-[-0.03em]">Start with a blank note</h2>
      <p className="mt-2 max-w-sm text-sm leading-6 text-muted-foreground">
        Capture an idea in markdown, or make a list you can check off as you go.
      </p>
      <div className="mt-6 flex gap-2">
        <Button variant="outline" onClick={onCreateList}><ListChecks /> New list</Button>
        <Button onClick={onCreateNote}><Plus /> New note</Button>
      </div>
    </div>
  );
}
