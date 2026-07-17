import { useQuery } from "@tanstack/react-query";
import { Outlet, useNavigate, useParams } from "@tanstack/react-router";
import { getNotes } from "@/lib/api";
import { useAuth } from "@/lib/auth";
import { AuthPage } from "@/components/auth-page";
import { NoteEditor } from "@/components/note-editor";
import { NotesList } from "@/components/notes-list";
import { SyncStatusBar } from "@/components/sync-badge";
import { useSync } from "@/hooks/use-sync";

export function RootPage() {
  const { isAuthenticated } = useAuth();

  if (!isAuthenticated) return <AuthPage />;

  return <AuthenticatedLayout />;
}

function AuthenticatedLayout() {
  const { status } = useSync();
  const hasSyncIssue = status !== "connected";

  return (
    <div
      className={
        hasSyncIssue
          ? "pt-10 [--sync-banner-height:2.5rem]"
          : "[--sync-banner-height:0rem]"
      }
    >
      <SyncStatusBar status={status} />
      <Outlet />
    </div>
  );
}

export function NotesPage() {
  const navigate = useNavigate();

  return (
    <NotesList
      onOpen={(noteId) =>
        void navigate({ to: "/notes/$noteId", params: { noteId } })
      }
    />
  );
}

export function NotePage() {
  const { noteId } = useParams({ from: "/notes/$noteId" });
  const { accountId } = useAuth();
  const navigate = useNavigate();
  const { data: notes = [] } = useQuery({
    queryKey: ["notes", accountId],
    queryFn: getNotes,
  });
  const note = notes.find((candidate) => candidate.id === noteId);

  if (!note) return null;

  return (
    <NoteEditor
      note={note}
      onBack={() => void navigate({ to: "/" })}
      blockNavigation
    />
  );
}
