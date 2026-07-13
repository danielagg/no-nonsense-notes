import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AuthProvider } from '@/lib/auth-context';
import { useAuth } from '@/lib/auth';
import { AuthPage } from '@/components/auth-page';
import { NotesList } from '@/components/notes-list';
import { SyncStatusBar } from '@/components/sync-badge';
import { useSync } from '@/hooks/use-sync';

const queryClient = new QueryClient();

function AppInner() {
  const { isAuthenticated } = useAuth();

  if (!isAuthenticated) {
    return <AuthPage />;
  }

  return <AuthenticatedApp />;
}

function AuthenticatedApp() {
  const { status } = useSync();
  const hasSyncIssue = status !== 'connected';

  return (
    <div
      className={
        hasSyncIssue
          ? 'pt-10 [--sync-banner-height:2.5rem]'
          : '[--sync-banner-height:0rem]'
      }
    >
      <SyncStatusBar status={status} />
      <NotesList />
    </div>
  );
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <AppInner />
      </AuthProvider>
    </QueryClientProvider>
  );
}
