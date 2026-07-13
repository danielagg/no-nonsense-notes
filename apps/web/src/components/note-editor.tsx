import { useState, useCallback, useEffect, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";
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
  RefreshCw,
  Trash2,
} from "lucide-react";
import { Brand } from "./brand";
import { ThemeToggle } from "./theme-toggle";

interface Props {
  note: Note;
  onBack: () => void;
}

interface NoteDraft {
  title: string;
  content: string;
  items: string[];
}

type SaveStatus = "saved" | "saving" | "error";

const AUTOSAVE_DELAY_MS = 650;

export function NoteEditor({ note, onBack }: Props) {
  const queryClient = useQueryClient();
  const [title, setTitle] = useState(note.title);
  const [content, setContent] = useState(note.content);
  const [items, setItems] = useState(note.items ?? []);
  const titleRef = useRef(note.title);
  const contentRef = useRef(note.content);
  const itemsRef = useRef(note.items ?? []);
  const initialDraftSignature = draftSignature({
    title: note.title,
    content: note.content,
    items: note.items ?? [],
  });
  const latestSignatureRef = useRef(initialDraftSignature);
  const lastRequestedSignatureRef = useRef(initialDraftSignature);
  const lastSavedSignatureRef = useRef(initialDraftSignature);
  const saveQueueRef = useRef<Promise<void>>(Promise.resolve());
  const saveTimerRef = useRef<number | null>(null);
  const mountedRef = useRef(true);
  const [saveStatus, setSaveStatus] = useState<SaveStatus>("saved");
  const nextItemId = useRef(note.items?.length ?? 0);
  const [itemIds, setItemIds] = useState(() =>
    (note.items ?? []).map((_, index) => `list-item-${index}`),
  );
  const [draggingIndex, setDraggingIndex] = useState<number | null>(null);
  const draggingIndexRef = useRef<number | null>(null);
  const dragPointerIdRef = useRef<number | null>(null);
  const dragHandleRef = useRef<HTMLButtonElement | null>(null);
  const dragTimerRef = useRef<number | null>(null);
  const dragDidReorderRef = useRef(false);
  const itemRowsRef = useRef<Array<HTMLDivElement | null>>([]);

  const getCurrentDraft = useCallback(
    (): NoteDraft => ({
      title: titleRef.current,
      content: contentRef.current,
      items: itemsRef.current,
    }),
    [],
  );

  const persistDraft = useCallback(
    (draft: NoteDraft): Promise<void> => {
      const signature = draftSignature(draft);
      latestSignatureRef.current = signature;
      lastRequestedSignatureRef.current = signature;
      if (mountedRef.current) setSaveStatus("saving");

      const operation = saveQueueRef.current
        .catch(() => undefined)
        .then(async () => {
          const titleOverride = draft.title !== note.title ? draft.title : null;
          if (note.type === "list") {
            await updateListNote(note.id, draft.items, titleOverride);
          } else {
            await updateMarkdownNote(note.id, draft.content, titleOverride);
          }
          lastSavedSignatureRef.current = signature;
          await queryClient.invalidateQueries({ queryKey: ["notes"] });
        });

      saveQueueRef.current = operation;
      void operation.then(
        () => {
          if (
            mountedRef.current &&
            latestSignatureRef.current === signature &&
            lastRequestedSignatureRef.current === signature
          ) {
            setSaveStatus("saved");
          }
        },
        () => {
          if (
            latestSignatureRef.current === signature &&
            lastRequestedSignatureRef.current === signature
          ) {
            lastRequestedSignatureRef.current = lastSavedSignatureRef.current;
            if (mountedRef.current) setSaveStatus("error");
          }
        },
      );
      return operation;
    },
    [note.id, note.title, note.type, queryClient],
  );

  const clearSaveTimer = useCallback(() => {
    if (saveTimerRef.current !== null) {
      window.clearTimeout(saveTimerRef.current);
      saveTimerRef.current = null;
    }
  }, []);

  const flushSave = useCallback((): Promise<void> => {
    clearSaveTimer();
    const draft = getCurrentDraft();
    const signature = draftSignature(draft);
    if (signature === lastRequestedSignatureRef.current) return saveQueueRef.current;
    return persistDraft(draft);
  }, [clearSaveTimer, getCurrentDraft, persistDraft]);

  useEffect(() => {
    const draft = getCurrentDraft();
    const signature = draftSignature(draft);
    latestSignatureRef.current = signature;
    if (signature === lastRequestedSignatureRef.current) return;

    if (mountedRef.current) setSaveStatus("saving");
    clearSaveTimer();
    saveTimerRef.current = window.setTimeout(() => {
      saveTimerRef.current = null;
      void persistDraft(draft).catch(() => undefined);
    }, AUTOSAVE_DELAY_MS);
    return clearSaveTimer;
  }, [title, content, items, clearSaveTimer, getCurrentDraft, persistDraft]);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      clearSaveTimer();
    };
  }, [clearSaveTimer]);

  const saveItemsImmediately = useCallback(
    (nextItems: string[]) => {
      itemsRef.current = nextItems;
      setItems(nextItems);
      void persistDraft({ ...getCurrentDraft(), items: nextItems }).catch(() => undefined);
    },
    [getCurrentDraft, persistDraft],
  );

  const addItem = useCallback(() => {
    saveItemsImmediately([...itemsRef.current, ""]);
    setItemIds((prev) => [...prev, `list-item-${nextItemId.current++}`]);
  }, [saveItemsImmediately]);

  const updateItem = useCallback((index: number, value: string) => {
    const nextItems = itemsRef.current.map((item, i) => (i === index ? value : item));
    itemsRef.current = nextItems;
    setItems(nextItems);
  }, []);

  const removeItem = useCallback(
    (index: number) => {
      saveItemsImmediately(itemsRef.current.filter((_, i) => i !== index));
      setItemIds((prev) => prev.filter((_, i) => i !== index));
    },
    [saveItemsImmediately],
  );

  const toggleItem = useCallback(
    (index: number) => {
      saveItemsImmediately(
        itemsRef.current.map((item, i) => {
          if (i !== index) return item;
          return item.startsWith("[x] ") ? item.slice(4) : `[x] ${item}`;
        }),
      );
    },
    [saveItemsImmediately],
  );

  const reorderItem = useCallback(
    (fromIndex: number, toIndex: number, saveImmediately = false) => {
      if (fromIndex === toIndex) return;
      const nextItems = moveArrayItem(itemsRef.current, fromIndex, toIndex);
      itemsRef.current = nextItems;
      setItems(nextItems);
      setItemIds((prev) => moveArrayItem(prev, fromIndex, toIndex));
      if (saveImmediately) {
        void persistDraft({ ...getCurrentDraft(), items: nextItems }).catch(() => undefined);
      }
    },
    [getCurrentDraft, persistDraft],
  );

  const clearDragTimer = useCallback(() => {
    if (dragTimerRef.current !== null) {
      window.clearTimeout(dragTimerRef.current);
      dragTimerRef.current = null;
    }
  }, []);

  const finishDrag = useCallback(() => {
    clearDragTimer();
    const shouldSave = dragDidReorderRef.current;
    dragDidReorderRef.current = false;
    const handle = dragHandleRef.current;
    const pointerId = dragPointerIdRef.current;
    dragHandleRef.current = null;
    dragPointerIdRef.current = null;
    draggingIndexRef.current = null;
    setDraggingIndex(null);
    if (handle && pointerId !== null && handle.hasPointerCapture(pointerId)) {
      handle.releasePointerCapture(pointerId);
    }
    if (shouldSave) {
      void persistDraft(getCurrentDraft()).catch(() => undefined);
    }
  }, [clearDragTimer, getCurrentDraft, persistDraft]);

  useEffect(() => finishDrag, [finishDrag]);

  const handleDragPointerDown = useCallback(
    (event: React.PointerEvent<HTMLButtonElement>, index: number) => {
      if (!event.isPrimary || (event.pointerType === "mouse" && event.button !== 0)) return;

      clearDragTimer();
      dragDidReorderRef.current = false;
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
        dragDidReorderRef.current = true;
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
      reorderItem(index, targetIndex, true);
    },
    [items.length, reorderItem],
  );

  const handleBack = useCallback(async () => {
    try {
      await flushSave();
      onBack();
    } catch {
      // Keep the editor open so the visible retry action can recover the save.
    }
  }, [flushSave, onBack]);

  return (
    <div className="min-h-[calc(100svh-var(--sync-banner-height))] bg-muted/35">
      <header className="sticky top-[var(--sync-banner-height)] z-20 border-b bg-background/85 backdrop-blur-xl">
        <div className="mx-auto flex h-16 w-full max-w-7xl items-center justify-between px-4 sm:px-8">
          <div className="flex items-center gap-2 sm:gap-5">
            <Brand compact className="hidden sm:flex" />
            <div className="hidden h-5 w-px bg-border sm:block" />
            <Button variant="ghost" onClick={() => void handleBack()}>
              <ArrowLeft />
              Back to notes
            </Button>
          </div>
          <div className="flex items-center gap-2">
            <ThemeToggle />
            <Button
              className="min-w-24 shadow-sm shadow-primary/20"
              variant={saveStatus === "error" ? "destructive" : "secondary"}
              onClick={() => void flushSave().catch(() => undefined)}
              disabled={saveStatus === "saving"}
            >
              {saveStatus === "saving" ? (
                <span className="size-3.5 animate-spin rounded-full border-2 border-current border-r-transparent" />
              ) : saveStatus === "error" ? (
                <RefreshCw />
              ) : (
                <Check />
              )}
              {saveStatus === "saving"
                ? "Saving..."
                : saveStatus === "error"
                  ? "Retry save"
                  : "Saved"}
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
            onChange={(e) => {
              titleRef.current = e.target.value;
              setTitle(e.target.value);
            }}
            placeholder="Untitled note"
            className="h-auto rounded-none border-0 bg-transparent px-0 py-0 font-heading text-[24px] font-semibold tracking-[-0.04em] shadow-none focus-visible:ring-0 dark:bg-transparent md:text-[32px]"
          />
          <div className="my-7 h-px bg-border" />

          {note.type === "markdown" ? (
            <Textarea
              value={content}
              onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => {
                contentRef.current = e.target.value;
                setContent(e.target.value);
              }}
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

function draftSignature(draft: NoteDraft): string {
  return JSON.stringify([draft.title, draft.content, draft.items]);
}
