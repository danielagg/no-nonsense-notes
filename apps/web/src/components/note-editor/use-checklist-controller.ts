import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type Dispatch,
  type KeyboardEvent,
  type PointerEvent as ReactPointerEvent,
  type SetStateAction,
} from "react";
import { moveArrayItem, parseListItem, type NoteDraft } from "./model";

interface DragPointerEvent {
  pointerId: number;
  clientY: number;
  preventDefault: () => void;
}

interface Options {
  items: string[];
  itemIds: string[];
  setItemIds: Dispatch<SetStateAction<string[]>>;
  createItemId: () => string;
  getItems: () => string[];
  updateItems: (items: string[]) => void;
  getCurrentDraft: () => NoteDraft;
  persistDraft: (draft: NoteDraft) => Promise<void>;
  saveItemsImmediately: (items: string[]) => void;
}

export function useChecklistController({
  items,
  itemIds,
  setItemIds,
  createItemId,
  getItems,
  updateItems,
  getCurrentDraft,
  persistDraft,
  saveItemsImmediately,
}: Options) {
  const [draggingIndex, setDraggingIndex] = useState<number | null>(null);
  const draggingIndexRef = useRef<number | null>(null);
  const dragPointerIdRef = useRef<number | null>(null);
  const dragTimerRef = useRef<number | null>(null);
  const dragDidReorderRef = useRef(false);
  const itemRowsRef = useRef<Array<HTMLDivElement | null>>([]);
  const itemInputRefs = useRef<Array<HTMLInputElement | null>>([]);
  const pendingItemFocusIdRef = useRef<string | null>(null);
  const finishDragRef = useRef<() => void>(() => undefined);
  const dragPointerMoveRef = useRef<(event: DragPointerEvent) => void>(
    () => undefined,
  );

  const addItem = useCallback(() => {
    saveItemsImmediately([...getItems(), "[ ] "]);
    setItemIds((previous) => [...previous, createItemId()]);
  }, [createItemId, getItems, saveItemsImmediately, setItemIds]);

  const insertItemAfter = useCallback(
    (index: number) => {
      const nextItems = [...getItems()];
      nextItems.splice(index + 1, 0, "[ ] ");
      const itemId = createItemId();
      pendingItemFocusIdRef.current = itemId;
      saveItemsImmediately(nextItems);
      setItemIds((previous) => {
        const nextIds = [...previous];
        nextIds.splice(index + 1, 0, itemId);
        return nextIds;
      });
    },
    [createItemId, getItems, saveItemsImmediately, setItemIds],
  );

  useEffect(() => {
    const itemId = pendingItemFocusIdRef.current;
    if (!itemId) return;
    const index = itemIds.indexOf(itemId);
    const input = itemInputRefs.current[index];
    if (!input) return;
    input.focus();
    pendingItemFocusIdRef.current = null;
  }, [itemIds]);

  const updateItem = useCallback(
    (index: number, value: string) => {
      updateItems(
        getItems().map((item, itemIndex) =>
          itemIndex === index ? value : item,
        ),
      );
    },
    [getItems, updateItems],
  );

  const removeItem = useCallback(
    (index: number) => {
      saveItemsImmediately(
        getItems().filter((_, itemIndex) => itemIndex !== index),
      );
      setItemIds((previous) =>
        previous.filter((_, itemIndex) => itemIndex !== index),
      );
    },
    [getItems, saveItemsImmediately, setItemIds],
  );

  const toggleItem = useCallback(
    (index: number) => {
      saveItemsImmediately(
        getItems().map((item, itemIndex) => {
          if (itemIndex !== index) return item;
          const { isChecked, text } = parseListItem(item);
          return `${isChecked ? "[ ]" : "[x]"} ${text}`;
        }),
      );
    },
    [getItems, saveItemsImmediately],
  );

  const reorderItem = useCallback(
    (fromIndex: number, toIndex: number, saveImmediately = false) => {
      if (fromIndex === toIndex) return;
      const nextItems = moveArrayItem(getItems(), fromIndex, toIndex);
      updateItems(nextItems);
      setItemIds((previous) =>
        moveArrayItem(previous, fromIndex, toIndex),
      );
      if (saveImmediately) {
        void persistDraft({ ...getCurrentDraft(), items: nextItems }).catch(
          () => undefined,
        );
      }
    },
    [getCurrentDraft, getItems, persistDraft, setItemIds, updateItems],
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
    dragPointerIdRef.current = null;
    draggingIndexRef.current = null;
    setDraggingIndex(null);
    if (shouldSave) {
      void persistDraft(getCurrentDraft()).catch(() => undefined);
    }
  }, [clearDragTimer, getCurrentDraft, persistDraft]);

  const handleDragPointerDown = useCallback(
    (event: ReactPointerEvent<HTMLButtonElement>, index: number) => {
      if (
        !event.isPrimary ||
        (event.pointerType === "mouse" && event.button !== 0)
      ) {
        return;
      }

      clearDragTimer();
      dragDidReorderRef.current = false;
      dragPointerIdRef.current = event.pointerId;

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
    (event: DragPointerEvent) => {
      if (event.pointerId !== dragPointerIdRef.current) return;
      const fromIndex = draggingIndexRef.current;
      if (fromIndex === null) return;

      event.preventDefault();
      const edgeSize = 72;
      if (event.clientY < edgeSize) window.scrollBy({ top: -12 });
      if (event.clientY > window.innerHeight - edgeSize) {
        window.scrollBy({ top: 12 });
      }

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

  useEffect(() => {
    finishDragRef.current = finishDrag;
    dragPointerMoveRef.current = handleDragPointerMove;
  }, [finishDrag, handleDragPointerMove]);

  useEffect(() => {
    const handlePointerMove = (event: globalThis.PointerEvent) => {
      dragPointerMoveRef.current(event);
    };
    const handlePointerEnd = (event: globalThis.PointerEvent) => {
      if (event.pointerId === dragPointerIdRef.current) finishDragRef.current();
    };

    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerEnd);
    window.addEventListener("pointercancel", handlePointerEnd);
    return () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerEnd);
      window.removeEventListener("pointercancel", handlePointerEnd);
      finishDragRef.current();
    };
  }, []);

  const handleReorderKeyDown = useCallback(
    (event: KeyboardEvent<HTMLButtonElement>, index: number) => {
      const direction =
        event.key === "ArrowUp" ? -1 : event.key === "ArrowDown" ? 1 : 0;
      if (direction === 0) return;
      const targetIndex = index + direction;
      if (targetIndex < 0 || targetIndex >= items.length) return;
      event.preventDefault();
      reorderItem(index, targetIndex, true);
    },
    [items.length, reorderItem],
  );

  return {
    draggingIndex,
    itemRowsRef,
    itemInputRefs,
    addItem,
    insertItemAfter,
    updateItem,
    removeItem,
    toggleItem,
    handleDragPointerDown,
    handleReorderKeyDown,
  };
}
