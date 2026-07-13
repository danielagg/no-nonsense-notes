import { useState, useCallback, useEffect, useRef } from "react";
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
  GripVertical,
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
  const nextItemId = useRef(note.items?.length ?? 0);
  const [itemIds, setItemIds] = useState(() =>
    (note.items ?? []).map((_, index) => `list-item-${index}`),
  );
  const [draggingIndex, setDraggingIndex] = useState<number | null>(null);
  const draggingIndexRef = useRef<number | null>(null);
  const dragPointerIdRef = useRef<number | null>(null);
  const dragHandleRef = useRef<HTMLButtonElement | null>(null);
  const dragTimerRef = useRef<number | null>(null);
  const itemRowsRef = useRef<Array<HTMLDivElement | null>>([]);

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
    setItemIds((prev) => [...prev, `list-item-${nextItemId.current++}`]);
  }, []);

  const updateItem = useCallback((index: number, value: string) => {
    setItems((prev) => prev.map((item, i) => (i === index ? value : item)));
  }, []);

  const removeItem = useCallback((index: number) => {
    setItems((prev) => prev.filter((_, i) => i !== index));
    setItemIds((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const toggleItem = useCallback((index: number) => {
    setItems((prev) =>
      prev.map((item, i) => {
        if (i !== index) return item;
        return item.startsWith("[x] ") ? item.slice(4) : `[x] ${item}`;
      }),
    );
  }, []);

  const reorderItem = useCallback((fromIndex: number, toIndex: number) => {
    if (fromIndex === toIndex) return;
    setItems((prev) => moveArrayItem(prev, fromIndex, toIndex));
    setItemIds((prev) => moveArrayItem(prev, fromIndex, toIndex));
  }, []);

  const clearDragTimer = useCallback(() => {
    if (dragTimerRef.current !== null) {
      window.clearTimeout(dragTimerRef.current);
      dragTimerRef.current = null;
    }
  }, []);

  const finishDrag = useCallback(() => {
    clearDragTimer();
    const handle = dragHandleRef.current;
    const pointerId = dragPointerIdRef.current;
    dragHandleRef.current = null;
    dragPointerIdRef.current = null;
    draggingIndexRef.current = null;
    setDraggingIndex(null);
    if (handle && pointerId !== null && handle.hasPointerCapture(pointerId)) {
      handle.releasePointerCapture(pointerId);
    }
  }, [clearDragTimer]);

  useEffect(() => finishDrag, [finishDrag]);

  const handleDragPointerDown = useCallback(
    (event: React.PointerEvent<HTMLButtonElement>, index: number) => {
      if (!event.isPrimary || (event.pointerType === "mouse" && event.button !== 0)) return;

      clearDragTimer();
      dragPointerIdRef.current = event.pointerId;
      dragHandleRef.current = event.currentTarget;
      event.currentTarget.setPointerCapture(event.pointerId);

      const activate = () => {
        draggingIndexRef.current = index;
        setDraggingIndex(index);
        dragTimerRef.current = null;
      };

      if (event.pointerType === "touch") {
        dragTimerRef.current = window.setTimeout(activate, 180);
      } else {
        activate();
      }
    },
    [clearDragTimer],
  );

  const handleDragPointerMove = useCallback(
    (event: React.PointerEvent<HTMLButtonElement>) => {
      if (event.pointerId !== dragPointerIdRef.current) return;
      const fromIndex = draggingIndexRef.current;
      if (fromIndex === null) return;

      event.preventDefault();
      const edgeSize = 72;
      if (event.clientY < edgeSize) window.scrollBy({ top: -12 });
      if (event.clientY > window.innerHeight - edgeSize) window.scrollBy({ top: 12 });

      let closestIndex = fromIndex;
      let closestDistance = Number.POSITIVE_INFINITY;
      itemRowsRef.current.slice(0, items.length).forEach((row, index) => {
        if (!row) return;
        const rect = row.getBoundingClientRect();
        const distance = Math.abs(event.clientY - (rect.top + rect.height / 2));
        if (distance < closestDistance) {
          closestDistance = distance;
          closestIndex = index;
        }
      });

      if (closestIndex !== fromIndex) {
        reorderItem(fromIndex, closestIndex);
        draggingIndexRef.current = closestIndex;
        setDraggingIndex(closestIndex);
      }
    },
    [items.length, reorderItem],
  );

  const handleReorderKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLButtonElement>, index: number) => {
      const direction = event.key === "ArrowUp" ? -1 : event.key === "ArrowDown" ? 1 : 0;
      if (direction === 0) return;
      const targetIndex = index + direction;
      if (targetIndex < 0 || targetIndex >= items.length) return;
      event.preventDefault();
      reorderItem(index, targetIndex);
    },
    [items.length, reorderItem],
  );

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
            className="h-auto rounded-none border-0 bg-transparent px-0 py-0 font-heading text-[24px] font-semibold tracking-[-0.04em] shadow-none focus-visible:ring-0 dark:bg-transparent md:text-[32px]"
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
                    key={itemIds[i]}
                    ref={(row) => {
                      itemRowsRef.current[i] = row;
                    }}
                    className={`group flex items-center gap-2 rounded-xl border bg-background/50 p-2 transition-[transform,box-shadow,border-color,background-color] duration-150 focus-within:border-primary/30 ${draggingIndex === i ? "relative z-10 scale-[1.015] border-primary/35 bg-card shadow-lg shadow-foreground/10" : ""}`}
                  >
                    <button
                      type="button"
                      className="grid size-8 shrink-0 touch-none cursor-grab select-none place-items-center rounded-lg text-muted-foreground/55 outline-none transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-3 focus-visible:ring-ring/30 active:cursor-grabbing"
                      onPointerDown={(event) => handleDragPointerDown(event, i)}
                      onPointerMove={handleDragPointerMove}
                      onPointerUp={finishDrag}
                      onPointerCancel={finishDrag}
                      onLostPointerCapture={finishDrag}
                      onKeyDown={(event) => handleReorderKeyDown(event, i)}
                      aria-label={`Reorder item ${i + 1}`}
                      aria-keyshortcuts="ArrowUp ArrowDown"
                      title="Hold and drag to reorder"
                    >
                      <GripVertical className="size-4" />
                    </button>
                    <Checkbox
                      checked={isChecked}
                      onCheckedChange={() => toggleItem(i)}
                      className="size-5 rounded-md"
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

function moveArrayItem<T>(items: T[], fromIndex: number, toIndex: number): T[] {
  const next = [...items];
  const [movedItem] = next.splice(fromIndex, 1);
  next.splice(toIndex, 0, movedItem);
  return next;
}
