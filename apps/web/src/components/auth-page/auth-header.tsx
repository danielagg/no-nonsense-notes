import { GitFork, Star } from "lucide-react";
import { Brand } from "@/components/brand";
import { ThemeToggle } from "@/components/theme-toggle";

const GITHUB_REPO_URL = "https://github.com/danielagg/no-nonsense-notes";

export function AuthHeader({ githubStarCount }: { githubStarCount: number }) {
  return (
    <header className="relative z-10 mx-auto flex h-20 w-full max-w-7xl items-center justify-between border-b border-primary/10 px-5 sm:px-8">
      <Brand />
      <div className="flex items-center gap-2">
        <a
          href={GITHUB_REPO_URL}
          target="_blank"
          rel="noreferrer"
          className="inline-flex h-9 items-center gap-2 rounded-md border border-primary/20 bg-primary/[0.04] px-3 font-heading text-[11px] font-semibold uppercase tracking-[0.08em] text-foreground transition-colors hover:border-primary/40 hover:bg-primary/10"
          aria-label={`View the open-source project on GitHub. ${githubStarCount} stars.`}
        >
          <GitFork className="size-3.5" />
          <span className="hidden sm:inline">Open source</span>
          <span className="inline-flex items-center gap-1 text-primary"><Star className="size-3 fill-current" />{githubStarCount}</span>
        </a>
        <ThemeToggle />
      </div>
    </header>
  );
}
