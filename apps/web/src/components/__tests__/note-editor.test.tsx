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
  it("renders markdown in preview mode without leaving the editor", () => {
    const container = renderEditor({
      ...markdownNote,
      content: "# Release notes\n\n**Private** by default.",
    });
    const previewButton = [...container.querySelectorAll<HTMLButtonElement>(
      '[role="tab"]',
    )].find((button) => button.textContent === "preview");

    act(() => previewButton!.click());

    expect(
      container.querySelector('[role="tabpanel"] h1')?.textContent,
    ).toBe("Release notes");
    expect(container.querySelector('[role="tabpanel"] strong')?.textContent).toBe(
      "Private",
    );
    expect(
      container.querySelector('textarea[placeholder="Start writing..."]'),
    ).toBeNull();

    const editButton = [...container.querySelectorAll<HTMLButtonElement>(
      '[role="tab"]',
    )].find((button) => button.textContent === "edit");
    act(() => editButton!.click());

    expect(
      container.querySelector('textarea[placeholder="Start writing..."]'),
    ).not.toBeNull();
  });

  it("debounces text edits", async () => {
    const container = renderEditor(markdownNote);
    switchToEditMode(container);
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

  it("saves periodically during continuous typing", async () => {
    const container = renderEditor(markdownNote);
    switchToEditMode(container);
    const textarea = container.querySelector<HTMLTextAreaElement>(
      'textarea[placeholder="Start writing..."]',
    );

    act(() => changeValue(textarea!, "Continuous edit 1"));
    await act(async () => vi.advanceTimersByTime(400));
    act(() => changeValue(textarea!, "Continuous edit 2"));
    await act(async () => vi.advanceTimersByTime(400));
    act(() => changeValue(textarea!, "Continuous edit 3"));
    await act(async () => vi.advanceTimersByTime(399));
    expect(mocks.updateMarkdownNote).not.toHaveBeenCalled();

    await act(async () => {
      vi.advanceTimersByTime(1);
      await Promise.resolve();
    });
    expect(mocks.updateMarkdownNote).toHaveBeenCalledWith(
      "note-1",
      "Continuous edit 3",
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
      ["First", "[ ] "],
      null,
    );
    expect(
      [...container.querySelectorAll<HTMLInputElement>('input[placeholder="List item"]')].map(
        (input) => input.value,
      ),
    ).toEqual(["First", ""]);
  });

  it("keeps a newly added blank checklist item after the saved note refreshes", async () => {
    const onBack = vi.fn();
    const listNote: Note = {
      ...markdownNote,
      type: "list",
      content: "First",
      items: ["First"],
    };
    const container = renderEditor(listNote, onBack);
    const addButton = [...container.querySelectorAll("button")].find((button) =>
      button.textContent?.includes("Add item"),
    );

    await act(async () => {
      addButton!.click();
      await Promise.resolve();
      await Promise.resolve();
    });
    rerenderEditor(
      {
        ...listNote,
        content: "First\n[ ] ",
        items: ["First", "[ ] "],
        updated_at: "2026-07-13T12:01:00Z",
      },
      onBack,
    );

    expect(
      [...container.querySelectorAll<HTMLInputElement>('input[placeholder="List item"]')].map(
        (input) => input.value,
      ),
    ).toEqual(["First", ""]);
  });

  it("inserts and focuses a blank checklist item after the current item on Enter", async () => {
    const listNote: Note = {
      ...markdownNote,
      type: "list",
      content: "First\nThird",
      items: ["First", "Third"],
    };
    const container = renderEditor(listNote);
    const inputs = container.querySelectorAll<HTMLInputElement>(
      'input[placeholder="List item"]',
    );

    await act(async () => {
      inputs[0].dispatchEvent(
        new KeyboardEvent("keydown", { key: "Enter", bubbles: true }),
      );
      await Promise.resolve();
    });

    const updatedInputs = container.querySelectorAll<HTMLInputElement>(
      'input[placeholder="List item"]',
    );
    expect([...updatedInputs].map((input) => input.value)).toEqual([
      "First",
      "",
      "Third",
    ]);
    expect(document.activeElement).toBe(updatedInputs[1]);
    expect(mocks.updateListNote).toHaveBeenCalledWith(
      "note-1",
      ["First", "[ ] ", "Third"],
      null,
    );
  });

  it("keeps focus while a stale list-note refresh arrives after autosave", async () => {
    const onBack = vi.fn();
    const listNote: Note = {
      ...markdownNote,
      type: "list",
      content: "First",
      items: ["First"],
    };
    const container = renderEditor(listNote, onBack);
    const input = container.querySelector<HTMLInputElement>(
      'input[placeholder="List item"]',
    );
    input!.focus();

    act(() => changeValue(input!, "Updated first"));
    await act(async () => {
      vi.advanceTimersByTime(650);
      await Promise.resolve();
      await Promise.resolve();
    });
    rerenderEditor(listNote, onBack);

    expect(input?.value).toBe("Updated first");
    expect(document.activeElement).toBe(input);
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
    switchToEditMode(container);
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
    switchToEditMode(container);
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

  it("shows remote markdown changes while the editor is idle", () => {
    const onBack = vi.fn();
    const container = renderEditor(markdownNote, onBack);
    switchToEditMode(container);
    const remoteNote: Note = {
      ...markdownNote,
      title: "Changed elsewhere",
      content: "Live remote content",
      updated_at: "2026-07-13T12:01:00Z",
    };

    rerenderEditor(remoteNote, onBack);

    expect(
      container.querySelector<HTMLInputElement>('input[placeholder="Untitled note"]')?.value,
    ).toBe("Changed elsewhere");
    expect(
      container.querySelector<HTMLTextAreaElement>('textarea[placeholder="Start writing..."]')
        ?.value,
    ).toBe("Live remote content");
    expect(mocks.updateMarkdownNote).not.toHaveBeenCalled();
  });

  it("shows remote checklist changes while the editor is idle", () => {
    const onBack = vi.fn();
    const listNote: Note = {
      ...markdownNote,
      type: "list",
      content: "First",
      items: ["First"],
    };
    const container = renderEditor(listNote, onBack);

    rerenderEditor(
      {
        ...listNote,
        content: "Remote first\nRemote second",
        items: ["Remote first", "Remote second"],
        updated_at: "2026-07-13T12:01:00Z",
      },
      onBack,
    );

    expect(
      [...container.querySelectorAll<HTMLInputElement>('input[placeholder="List item"]')].map(
        (input) => input.value,
      ),
    ).toEqual(["Remote first", "Remote second"]);
    expect(mocks.updateListNote).not.toHaveBeenCalled();
  });

  it("does not overwrite a local draft with an incoming update", () => {
    const onBack = vi.fn();
    const container = renderEditor(markdownNote, onBack);
    switchToEditMode(container);
    const textarea = container.querySelector<HTMLTextAreaElement>(
      'textarea[placeholder="Start writing..."]',
    );

    act(() => changeValue(textarea!, "Local draft in progress"));
    rerenderEditor(
      {
        ...markdownNote,
        content: "Incoming remote content",
        updated_at: "2026-07-13T12:01:00Z",
      },
      onBack,
    );

    expect(textarea?.value).toBe("Local draft in progress");
    expect(mocks.updateMarkdownNote).not.toHaveBeenCalled();
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

function rerenderEditor(note: Note, onBack: () => void) {
  const root = mountedRoots[mountedRoots.length - 1];
  act(() => root.render(<NoteEditor note={note} onBack={onBack} />));
}

function switchToEditMode(container: HTMLDivElement) {
  const editButton = [...container.querySelectorAll<HTMLButtonElement>(
    '[role="tab"]',
  )].find((button) => button.textContent === "edit");
  act(() => editButton!.click());
}

function changeValue(element: HTMLInputElement | HTMLTextAreaElement, value: string) {
  const tag = (element as Element).tagName;
  const prototype =
    tag === "INPUT" ? HTMLInputElement.prototype : HTMLTextAreaElement.prototype;
  const setter = Object.getOwnPropertyDescriptor(prototype, "value")?.set;
  setter?.call(element, value);
  element.dispatchEvent(new Event("input", { bubbles: true }));
}
