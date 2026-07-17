import { ArrowLeft, Check, RefreshCw } from "lucide-react";
import { Brand } from "@/components/brand";
import { ThemeToggle } from "@/components/theme-toggle";
import { Button } from "@/components/ui/button";
import type { SaveStatus } from "./model";

export function EditorHeader({
  saveStatus,
  onBack,
  onRetry,
}: {
  saveStatus: SaveStatus;
  onBack: () => void;
  onRetry: () => void;
}) {
  return (
    <header className="sticky top-[var(--sync-banner-height)] z-20 border-b border-primary/15 bg-background/88 backdrop-blur-xl">
      <div className="mx-auto flex h-16 w-full max-w-7xl items-center justify-between px-4 sm:px-8">
        <div className="flex items-center gap-2 sm:gap-5">
          <Brand compact className="hidden sm:flex" />
          <div className="hidden h-5 w-px bg-border sm:block" />
          <Button variant="ghost" onClick={onBack}>
            <ArrowLeft /> Back to notes
          </Button>
        </div>
        <div className="flex items-center gap-4">
          <ThemeToggle />
          <button
            type="button"
            className="min-w-24 flex items-center gap-2 text-xs opacity-50 font-heading uppercase"
            onClick={onRetry}
          >
            {saveStatus === "saving" ? (
              <span className="size-3.5 animate-spin rounded-full border-2 border-current border-r-transparent" />
            ) : saveStatus === "error" ? (
              <RefreshCw className="size-3" />
            ) : (
              <Check className="size-3" />
            )}
            {saveStatus === "saving" ? "Saving..." : saveStatus === "error" ? "Retry save" : "Saved"}
          </button>
        </div>
      </div>
    </header>
  );
}
