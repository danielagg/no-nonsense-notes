import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { SyncStatusBar } from "../sync-badge";

describe("SyncStatusBar", () => {
  it("stays hidden while sync is healthy", () => {
    expect(renderToStaticMarkup(<SyncStatusBar status="connected" />)).toBe("");
  });

  it.each([
    ["disconnected", "offline"],
    ["connecting", "Connecting to sync"],
    ["error", "Sync issue"],
  ] as const)("shows the %s state", (status, label) => {
    const markup = renderToStaticMarkup(<SyncStatusBar status={status} />);

    expect(markup).toContain(`data-sync-status="${status}"`);
    expect(markup).toContain('role="status"');
    expect(markup).toContain(label);
  });
});
