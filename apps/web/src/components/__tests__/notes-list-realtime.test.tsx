import { act } from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";
import type { Note } from "@/lib/api";

const apiMocks = vi.hoisted(() => ({
  getNotes: vi.fn(),
  createNote: vi.fn(),
  deleteNote: vi.fn(),
  updateMarkdownNote: vi.fn(),
  updateListNote: vi.fn(),
}));

vi.mock("@/lib/api", () => apiMocks);
vi.mock("@/lib/auth", () => ({
  useAuth: () => ({
    logout: vi.fn(),
    accountId: "account-1",
  }),
}));
vi.mock("../brand", () => ({ Brand: () => <div>Brand</div> }));
vi.mock("../theme-toggle", () => ({ ThemeToggle: () => <button>Theme</button> }));

import { NotesList } from "../notes-list";

describe("NotesList live updates", () => {
  it("restores and saves the preferred sidebar width", () => {
    (
      globalThis as typeof globalThis & {
        IS_REACT_ACT_ENVIRONMENT: boolean;
      }
    ).IS_REACT_ACT_ENVIRONMENT = true;
    localStorage.setItem("nnn-sidebar-width", "320");
    const queryClient = new QueryClient({
      defaultOptions: { queries: { staleTime: Infinity } },
    });
    queryClient.setQueryData(["notes", "account-1"], []);
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);

    act(() => {
      root.render(
        <QueryClientProvider client={queryClient}>
          <NotesList />
        </QueryClientProvider>,
      );
    });

    expect(
      container.firstElementChild?.getAttribute("style"),
    ).toContain("--sidebar-width: 320px");

    const resizeHandle = container.querySelector<HTMLElement>(
      '[role="separator"]',
    );
    act(() => {
      resizeHandle!.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true }),
      );
    });

    expect(localStorage.getItem("nnn-sidebar-width")).toBe("336");

    act(() => {
      root.unmount();
      queryClient.clear();
    });
    container.remove();
    localStorage.removeItem("nnn-sidebar-width");
  });

  it("keeps an open editor connected to refreshed query data", async () => {
    vi.useFakeTimers();
    (
      globalThis as typeof globalThis & {
        IS_REACT_ACT_ENVIRONMENT: boolean;
      }
    ).IS_REACT_ACT_ENVIRONMENT = true;
    const initialNote: Note = {
      id: "note-1",
      title: "Shared note",
      type: "markdown",
      content: "Original content",
      updated_at: "2026-07-13T14:00:00Z",
    };
    const queryClient = new QueryClient({
      defaultOptions: { queries: { staleTime: Infinity } },
    });
    queryClient.setQueryData(["notes", "account-1"], [initialNote]);
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);

    act(() => {
      root.render(
        <QueryClientProvider client={queryClient}>
          <NotesList />
        </QueryClientProvider>,
      );
    });
    const openButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Open Shared note"]',
    );
    act(() => openButton!.click());

    const editButton = [...container.querySelectorAll<HTMLButtonElement>(
      '[role="tab"]',
    )].find((button) => button.textContent === "edit");
    act(() => editButton!.click());

    await act(async () => {
      queryClient.setQueryData<Note[]>(["notes", "account-1"], [
        {
          ...initialNote,
          content: "Updated from another session",
          updated_at: "2026-07-13T14:00:01Z",
        },
      ]);
      await vi.runOnlyPendingTimersAsync();
    });

    expect(
      container.querySelector<HTMLTextAreaElement>('textarea[placeholder="Start writing..."]')
        ?.value,
    ).toBe("Updated from another session");

    act(() => {
      root.unmount();
      queryClient.clear();
    });
    container.remove();
    vi.useRealTimers();
  });
});
