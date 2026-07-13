import { useState, useCallback } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { updateMarkdownNote, updateListNote, type Note } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Checkbox } from "@/components/ui/checkbox";
import {
  ArrowLeft,
  Check,
  FileText,
  ListChecks,
  Plus,
  Save,
  Trash2,
} from "lucide-react";
import { Brand } from "./brand";
import { ThemeToggle } from "./theme-toggle";

interface Props {
  note: Note;
  onBack: () => void;
}

export function NoteEditor({ note, onBack }: Props) {
  const queryClient = useQueryClient();
  const [title, setTitle] = useState(note.title);
  const [content, setContent] = useState(note.content);
  const [items, setItems] = useState(note.items ?? []);

  const saveMutation = useMutation({
    mutationFn: () => {
      const titleOverride = title !== note.title ? title : null;
      if (note.type === "list") {
        return updateListNote(note.id, items, titleOverride);
      }
      return updateMarkdownNote(note.id, content, titleOverride);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["notes"] });
      onBack();
    },
  });

  const addItem = useCallback(() => {
    setItems((prev) => [...prev, ""]);
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
        return item.startsWith("[x] ") ? item.slice(4) : `[x] ${item}`;
      }),
    );
  }, []);

  return (
    <div className="min-h-svh bg-muted/35">
      <header className="sticky top-0 z-20 border-b bg-background/85 backdrop-blur-xl">
        <div className="mx-auto flex h-16 w-full max-w-7xl items-center justify-between px-4 sm:px-8">
          <div className="flex items-center gap-2 sm:gap-5">
            <Brand compact className="hidden sm:flex" />
            <div className="hidden h-5 w-px bg-border sm:block" />
            <Button variant="ghost" onClick={onBack}>
              <ArrowLeft />
              Back to notes
            </Button>
          </div>
          <div className="flex items-center gap-2">
            <ThemeToggle />
            <Button
              className="min-w-24 shadow-sm shadow-primary/20"
              onClick={() => saveMutation.mutate()}
              disabled={saveMutation.isPending}
            >
              {saveMutation.isPending ? (
                <span className="size-3.5 animate-spin rounded-full border-2 border-current border-r-transparent" />
              ) : (
                <Save />
              )}
              {saveMutation.isPending ? "Saving..." : "Save note"}
            </Button>
          </div>
        </div>
      </header>

      <main className="mx-auto w-full max-w-4xl px-4 py-6 sm:px-8 sm:py-10">
        <div className="mb-4 flex items-center justify-between px-1 text-xs text-muted-foreground">
          <div className="flex items-center gap-2">
            {note.type === "markdown" ? (
              <FileText className="size-3.5" />
            ) : (
              <ListChecks className="size-3.5" />
            )}
            <span>
              {note.type === "markdown" ? "Markdown note" : "Checklist"}
            </span>
          </div>
          <span>Edited {new Date(note.updated_at).toLocaleString()}</span>
        </div>

        <section className="min-h-[calc(100svh-10rem)] rounded-2xl border bg-card px-5 py-7 shadow-sm sm:px-10 sm:py-10">
          <Input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Untitled note"
            className="h-auto rounded-none border-0 bg-transparent px-0 py-0 font-heading text-3xl font-semibold tracking-[-0.04em] shadow-none focus-visible:ring-0 dark:bg-transparent sm:text-4xl"
          />
          <div className="my-7 h-px bg-border" />

          {note.type === "markdown" ? (
            <Textarea
              value={content}
              onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) =>
                setContent(e.target.value)
              }
              placeholder="Start writing..."
              className="min-h-[60vh] resize-none rounded-none border-0 bg-transparent px-0 py-0 font-mono text-[15px] leading-7 shadow-none focus-visible:ring-0 dark:bg-transparent"
            />
          ) : (
            <div className="space-y-3">
              {items.map((item, i) => {
                const isChecked = item.startsWith("[x] ");
                const text = isChecked ? item.slice(4) : item;
                return (
                  <div
                    key={i}
                    className="group flex items-center gap-3 rounded-xl border bg-background/50 p-2 transition-colors focus-within:border-primary/30"
                  >
                    <Checkbox
                      checked={isChecked}
                      onCheckedChange={() => toggleItem(i)}
                      className="ml-1 size-5 rounded-md"
                    />
                    <Input
                      value={text}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                        const prefix = isChecked ? "[x] " : "";
                        updateItem(i, prefix + e.target.value);
                      }}
                      placeholder="List item"
                      className={`h-9 flex-1 border-0 bg-transparent shadow-none focus-visible:ring-0 dark:bg-transparent ${isChecked ? "text-muted-foreground line-through" : ""}`}
                    />
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      className="text-muted-foreground opacity-60 hover:bg-destructive/10 hover:text-destructive sm:opacity-0 sm:group-hover:opacity-100"
                      onClick={() => removeItem(i)}
                      aria-label="Remove item"
                    >
                      <Trash2 />
                    </Button>
                  </div>
                );
              })}
              {items.length === 0 && (
                <div className="rounded-xl border border-dashed py-10 text-center text-sm text-muted-foreground">
                  <Check className="mx-auto mb-3 size-5" />
                  No items yet.
                </div>
              )}
              <Button variant="outline" className="mt-2" onClick={addItem}>
                <Plus /> Add item
              </Button>
            </div>
          )}
        </section>
      </main>
    </div>
  );
}
