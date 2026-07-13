import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getNotes, createNote, deleteNote, type Note } from '@/lib/api';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useAuth } from '@/lib/auth-context';
import { NoteEditor } from './note-editor';

export function NotesList() {
  const { logout, accountId } = useAuth();
  const queryClient = useQueryClient();
  const [selectedNote, setSelectedNote] = useState<Note | null>(null);

  const { data: notes = [] } = useQuery({
    queryKey: ['notes'],
    queryFn: getNotes,
  });

  const createMutation = useMutation({
    mutationFn: (type: 'markdown' | 'list') => Promise.resolve(createNote(type)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['notes'] }),
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => { await deleteNote(id); },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['notes'] });
      setSelectedNote(null);
    },
  });

  if (selectedNote) {
    return <NoteEditor note={selectedNote} onBack={() => setSelectedNote(null)} />;
  }

  return (
    <div className="min-h-screen p-4 max-w-2xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold">Notes</h1>
          <p className="text-sm text-muted-foreground truncate max-w-[200px]">
            {accountId}
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => createMutation.mutate('markdown')}>
            + Markdown
          </Button>
          <Button variant="outline" size="sm" onClick={() => createMutation.mutate('list')}>
            + List
          </Button>
          <Button variant="ghost" size="sm" onClick={logout}>
            Logout
          </Button>
        </div>
      </div>

      {notes.length === 0 ? (
        <div className="text-center text-muted-foreground py-16">
          <p>No notes yet. Create one above.</p>
        </div>
      ) : (
        <div className="space-y-2">
          {notes.map((note) => (
            <Card
              key={note.id}
              className="cursor-pointer hover:bg-accent/50 transition-colors"
              onClick={() => setSelectedNote(note)}
            >
              <CardHeader className="py-3 px-4">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-base">{note.title}</CardTitle>
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-muted-foreground px-1.5 py-0.5 rounded bg-muted">
                      {note.type}
                    </span>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-6 text-xs text-destructive"
                      onClick={(e: React.MouseEvent) => {
                        e.stopPropagation();
                        deleteMutation.mutate(note.id);
                      }}
                    >
                      Delete
                    </Button>
                  </div>
                </div>
              </CardHeader>
              {note.type === 'markdown' && (
                <CardContent className="pt-0 px-4 pb-3">
                  <p className="text-sm text-muted-foreground line-clamp-2 whitespace-pre-wrap">
                    {note.content.replace(/[#*_`]/g, '').slice(0, 200)}
                  </p>
                </CardContent>
              )}
              {note.type === 'list' && note.items && (
                <CardContent className="pt-0 px-4 pb-3">
                  <p className="text-sm text-muted-foreground">
                    {note.items.length} item{note.items.length !== 1 ? 's' : ''}
                  </p>
                </CardContent>
              )}
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
