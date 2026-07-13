import { useState, useCallback } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { updateMarkdownNote, updateListNote, type Note } from '@/lib/api';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Checkbox } from '@/components/ui/checkbox';

interface Props {
  note: Note;
  onBack: () => void;
}

export function NoteEditor({ note, onBack }: Props) {
  const queryClient = useQueryClient();
  const [content, setContent] = useState(note.content);
  const [items, setItems] = useState(note.items ?? []);

  const saveMutation = useMutation({
    mutationFn: () => {
      if (note.type === 'list') {
        return updateListNote(note.id, items);
      }
      return updateMarkdownNote(note.id, content);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['notes'] });
      onBack();
    },
  });

  const addItem = useCallback(() => {
    setItems((prev) => [...prev, '']);
  }, []);

  const updateItem = useCallback((index: number, value: string) => {
    setItems((prev) => prev.map((item, i) => (i === index ? value : item)));
  }, []);

  const removeItem = useCallback((index: number) => {
    setItems((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const toggleItem = useCallback((index: number) => {
    setItems((prev) =>
      prev.map((item, i) => {
        if (i !== index) return item;
        return item.startsWith('[x] ') ? item.slice(4) : `[x] ${item}`;
      }),
    );
  }, []);

  return (
    <div className="min-h-screen p-4 max-w-2xl mx-auto">
      <div className="flex items-center justify-between mb-4">
        <Button variant="ghost" size="sm" onClick={onBack}>
          &larr; Back
        </Button>
        <Button size="sm" onClick={() => saveMutation.mutate()} disabled={saveMutation.isPending}>
          {saveMutation.isPending ? 'Saving...' : 'Save'}
        </Button>
      </div>

      <h2 className="text-lg font-semibold mb-4 text-muted-foreground">{note.title}</h2>

      {note.type === 'markdown' ? (
        <Textarea
          value={content}
          onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setContent(e.target.value)}
          placeholder="Write markdown..."
          className="min-h-[60vh] font-mono text-sm resize-y"
        />
      ) : (
        <div className="space-y-2">
          {items.map((item, i) => {
            const isChecked = item.startsWith('[x] ');
            const text = isChecked ? item.slice(4) : item;
            return (
              <div key={i} className="flex items-center gap-2">
                <Checkbox
                  checked={isChecked}
                  onCheckedChange={() => toggleItem(i)}
                />
                <Input
                  value={text}
                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                    const prefix = isChecked ? '[x] ' : '';
                    updateItem(i, prefix + e.target.value);
                  }}
                  className="flex-1"
                />
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-8 w-8 p-0 text-destructive"
                  onClick={() => removeItem(i)}
                >
                  &times;
                </Button>
              </div>
            );
          })}
          <Button variant="outline" size="sm" onClick={addItem}>
            + Add item
          </Button>
        </div>
      )}

      <p className="text-xs text-muted-foreground mt-4">
        Last updated: {new Date(note.updated_at).toLocaleString()}
      </p>
    </div>
  );
}
