import {
  useRef,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import { LogOut, NotebookPen } from "lucide-react";
import { Brand } from "../brand";
import { ThemeToggle } from "../theme-toggle";
import { Button } from "../ui/button";
import {
  MAX_SIDEBAR_WIDTH,
  MIN_SIDEBAR_WIDTH,
  SIDEBAR_WIDTH_STORAGE_KEY,
  clampSidebarWidth,
} from "./use-sidebar-width";

interface Props {
  noteCount: number;
  width: number;
  setWidth: Dispatch<SetStateAction<number>>;
  onLogout: () => void;
}

export function NotesSidebar({ noteCount, width, setWidth, onLogout }: Props) {
  const [isResizing, setIsResizing] = useState(false);
  const resizeStart = useRef<{ x: number; width: number } | null>(null);

  const updateWidth = (nextWidth: number, persist = false) => {
    const clampedWidth = clampSidebarWidth(nextWidth);
    setWidth(clampedWidth);
    if (persist) {
      localStorage.setItem(SIDEBAR_WIDTH_STORAGE_KEY, String(clampedWidth));
    }
  };

  const finishResize = (clientX: number) => {
    if (!resizeStart.current) return;
    const { x, width: startWidth } = resizeStart.current;
    updateWidth(startWidth + clientX - x, true);
    resizeStart.current = null;
    setIsResizing(false);
  };

  return (
    <aside
      className="relative hidden h-[calc(100svh-var(--sync-banner-height))] flex-col border-r border-primary/15 bg-sidebar/92 px-3 py-4 backdrop-blur md:sticky md:top-[var(--sync-banner-height)] md:flex"
    >
      <Brand className="px-2" />

      <nav className="mt-8">
        <div className="flex h-10 items-center gap-3 rounded-md border border-primary/20 bg-primary/[0.07] px-3 font-heading text-xs font-semibold uppercase tracking-[0.045em] text-sidebar-accent-foreground">
          <NotebookPen className="size-4 text-primary" />
          All notes
          <span className="ml-auto border border-primary/15 bg-background/50 px-1.5 py-0.5 font-mono text-[10px] tabular-nums text-primary">
            {noteCount}
          </span>
        </div>
      </nav>

      <div className="mt-auto space-y-1">
        <ThemeToggle showLabel />
        <Button
          variant="ghost"
          className="w-full justify-start"
          onClick={onLogout}
        >
          <LogOut />
          Log out
        </Button>
      </div>

      <div
        role="separator"
        aria-label="Resize notes sidebar"
        aria-orientation="vertical"
        aria-valuemin={MIN_SIDEBAR_WIDTH}
        aria-valuemax={MAX_SIDEBAR_WIDTH}
        aria-valuenow={width}
        tabIndex={0}
        className={`absolute -right-1 top-0 z-20 hidden h-full w-2 touch-none select-none cursor-col-resize md:block ${
          isResizing ? "bg-primary/25" : "hover:bg-primary/12"
        }`}
        onPointerDown={(event) => {
          event.currentTarget.setPointerCapture(event.pointerId);
          resizeStart.current = { x: event.clientX, width };
          setIsResizing(true);
        }}
        onPointerMove={(event) => {
          if (!resizeStart.current) return;
          updateWidth(
            resizeStart.current.width + event.clientX - resizeStart.current.x,
          );
        }}
        onPointerUp={(event) => finishResize(event.clientX)}
        onPointerCancel={(event) => finishResize(event.clientX)}
        onKeyDown={(event) => {
          const step = event.shiftKey ? 32 : 16;
          if (event.key === "ArrowLeft") {
            event.preventDefault();
            updateWidth(width - step, true);
          }
          if (event.key === "ArrowRight") {
            event.preventDefault();
            updateWidth(width + step, true);
          }
          if (event.key === "Home") {
            event.preventDefault();
            updateWidth(MIN_SIDEBAR_WIDTH, true);
          }
          if (event.key === "End") {
            event.preventDefault();
            updateWidth(MAX_SIDEBAR_WIDTH, true);
          }
        }}
      />
    </aside>
  );
}
