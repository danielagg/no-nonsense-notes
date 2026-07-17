import type { CSSProperties } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ListChecks, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { createNote, deleteNote, getNotes } from "@/lib/api";
import { useAuth } from "@/lib/auth";
import { EmptyState } from "./notes-list/empty-state";
import { MobileHeader } from "./notes-list/mobile-header";
import { NoteCard } from "./notes-list/note-card";
import { NotesSidebar } from "./notes-list/notes-sidebar";
import { useStoredSidebarWidth } from "./notes-list/use-sidebar-width";

interface Props {
  onOpen: (noteId: string) => void;
}

export function NotesList({ onOpen }: Props) {
  const { logout, accountId } = useAuth();
  const queryClient = useQueryClient();
  const [sidebarWidth, setSidebarWidth] = useStoredSidebarWidth();
  const { data: notes = [] } = useQuery({
    queryKey: ["notes", accountId],
    queryFn: getNotes,
  });
  const createMutation = useMutation({
    mutationFn: (type: "markdown" | "list") => Promise.resolve(createNote(type)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["notes"] }),
  });
  const deleteMutation = useMutation({
    mutationFn: deleteNote,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["notes"] }),
  });

  return (
    <div
      className="terminal-grid min-h-[calc(100svh-var(--sync-banner-height))] md:grid md:grid-cols-[var(--sidebar-width)_minmax(0,1fr)]"
      style={{ "--sidebar-width": `${sidebarWidth}px` } as CSSProperties}
    >
      <NotesSidebar
        noteCount={notes.length}
        width={sidebarWidth}
        setWidth={setSidebarWidth}
        onLogout={logout}
      />

      <div className="min-w-0">
        <MobileHeader onLogout={logout} />

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
                    onOpen={() => onOpen(note.id)}
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
