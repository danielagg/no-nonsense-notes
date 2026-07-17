import { Trash2 } from "lucide-react";
import type { Note } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardTitle } from "@/components/ui/card";

export function NoteCard({
  note,
  onOpen,
  onDelete,
}: {
  note: Note;
  onOpen: () => void;
  onDelete: () => void;
}) {
  const isMarkdown = note.type === "markdown";
  const preview = isMarkdown
    ? note.content.replace(/[#*_`]/g, "").trim().slice(0, 160)
    : `${note.items?.length ?? 0} item${note.items?.length === 1 ? "" : "s"}`;

  return (
    <Card className="terminal-glow group relative min-h-48 cursor-pointer gap-0 rounded-lg border border-primary/12 bg-card/88 py-0 ring-0 transition-all duration-200 hover:-translate-y-0.5 hover:border-primary/45 hover:shadow-[0_0_28px_color-mix(in_oklch,var(--primary)_7%,transparent)]">
      <button type="button" className="absolute inset-0 z-0 rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-primary/35" onClick={onOpen} aria-label={`Open ${note.title || "Untitled note"}`} />
      <CardContent className="pointer-events-none relative z-[1] flex flex-1 flex-col p-8">
        <CardTitle className="line-clamp-1 font-heading text-[17px] font-semibold flex items-center justify-between">
          <p>{note.title || "Untitled note"}</p>
          <Button
            variant="ghost"
            size="icon-sm"
            className="pointer-events-auto relative z-10 text-muted-foreground opacity-60 hover:bg-destructive/10 hover:text-destructive"
            onClick={(event) => { event.stopPropagation(); onDelete(); }}
            aria-label={`Delete ${note.title || "Untitled note"}`}
            title="Delete note"
          >
            <Trash2 />
          </Button>
        </CardTitle>
        <p className="mt-2 line-clamp-3 whitespace-pre-wrap text-sm leading-6 text-muted-foreground">{preview || "No content yet."}</p>
        <div className="mt-auto flex items-center justify-between pt-5 font-heading text-[9px] uppercase tracking-[0.12em] text-muted-foreground">
          <span>{isMarkdown ? "Markdown" : "Checklist"}</span>
          <span>{formatUpdatedAt(note.updated_at)}</span>
        </div>
      </CardContent>
    </Card>
  );
}

function formatUpdatedAt(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "Recently updated";
  return new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric" }).format(date);
}
