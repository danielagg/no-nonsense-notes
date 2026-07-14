import { useRef, useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { getNotes, createNote, deleteNote, type Note } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useAuth } from "@/lib/auth";
import { NoteEditor } from "./note-editor";
import { Brand } from "./brand";
import { ThemeToggle } from "./theme-toggle";
import {
  FileText,
  ListChecks,
  LogOut,
  MoreHorizontal,
  NotebookPen,
  Plus,
  Trash2,
} from "lucide-react";

const SIDEBAR_WIDTH_STORAGE_KEY = "nnn-sidebar-width";
const DEFAULT_SIDEBAR_WIDTH = 248;
const MIN_SIDEBAR_WIDTH = 220;
const MAX_SIDEBAR_WIDTH = 420;

function clampSidebarWidth(width: number) {
  return Math.min(MAX_SIDEBAR_WIDTH, Math.max(MIN_SIDEBAR_WIDTH, width));
}

function getStoredSidebarWidth() {
  const storedWidth = Number(localStorage.getItem(SIDEBAR_WIDTH_STORAGE_KEY));
  return Number.isFinite(storedWidth)
    ? clampSidebarWidth(storedWidth)
    : DEFAULT_SIDEBAR_WIDTH;
}

export function NotesList() {
  const { logout, accountId } = useAuth();
  const queryClient = useQueryClient();
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null);
  const [sidebarWidth, setSidebarWidth] = useState(getStoredSidebarWidth);
  const [isResizingSidebar, setIsResizingSidebar] = useState(false);
  const resizeStart = useRef<{ x: number; width: number } | null>(null);

  const updateSidebarWidth = (width: number, persist = false) => {
    const nextWidth = clampSidebarWidth(width);
    setSidebarWidth(nextWidth);

    if (persist) {
      localStorage.setItem(SIDEBAR_WIDTH_STORAGE_KEY, String(nextWidth));
    }
  };

  const finishSidebarResize = (clientX: number) => {
    if (!resizeStart.current) return;

    const { x, width } = resizeStart.current;
    updateSidebarWidth(width + clientX - x, true);
    resizeStart.current = null;
    setIsResizingSidebar(false);
  };

  const { data: notes = [] } = useQuery({
    queryKey: ["notes", accountId],
    queryFn: getNotes,
  });
  const selectedNote = selectedNoteId
    ? notes.find((note) => note.id === selectedNoteId)
    : null;

  const createMutation = useMutation({
    mutationFn: (type: "markdown" | "list") =>
      Promise.resolve(createNote(type)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["notes"] }),
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      await deleteNote(id);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["notes"] });
      setSelectedNoteId(null);
    },
  });

  if (selectedNote) {
    return (
      <NoteEditor note={selectedNote} onBack={() => setSelectedNoteId(null)} />
    );
  }

  return (
    <div
      className="terminal-grid min-h-[calc(100svh-var(--sync-banner-height))] md:grid md:grid-cols-[var(--sidebar-width)_minmax(0,1fr)]"
      style={{ "--sidebar-width": `${sidebarWidth}px` } as React.CSSProperties}
    >
      <aside className="relative hidden h-[calc(100svh-var(--sync-banner-height))] flex-col border-r border-primary/15 bg-sidebar/92 px-3 py-4 backdrop-blur md:sticky md:top-[var(--sync-banner-height)] md:flex">
        <Brand className="px-2" />

        <nav className="mt-8">
          <div className="flex h-10 items-center gap-3 rounded-md border border-primary/20 bg-primary/[0.07] px-3 font-heading text-xs font-semibold uppercase tracking-[0.045em] text-sidebar-accent-foreground">
            <NotebookPen className="size-4 text-primary" />
            All notes
            <span className="ml-auto border border-primary/15 bg-background/50 px-1.5 py-0.5 font-mono text-[10px] tabular-nums text-primary">
              {notes.length}
            </span>
          </div>
        </nav>

        <div className="mt-auto space-y-1">
          <ThemeToggle showLabel />
          <Button
            variant="ghost"
            className="w-full justify-start"
            onClick={logout}
          >
            <LogOut />
            Log out
          </Button>
        </div>

        <div
          role="separator"
          aria-label="Resize notes sidebar"
          aria-orientation="vertical"
          aria-valuemin={MIN_SIDEBAR_WIDTH}
          aria-valuemax={MAX_SIDEBAR_WIDTH}
          aria-valuenow={sidebarWidth}
          tabIndex={0}
          className={`absolute -right-1 top-0 z-20 hidden h-full w-2 touch-none select-none cursor-col-resize md:block ${
            isResizingSidebar ? "bg-primary/25" : "hover:bg-primary/12"
          }`}
          onPointerDown={(event) => {
            event.currentTarget.setPointerCapture(event.pointerId);
            resizeStart.current = { x: event.clientX, width: sidebarWidth };
            setIsResizingSidebar(true);
          }}
          onPointerMove={(event) => {
            if (!resizeStart.current) return;

            updateSidebarWidth(
              resizeStart.current.width + event.clientX - resizeStart.current.x,
            );
          }}
          onPointerUp={(event) => finishSidebarResize(event.clientX)}
          onPointerCancel={(event) => finishSidebarResize(event.clientX)}
          onKeyDown={(event) => {
            const step = event.shiftKey ? 32 : 16;

            if (event.key === "ArrowLeft") {
              event.preventDefault();
              updateSidebarWidth(sidebarWidth - step, true);
            }

            if (event.key === "ArrowRight") {
              event.preventDefault();
              updateSidebarWidth(sidebarWidth + step, true);
            }

            if (event.key === "Home") {
              event.preventDefault();
              updateSidebarWidth(MIN_SIDEBAR_WIDTH, true);
            }

            if (event.key === "End") {
              event.preventDefault();
              updateSidebarWidth(MAX_SIDEBAR_WIDTH, true);
            }
          }}
        />
      </aside>

      <div className="min-w-0">
        <header className="flex h-16 items-center justify-between border-b border-primary/10 bg-background/85 px-4 backdrop-blur md:hidden">
          <Brand />
          <div className="flex items-center gap-1">
            <ThemeToggle />
            <Button
              variant="ghost"
              size="icon"
              onClick={logout}
              aria-label="Log out"
              title="Log out"
            >
              <LogOut />
            </Button>
          </div>
        </header>

        <main className="mx-auto w-full max-w-6xl px-4 py-8 sm:px-8 md:py-12 lg:px-12">
          <div className="flex flex-col gap-5 border-b border-primary/15 pb-7 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <h1 className="font-heading text-3xl font-semibold tracking-[-0.055em] sm:text-4xl">
                All notes
              </h1>
              <p className="mt-2 text-sm text-muted-foreground">
                {notes.length === 0
                  ? "A quiet space, ready for your next idea."
                  : `${notes.length} note${notes.length === 1 ? "" : "s"} in your workspace.`}
              </p>
            </div>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="lg"
                className="flex-1 sm:flex-none"
                onClick={() => createMutation.mutate("list")}
                disabled={createMutation.isPending}
              >
                <ListChecks />
                New list
              </Button>
              <Button
                size="lg"
                className="flex-1 sm:flex-none"
                onClick={() => createMutation.mutate("markdown")}
                disabled={createMutation.isPending}
              >
                <Plus />
                New note
              </Button>
            </div>
          </div>

          <section className="pt-7">
            {notes.length === 0 ? (
              <EmptyState
                onCreateNote={() => createMutation.mutate("markdown")}
                onCreateList={() => createMutation.mutate("list")}
              />
            ) : (
              <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
                {notes.map((note) => (
                  <NoteCard
                    key={note.id}
                    note={note}
                    onOpen={() => setSelectedNoteId(note.id)}
                    onDelete={() => deleteMutation.mutate(note.id)}
                  />
                ))}
              </div>
            )}
          </section>
        </main>
      </div>
    </div>
  );
}

function EmptyState({
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
      <p className="mt-6 font-heading text-[10px] font-semibold uppercase tracking-[0.16em] text-primary">
        Buffer empty
      </p>
      <h2 className="mt-2 font-heading text-lg font-semibold tracking-[-0.03em]">
        Start with a blank note
      </h2>
      <p className="mt-2 max-w-sm text-sm leading-6 text-muted-foreground">
        Capture an idea in markdown, or make a list you can check off as you go.
      </p>
      <div className="mt-6 flex gap-2">
        <Button variant="outline" onClick={onCreateList}>
          <ListChecks /> New list
        </Button>
        <Button onClick={onCreateNote}>
          <Plus /> New note
        </Button>
      </div>
    </div>
  );
}

function NoteCard({
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
    ? note.content
        .replace(/[#*_`]/g, "")
        .trim()
        .slice(0, 160)
    : `${note.items?.length ?? 0} item${note.items?.length === 1 ? "" : "s"}`;

  return (
    <Card className="terminal-glow group relative min-h-48 cursor-pointer gap-0 rounded-lg border border-primary/12 bg-card/88 py-0 ring-0 transition-all duration-200 hover:-translate-y-0.5 hover:border-primary/45 hover:shadow-[0_0_28px_color-mix(in_oklch,var(--primary)_7%,transparent)]">
      <button
        type="button"
        className="absolute inset-0 z-0 rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-primary/35"
        onClick={onOpen}
        aria-label={`Open ${note.title || "Untitled note"}`}
      />
      <CardContent className="pointer-events-none relative z-[1] flex flex-1 flex-col p-8">
        <CardTitle className="line-clamp-1 font-heading text-[17px] font-semibold flex items-center justify-between">
          <p>{note.title || "Untitled note"}</p>
          <Button
            variant="ghost"
            size="icon-sm"
            className="pointer-events-auto relative z-10 text-muted-foreground opacity-60 hover:bg-destructive/10 hover:text-destructive"
            onClick={(event: React.MouseEvent) => {
              event.stopPropagation();
              onDelete();
            }}
            aria-label={`Delete ${note.title || "Untitled note"}`}
            title="Delete note"
          >
            <Trash2 />
          </Button>
        </CardTitle>
        <p className="mt-2 line-clamp-3 whitespace-pre-wrap text-sm leading-6 text-muted-foreground">
          {preview || "No content yet."}
        </p>
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
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
  }).format(date);
}
