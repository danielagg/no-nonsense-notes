import type { MutableRefObject } from "react";
import { useBlocker } from "@tanstack/react-router";

export function NavigationBlocker({
  onBack,
  allowNavigationRef,
}: {
  onBack: () => Promise<void>;
  allowNavigationRef: MutableRefObject<boolean>;
}) {
  useBlocker({
    shouldBlockFn: () => {
      if (allowNavigationRef.current) {
        allowNavigationRef.current = false;
        return false;
      }
      void onBack();
      return true;
    },
    enableBeforeUnload: false,
  });

  return null;
}
