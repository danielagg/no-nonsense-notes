import { useCallback, useRef, useState } from "react";
import { Input } from "@/components/ui/input";
import { ChecklistEditor } from "./note-editor/checklist-editor";
import { EditorHeader } from "./note-editor/editor-header";
import { EditorMeta } from "./note-editor/editor-meta";
import { MarkdownEditor } from "./note-editor/markdown-editor";
import { NavigationBlocker } from "./note-editor/navigation-blocker";
import type { MarkdownMode } from "./note-editor/model";
import { useChecklistController } from "./note-editor/use-checklist-controller";
import { useNoteDraft } from "./note-editor/use-note-draft";
import type { Note } from "@/lib/api";

interface Props {
  note: Note;
  onBack: () => void;
  blockNavigation?: boolean;
}

export function NoteEditor({ note, onBack, blockNavigation = false }: Props) {
  const [markdownMode, setMarkdownMode] = useState<MarkdownMode>("edit");
  const editorRef = useRef<HTMLDivElement | null>(null);
  const allowNavigationRef = useRef(false);
  const draft = useNoteDraft(note, editorRef);
  const checklist = useChecklistController({
    items: draft.items,
    itemIds: draft.itemIds,
    setItemIds: draft.setItemIds,
    createItemId: draft.createItemId,
    getItems: draft.getItems,
    updateItems: draft.updateItems,
    getCurrentDraft: draft.getCurrentDraft,
    persistDraft: draft.persistDraft,
    saveItemsImmediately: draft.saveItemsImmediately,
  });
  const { flushSave, applyPendingRemoteNote } = draft;

  const handleBack = useCallback(async () => {
    try {
      await flushSave();
      allowNavigationRef.current = true;
      onBack();
    } catch {
      // Keep the editor open so the visible retry action can recover the save.
    }
  }, [flushSave, onBack]);

  const handleEditorBlur = useCallback(() => {
    window.setTimeout(applyPendingRemoteNote, 0);
  }, [applyPendingRemoteNote]);

  return (
    <div
      ref={editorRef}
      onBlurCapture={handleEditorBlur}
      className="terminal-grid min-h-[calc(100svh-var(--sync-banner-height))]"
    >
      {blockNavigation && (
        <NavigationBlocker
          onBack={handleBack}
          allowNavigationRef={allowNavigationRef}
        />
      )}
      <EditorHeader
        saveStatus={draft.saveStatus}
        onBack={() => void handleBack()}
        onRetry={() => void draft.flushSave().catch(() => undefined)}
      />

      <main className="mx-auto w-full max-w-4xl px-4 py-6 sm:px-8 sm:py-10">
        <EditorMeta
          note={note}
          markdownMode={markdownMode}
          onMarkdownModeChange={setMarkdownMode}
        />

        <section className="terminal-glow min-h-[calc(100svh-10rem)] rounded-lg border border-primary/15 bg-card/90 px-5 py-7 sm:px-10 sm:py-10">
          <Input
            value={draft.title}
            onChange={(event) => draft.updateTitle(event.target.value)}
            placeholder="Untitled note"
            className="h-auto rounded-none border-0 bg-transparent px-0 py-0 font-heading text-[24px] font-semibold tracking-[-0.04em] shadow-none focus-visible:ring-0 dark:bg-transparent md:text-[32px]"
          />
          <div className="my-7 h-px bg-primary/15" />

          {note.type === "markdown" ? (
            <MarkdownEditor
              content={draft.content}
              mode={markdownMode}
              onChange={draft.updateContent}
            />
          ) : (
            <ChecklistEditor
              items={draft.items}
              itemIds={draft.itemIds}
              draggingIndex={checklist.draggingIndex}
              itemRowsRef={checklist.itemRowsRef}
              itemInputRefs={checklist.itemInputRefs}
              onAdd={checklist.addItem}
              onInsertAfter={checklist.insertItemAfter}
              onUpdate={checklist.updateItem}
              onRemove={checklist.removeItem}
              onToggle={checklist.toggleItem}
              onDragStart={checklist.handleDragPointerDown}
              onReorderKeyDown={checklist.handleReorderKeyDown}
            />
          )}
        </section>
      </main>
    </div>
  );
}
