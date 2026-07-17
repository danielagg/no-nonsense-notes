import { FileText, ListChecks } from "lucide-react";
import type { Note } from "@/lib/api";
import type { MarkdownMode } from "./model";

export function EditorMeta({
  note,
  markdownMode,
  onMarkdownModeChange,
}: {
  note: Note;
  markdownMode: MarkdownMode;
  onMarkdownModeChange: (mode: MarkdownMode) => void;
}) {
  return (
    <div className="mb-4 flex items-center justify-between px-1 font-heading text-[10px] uppercase tracking-[0.1em] text-muted-foreground">
      <div className="flex items-center gap-2 text-primary/80">
        {note.type === "markdown" ? <FileText className="size-3.5" /> : <ListChecks className="size-3.5" />}
        <span>{note.type === "markdown" ? "Markdown // buffer" : "List // buffer"}</span>
      </div>
      <div className="flex items-center gap-3">
        {note.type === "markdown" && (
          <div role="tablist" aria-label="Markdown display mode" className="flex rounded-sm border border-primary/15 bg-background/55 p-0.5">
            {(["edit", "preview"] as const).map((mode) => (
              <button
                key={mode}
                type="button"
                role="tab"
                aria-selected={markdownMode === mode}
                className={`rounded-sm px-2 py-1 font-heading text-[9px] font-semibold uppercase tracking-[0.1em] transition-colors ${markdownMode === mode ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground"}`}
                onClick={() => onMarkdownModeChange(mode)}
              >
                {mode}
              </button>
            ))}
          </div>
        )}
        <span className="hidden sm:inline">Modified // {new Date(note.updated_at).toLocaleString()}</span>
      </div>
    </div>
  );
}
