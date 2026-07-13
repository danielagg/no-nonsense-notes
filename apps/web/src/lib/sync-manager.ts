type PushFn = (docId: string, noteType: string) => Promise<void>;

const LEGACY_PENDING_PUSH_KEY = 'no-nonsense-notes-pending-pushes';
const PENDING_PUSH_KEY_PREFIX = `${LEGACY_PENDING_PUSH_KEY}:`;

const pendingPushes = new Map<string, string>();
let activeAccountId: string | null =
  typeof localStorage === 'undefined' ? null : localStorage.getItem('nnn-account');
let accountGeneration = 0;
let pushFn: PushFn | null = null;
let flushPromise: Promise<void> | null = null;

function pendingPushKey(): string | null {
  return activeAccountId ? `${PENDING_PUSH_KEY_PREFIX}${activeAccountId}` : null;
}

export function registerPush(fn: PushFn | null) {
  pushFn = fn;
  if (fn && pendingPushes.size > 0) void flushPendingPushes();
}

export async function pushNote(docId: string, noteType: string): Promise<void> {
  if (!activeAccountId) throw new Error('cannot sync a note without an active account');
  pendingPushes.set(docId, noteType);
  savePendingPushes();
  await flushPendingPushes();
}

async function flushPendingPushes(): Promise<void> {
  if (flushPromise) return flushPromise;
  if (!pushFn) return;

  const generation = accountGeneration;
  const run = (async () => {
    while (generation === accountGeneration && pendingPushes.size > 0) {
      const currentPush = pushFn;
      if (!currentPush) break;
      const next = pendingPushes.entries().next().value as [string, string] | undefined;
      if (!next) break;
      const [docId, noteType] = next;
      try {
        await currentPush(docId, noteType);
        if (generation !== accountGeneration) break;
        pendingPushes.delete(docId);
        savePendingPushes();
      } catch (error) {
        console.error('push failed for', docId, error);
        break;
      }
    }
  })();

  flushPromise = run;
  try {
    await run;
  } finally {
    if (flushPromise === run) flushPromise = null;
  }
}

function savePendingPushes() {
  const key = pendingPushKey();
  if (!key) return;
  const entries = Array.from(pendingPushes, ([docId, noteType]) => ({ docId, noteType }));
  localStorage.setItem(key, JSON.stringify(entries));
}

export function loadPendingPushes(accountId: string) {
  if (activeAccountId !== accountId) {
    activeAccountId = accountId;
    accountGeneration += 1;
    pendingPushes.clear();
    flushPromise = null;
  }

  localStorage.removeItem(LEGACY_PENDING_PUSH_KEY);
  const key = pendingPushKey();
  if (!key) return;

  try {
    const json = localStorage.getItem(key);
    if (!json) return;
    const entries = JSON.parse(json) as Array<{ docId: string; noteType: string }>;
    for (const { docId, noteType } of entries) pendingPushes.set(docId, noteType);
  } catch {
    localStorage.removeItem(key);
  }
}
