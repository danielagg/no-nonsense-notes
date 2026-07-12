import { QueryClient, QueryClientProvider, useQueryClient } from '@tanstack/react-query';
import { AuthProvider, useAuth } from '@/lib/auth-context';
import { SyncProvider } from '@/lib/sync-context';
import { AuthPage } from '@/components/auth-page';
import { NotesList } from '@/components/notes-list';
import { SyncBadge } from '@/components/sync-badge';

const queryClient = new QueryClient();

function AppInner() {
  const { isAuthenticated } = useAuth();
  const queryClient_ = useQueryClient();

  if (!isAuthenticated) {
    return <AuthPage />;
  }

  return (
    <SyncProvider onSyncComplete={() => queryClient_.invalidateQueries({ queryKey: ['notes'] })}>
      <SyncBadge />
      <NotesList />
    </SyncProvider>
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
