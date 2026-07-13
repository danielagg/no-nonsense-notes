import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Note } from "@/lib/api";

const mocks = vi.hoisted(() => ({
  updateMarkdownNote: vi.fn(),
  updateListNote: vi.fn(),
  invalidateQueries: vi.fn(() => Promise.resolve()),
}));

vi.mock("@/lib/api", () => ({
  updateMarkdownNote: mocks.updateMarkdownNote,
  updateListNote: mocks.updateListNote,
}));

vi.mock("@tanstack/react-query", () => ({
  useQueryClient: () => ({ invalidateQueries: mocks.invalidateQueries }),
}));

vi.mock("../brand", () => ({ Brand: () => <div>Brand</div> }));
vi.mock("../theme-toggle", () => ({ ThemeToggle: () => <button>Theme</button> }));

import { NoteEditor } from "../note-editor";

const markdownNote: Note = {
  id: "note-1",
  title: "Original title",
  type: "markdown",
  content: "Original content",
  updated_at: "2026-07-13T12:00:00Z",
};

const mountedRoots: Root[] = [];

beforeEach(() => {
  vi.useFakeTimers();
  vi.clearAllMocks();
  mocks.updateMarkdownNote.mockResolvedValue(markdownNote);
  mocks.updateListNote.mockResolvedValue(markdownNote);
  (
    globalThis as typeof globalThis & {
      IS_REACT_ACT_ENVIRONMENT: boolean;
    }
  ).IS_REACT_ACT_ENVIRONMENT = true;
});

afterEach(() => {
  mountedRoots.forEach((root) => {
    act(() => root.unmount());
  });
  mountedRoots.length = 0;
  vi.useRealTimers();
});

describe("NoteEditor autosave", () => {
  it("debounces text edits", async () => {
    const container = renderEditor(markdownNote);
    const textarea = container.querySelector<HTMLTextAreaElement>(
      'textarea[placeholder="Start writing..."]',
    );
    expect(textarea).not.toBeNull();

    act(() => changeValue(textarea!, "Updated content"));
    await act(async () => vi.advanceTimersByTime(649));
    expect(mocks.updateMarkdownNote).not.toHaveBeenCalled();

    await act(async () => {
      vi.advanceTimersByTime(1);
      await Promise.resolve();
    });
    expect(mocks.updateMarkdownNote).toHaveBeenCalledWith(
      "note-1",
      "Updated content",
      null,
    );
  });

  it("saves checklist structure changes immediately", async () => {
    const listNote: Note = {
      ...markdownNote,
      type: "list",
      content: "First",
      items: ["First"],
    };
    const container = renderEditor(listNote);
    const addButton = [...container.querySelectorAll("button")].find((button) =>
      button.textContent?.includes("Add item"),
    );
    expect(addButton).toBeDefined();

    await act(async () => {
      addButton!.click();
      await Promise.resolve();
    });
    expect(mocks.updateListNote).toHaveBeenCalledWith(
      "note-1",
      ["First", ""],
      null,
    );
  });

  it("flushes a pending edit before going back", async () => {
    let finishSave: (() => void) | undefined;
    mocks.updateMarkdownNote.mockImplementation(
      () => new Promise<Note>((resolve) => {
        finishSave = () => resolve(markdownNote);
      }),
    );
    const onBack = vi.fn();
    const container = renderEditor(markdownNote, onBack);
    const textarea = container.querySelector<HTMLTextAreaElement>(
      'textarea[placeholder="Start writing..."]',
    );
    const backButton = [...container.querySelectorAll("button")].find((button) =>
      button.textContent?.includes("Back to notes"),
    );

    act(() => changeValue(textarea!, "Save before leaving"));
    await act(async () => {
      backButton!.click();
      await Promise.resolve();
    });
    expect(mocks.updateMarkdownNote).toHaveBeenCalled();
    expect(onBack).not.toHaveBeenCalled();

    await act(async () => {
      finishSave!();
      await Promise.resolve();
    });
    expect(onBack).toHaveBeenCalledOnce();
  });

  it("queues the latest draft behind an in-flight save", async () => {
    let finishFirstSave: (() => void) | undefined;
    mocks.updateMarkdownNote
      .mockImplementationOnce(
        () =>
          new Promise<Note>((resolve) => {
            finishFirstSave = () => resolve(markdownNote);
          }),
      )
      .mockResolvedValue(markdownNote);
    const container = renderEditor(markdownNote);
    const textarea = container.querySelector<HTMLTextAreaElement>(
      'textarea[placeholder="Start writing..."]',
    );

    act(() => changeValue(textarea!, "First draft"));
    await act(async () => {
      vi.advanceTimersByTime(650);
      await Promise.resolve();
    });
    act(() => changeValue(textarea!, "Original content"));
    await act(async () => {
      vi.advanceTimersByTime(650);
      await Promise.resolve();
    });
    expect(mocks.updateMarkdownNote).toHaveBeenCalledTimes(1);

    await act(async () => {
      finishFirstSave!();
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(mocks.updateMarkdownNote).toHaveBeenNthCalledWith(
      2,
      "note-1",
      "Original content",
      null,
    );
  });
});

function renderEditor(note: Note, onBack = vi.fn()): HTMLDivElement {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  mountedRoots.push(root);
  act(() => root.render(<NoteEditor note={note} onBack={onBack} />));
  return container;
}

function changeValue(element: HTMLInputElement | HTMLTextAreaElement, value: string) {
  const prototype =
    element instanceof HTMLInputElement ? HTMLInputElement.prototype : HTMLTextAreaElement.prototype;
  const setter = Object.getOwnPropertyDescriptor(prototype, "value")?.set;
  setter?.call(element, value);
  element.dispatchEvent(new Event("input", { bubbles: true }));
}
