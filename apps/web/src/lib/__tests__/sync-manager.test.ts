import { beforeEach, describe, expect, it, vi } from 'vitest';

import { loadPendingPushes, pushNote, registerPush } from '../sync-manager';

beforeEach(() => {
  localStorage.clear();
  registerPush(null);
  loadPendingPushes(`account-${crypto.randomUUID()}`);
});

describe('sync manager', () => {
  it('keeps a pending push until the server acknowledgement resolves', async () => {
    let acknowledge: (() => void) | undefined;
    const push = vi.fn(
      () => new Promise<void>((resolve) => {
        acknowledge = resolve;
      }),
    );
    registerPush(push);

    const pending = pushNote('note-1', 'markdown');
    await vi.waitFor(() => expect(push).toHaveBeenCalledWith('note-1', 'markdown'));

    const key = Object.keys(localStorage).find((candidate) =>
      candidate.startsWith('no-nonsense-notes-pending-pushes:'),
    );
    expect(key).toBeDefined();
    expect(localStorage.getItem(key!)).toContain('note-1');

    acknowledge?.();
    await pending;
    expect(localStorage.getItem(key!)).toBe('[]');
  });

  it('retains an offline tombstone for the next connection', async () => {
    registerPush(null);
    await pushNote('note-offline', 'delete');

    const push = vi.fn(async () => {});
    registerPush(push);
    await vi.waitFor(() => expect(push).toHaveBeenCalledWith('note-offline', 'delete'));
  });

  it('does not load another account pending queue', async () => {
    registerPush(null);
    loadPendingPushes('account-a');
    await pushNote('private-note', 'markdown');

    loadPendingPushes('account-b');
    const push = vi.fn(async () => {});
    registerPush(push);

    await new Promise<void>((resolve) => queueMicrotask(() => resolve()));
    expect(push).not.toHaveBeenCalled();
  });
});
