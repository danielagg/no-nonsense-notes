import { LogOut } from "lucide-react";
import { Brand } from "../brand";
import { ThemeToggle } from "../theme-toggle";
import { Button } from "../ui/button";

interface Props {
  onLogout: () => void;
}

export function MobileHeader({ onLogout }: Props) {
  return (
    <header className="flex h-16 items-center justify-between border-b border-primary/10 bg-background/85 px-4 backdrop-blur md:hidden">
      <Brand />
      <div className="flex items-center gap-1">
        <ThemeToggle />
        <Button
          variant="ghost"
          size="icon"
          onClick={onLogout}
          aria-label="Log out"
          title="Log out"
        >
          <LogOut />
        </Button>
      </div>
    </header>
  );
}
