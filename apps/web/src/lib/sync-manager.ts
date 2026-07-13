type PushFn = (docId: string, noteType: string) => Promise<void>;

let pushFn: PushFn | null = null;

export function registerPush(fn: PushFn | null) {
  pushFn = fn;
}

export async function pushNote(docId: string, noteType: string): Promise<void> {
  if (pushFn) {
    try {
      await pushFn(docId, noteType);
    } catch (err) {
      console.error('push failed:', err);
    }
  }
}
