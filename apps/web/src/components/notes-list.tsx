import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { getNotes, createNote, deleteNote, type Note } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useAuth } from "@/lib/auth-context";
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
} from "lucide-react";

export function NotesList() {
  const { logout, accountId } = useAuth();
  const queryClient = useQueryClient();
  const [selectedNote, setSelectedNote] = useState<Note | null>(null);

  const { data: notes = [] } = useQuery({
    queryKey: ["notes", accountId],
    queryFn: getNotes,
  });

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
      setSelectedNote(null);
    },
  });

  if (selectedNote) {
    return (
      <NoteEditor note={selectedNote} onBack={() => setSelectedNote(null)} />
    );
  }

  return (
    <div className="min-h-svh bg-background md:grid md:grid-cols-[240px_1fr]">
      <aside className="hidden h-svh flex-col border-r bg-sidebar px-3 py-4 md:sticky md:top-0 md:flex">
        <Brand className="px-2" />

        <nav className="mt-8">
          <div className="flex h-10 items-center gap-3 rounded-xl bg-sidebar-accent px-3 text-sm font-medium text-sidebar-accent-foreground">
            <NotebookPen className="size-4 text-primary" />
            All notes
            <span className="ml-auto rounded-md bg-background/70 px-1.5 py-0.5 text-[11px] tabular-nums text-muted-foreground">
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
          <div className="mt-3 border-t px-2 pt-4">
            <p className="text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">
              Workspace
            </p>
            <p
              className="mt-1 truncate text-xs text-sidebar-foreground/70"
              title={accountId ?? undefined}
            >
              {accountId}
            </p>
          </div>
        </div>
      </aside>

      <div className="min-w-0">
        <header className="flex h-16 items-center justify-between border-b bg-background/80 px-4 backdrop-blur md:hidden">
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
          <div className="flex flex-col gap-5 border-b pb-7 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <p className="mb-2 text-xs font-medium uppercase tracking-[0.14em] text-primary">
                Your workspace
              </p>
              <h1 className="font-heading text-3xl font-semibold tracking-[-0.04em] sm:text-4xl">
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
                className="flex-1 shadow-sm shadow-primary/20 sm:flex-none"
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
                    onOpen={() => setSelectedNote(note)}
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
    <div className="flex min-h-[360px] flex-col items-center justify-center rounded-2xl border border-dashed bg-card/45 px-6 text-center">
      <span className="grid size-14 place-items-center rounded-2xl bg-primary/10 text-primary ring-8 ring-primary/[0.04]">
        <NotebookPen className="size-6" />
      </span>
      <h2 className="mt-6 font-heading text-lg font-semibold tracking-[-0.02em]">
        Start with a blank page
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
    <Card className="group relative min-h-48 cursor-pointer gap-0 rounded-2xl border bg-card py-0 shadow-sm ring-0 transition-all duration-200 hover:-translate-y-0.5 hover:border-primary/20 hover:shadow-lg hover:shadow-foreground/[0.05]">
      <button
        type="button"
        className="absolute inset-0 z-0 rounded-2xl outline-none focus-visible:ring-3 focus-visible:ring-primary/30"
        onClick={onOpen}
        aria-label={`Open ${note.title || "Untitled note"}`}
      />
      <CardHeader className="pointer-events-none relative z-[1] flex-row items-start justify-between px-5 pb-0 pt-5">
        <span className="grid size-9 place-items-center rounded-xl bg-muted text-muted-foreground transition-colors group-hover:bg-primary/10 group-hover:text-primary">
          {isMarkdown ? (
            <FileText className="size-4" />
          ) : (
            <ListChecks className="size-4" />
          )}
        </span>
        <Button
          variant="ghost"
          size="icon-sm"
          className="pointer-events-auto relative z-10 text-muted-foreground opacity-60 hover:bg-destructive/10 hover:text-destructive sm:opacity-0 sm:group-hover:opacity-100"
          onClick={(event: React.MouseEvent) => {
            event.stopPropagation();
            onDelete();
          }}
          aria-label={`Delete ${note.title || "Untitled note"}`}
          title="Delete note"
        >
          <MoreHorizontal />
        </Button>
      </CardHeader>
      <CardContent className="pointer-events-none relative z-[1] flex flex-1 flex-col px-5 pb-5 pt-4">
        <CardTitle className="line-clamp-1 text-[15px] font-semibold">
          {note.title || "Untitled note"}
        </CardTitle>
        <p className="mt-2 line-clamp-3 whitespace-pre-wrap text-sm leading-6 text-muted-foreground">
          {preview || "No content yet."}
        </p>
        <div className="mt-auto flex items-center justify-between pt-5 text-[11px] text-muted-foreground">
          <span className="capitalize">{isMarkdown ? "Note" : "List"}</span>
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
