import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type RefObject,
} from "react";
import { useQueryClient } from "@tanstack/react-query";
import { updateListNote, updateMarkdownNote, type Note } from "@/lib/api";
import {
  draftSignature,
  noteToDraft,
  type NoteDraft,
  type SaveStatus,
} from "./model";

const AUTOSAVE_DELAY_MS = 650;
const AUTOSAVE_MAX_WAIT_MS = 1_200;

export function useNoteDraft(
  note: Note,
  editorRef: RefObject<HTMLDivElement | null>,
) {
  const queryClient = useQueryClient();
  const [title, setTitle] = useState(note.title);
  const [content, setContent] = useState(note.content);
  const [items, setItems] = useState(note.items ?? []);
  const titleRef = useRef(note.title);
  const contentRef = useRef(note.content);
  const itemsRef = useRef(note.items ?? []);
  const nextItemId = useRef(note.items?.length ?? 0);
  const [itemIds, setItemIds] = useState(() =>
    (note.items ?? []).map((_, index) => `list-item-${index}`),
  );

  const initialDraftSignature = draftSignature(noteToDraft(note));
  const latestSignatureRef = useRef(initialDraftSignature);
  const lastRequestedSignatureRef = useRef(initialDraftSignature);
  const lastSavedSignatureRef = useRef(initialDraftSignature);
  const saveQueueRef = useRef<Promise<void>>(Promise.resolve());
  const saveTimerRef = useRef<number | null>(null);
  const saveMaxWaitTimerRef = useRef<number | null>(null);
  const mountedRef = useRef(true);
  const hasPendingLocalChangesRef = useRef(false);
  const pendingRemoteNoteRef = useRef<Note | null>(null);
  const [saveStatus, setSaveStatus] = useState<SaveStatus>("saved");

  const createItemId = useCallback(
    () => `list-item-${nextItemId.current++}`,
    [],
  );

  const getItems = useCallback(() => itemsRef.current, []);

  const updateTitle = useCallback((value: string) => {
    titleRef.current = value;
    setTitle(value);
  }, []);

  const updateContent = useCallback((value: string) => {
    contentRef.current = value;
    setContent(value);
  }, []);

  const updateItems = useCallback((value: string[]) => {
    itemsRef.current = value;
    setItems(value);
  }, []);

  const getCurrentDraft = useCallback(
    (): NoteDraft => ({
      title: titleRef.current,
      content: contentRef.current,
      items: itemsRef.current,
    }),
    [],
  );

  const clearSaveTimer = useCallback(() => {
    if (saveTimerRef.current !== null) {
      window.clearTimeout(saveTimerRef.current);
      saveTimerRef.current = null;
    }
  }, []);

  const clearSaveMaxWaitTimer = useCallback(() => {
    if (saveMaxWaitTimerRef.current !== null) {
      window.clearTimeout(saveMaxWaitTimerRef.current);
      saveMaxWaitTimerRef.current = null;
    }
  }, []);

  const applyIncomingNote = useCallback(
    (incomingNote: Note) => {
      const incomingDraft = noteToDraft(incomingNote);
      const signature = draftSignature(incomingDraft);
      const currentSignature = draftSignature(getCurrentDraft());

      clearSaveTimer();
      clearSaveMaxWaitTimer();
      latestSignatureRef.current = signature;
      lastRequestedSignatureRef.current = signature;
      lastSavedSignatureRef.current = signature;
      hasPendingLocalChangesRef.current = false;

      if (signature !== currentSignature) {
        updateTitle(incomingDraft.title);
        updateContent(incomingDraft.content);
        updateItems(incomingDraft.items);
        setItemIds(incomingDraft.items.map(createItemId));
      }
      if (mountedRef.current) setSaveStatus("saved");
    },
    [
      clearSaveMaxWaitTimer,
      clearSaveTimer,
      createItemId,
      getCurrentDraft,
      updateContent,
      updateItems,
      updateTitle,
    ],
  );

  const isEditorInputFocused = useCallback(() => {
    const activeElement = document.activeElement;
    return (
      (activeElement instanceof HTMLInputElement ||
        activeElement instanceof HTMLTextAreaElement) &&
      editorRef.current?.contains(activeElement)
    );
  }, [editorRef]);

  const applyPendingRemoteNote = useCallback(() => {
    const pendingNote = pendingRemoteNoteRef.current;
    if (
      !pendingNote ||
      hasPendingLocalChangesRef.current ||
      isEditorInputFocused()
    ) {
      return;
    }
    pendingRemoteNoteRef.current = null;
    applyIncomingNote(pendingNote);
  }, [applyIncomingNote, isEditorInputFocused]);

  const persistDraft = useCallback(
    (draft: NoteDraft): Promise<void> => {
      clearSaveTimer();
      clearSaveMaxWaitTimer();
      const signature = draftSignature(draft);
      latestSignatureRef.current = signature;
      lastRequestedSignatureRef.current = signature;
      hasPendingLocalChangesRef.current = true;
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
            hasPendingLocalChangesRef.current = false;
            setSaveStatus("saved");
            applyPendingRemoteNote();
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
    [
      applyPendingRemoteNote,
      clearSaveMaxWaitTimer,
      clearSaveTimer,
      note.id,
      note.title,
      note.type,
      queryClient,
    ],
  );

  const flushSave = useCallback((): Promise<void> => {
    clearSaveTimer();
    clearSaveMaxWaitTimer();
    const draft = getCurrentDraft();
    const signature = draftSignature(draft);
    if (signature === lastRequestedSignatureRef.current) {
      return saveQueueRef.current;
    }
    return persistDraft(draft);
  }, [clearSaveMaxWaitTimer, clearSaveTimer, getCurrentDraft, persistDraft]);

  useEffect(() => {
    const draft = getCurrentDraft();
    const signature = draftSignature(draft);
    latestSignatureRef.current = signature;
    if (signature === lastRequestedSignatureRef.current) {
      if (signature === lastSavedSignatureRef.current) {
        clearSaveMaxWaitTimer();
        hasPendingLocalChangesRef.current = false;
        if (mountedRef.current) setSaveStatus("saved");
        applyPendingRemoteNote();
      }
      return;
    }

    hasPendingLocalChangesRef.current = true;
    if (mountedRef.current) setSaveStatus("saving");
    clearSaveTimer();
    saveTimerRef.current = window.setTimeout(() => {
      saveTimerRef.current = null;
      void persistDraft(getCurrentDraft()).catch(() => undefined);
    }, AUTOSAVE_DELAY_MS);
    if (saveMaxWaitTimerRef.current === null) {
      saveMaxWaitTimerRef.current = window.setTimeout(() => {
        saveMaxWaitTimerRef.current = null;
        clearSaveTimer();
        void persistDraft(getCurrentDraft()).catch(() => undefined);
      }, AUTOSAVE_MAX_WAIT_MS);
    }
    return clearSaveTimer;
  }, [
    title,
    content,
    items,
    applyPendingRemoteNote,
    clearSaveMaxWaitTimer,
    clearSaveTimer,
    getCurrentDraft,
    persistDraft,
  ]);

  useEffect(() => {
    if (hasPendingLocalChangesRef.current || isEditorInputFocused()) {
      pendingRemoteNoteRef.current = note;
      return;
    }
    pendingRemoteNoteRef.current = null;
    applyIncomingNote(note);
  }, [note, applyIncomingNote, isEditorInputFocused]);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      clearSaveTimer();
      clearSaveMaxWaitTimer();
    };
  }, [clearSaveMaxWaitTimer, clearSaveTimer]);

  const saveItemsImmediately = useCallback(
    (nextItems: string[]) => {
      updateItems(nextItems);
      void persistDraft({ ...getCurrentDraft(), items: nextItems }).catch(
        () => undefined,
      );
    },
    [getCurrentDraft, persistDraft, updateItems],
  );

  return {
    title,
    content,
    items,
    itemIds,
    saveStatus,
    setItemIds,
    createItemId,
    getItems,
    updateTitle,
    updateContent,
    updateItems,
    getCurrentDraft,
    persistDraft,
    flushSave,
    saveItemsImmediately,
    applyPendingRemoteNote,
  };
}
