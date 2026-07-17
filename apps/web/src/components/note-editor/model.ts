import type { Note } from "@/lib/api";

export interface NoteDraft {
  title: string;
  content: string;
  items: string[];
}

export type SaveStatus = "saved" | "saving" | "error";
export type MarkdownMode = "edit" | "preview";

export function draftSignature(draft: NoteDraft): string {
  return JSON.stringify([draft.title, draft.content, draft.items]);
}

export function noteToDraft(note: Note): NoteDraft {
  return {
    title: note.title,
    content: note.content,
    items: note.items ?? [],
  };
}

export function parseListItem(item: string) {
  if (item.startsWith("[x] ")) {
    return { isChecked: true, text: item.slice(4), hasCheckboxPrefix: true };
  }
  if (item.startsWith("[ ] ")) {
    return { isChecked: false, text: item.slice(4), hasCheckboxPrefix: true };
  }
  return { isChecked: false, text: item, hasCheckboxPrefix: false };
}

export function moveArrayItem<T>(items: T[], fromIndex: number, toIndex: number) {
  const next = [...items];
  const [movedItem] = next.splice(fromIndex, 1);
  next.splice(toIndex, 0, movedItem);
  return next;
}
