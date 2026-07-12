import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AuthProvider, useAuth } from '@/lib/auth-context';
import { AuthPage } from '@/components/auth-page';
import { NotesList } from '@/components/notes-list';
import { SyncBadge } from '@/components/sync-badge';

const queryClient = new QueryClient();

function AppInner() {
  const { isAuthenticated } = useAuth();

  if (!isAuthenticated) {
    return <AuthPage />;
  }

  return (
    <>
      <SyncBadge />
      <NotesList />
    </>
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
