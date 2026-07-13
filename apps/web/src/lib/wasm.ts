interface WasmStore {
  free(): void;
  createNote(note_type: string, folder_id?: string | null): any;
  getNote(id: string): any;
  updateNote(id: string, content: string): any;
  updateList(id: string, items_json: string): any;
  listAddItem(id: string, item: string): any;
  listRemoveItem(id: string, item: string): any;
  softDelete(id: string): void;
  listNotes(folder_id?: string | null): any;
  searchNotes(query: string): any;
  applyRemoteUpdate(note_id: string, note_type: string, update_blob: Uint8Array): any;
  getSyncCursor(): bigint;
  setSyncCursor(seq: bigint): void;
  getDeviceId(): string;
  exportNoteBlob(note_id: string): Uint8Array;
}

interface WasmProtocol {
  encodePushFrame(doc_id: string, device_id: string, note_type: string, loro_blob: Uint8Array): Uint8Array;
  decodePushResponse(data: Uint8Array): bigint;
  encodePullRequest(last_seq: bigint): string;
  decodePullResponse(text: string): {
    currentSeq: number;
    entries: Array<{ docId: string; noteType: string; loroBlob: Uint8Array }>;
  };
}

let storePromise: Promise<{ store: WasmStore; protocol: WasmProtocol }> | null = null;

async function getStore(): Promise<WasmStore> {
  return (await getStoreAndProtocol()).store;
}

async function getStoreAndProtocol(): Promise<{ store: WasmStore; protocol: WasmProtocol }> {
  if (!storePromise) {
    const p = (async () => {
      const wasmUrl = new URL('./wasm-pkg/no_nonsense_notes_wasm_bg.wasm', import.meta.url).href;
      const mod = await import('./wasm-pkg/no_nonsense_notes_wasm.js');
      await mod.default(wasmUrl);
      const store = new mod.WasmStore();
      const protocol = {
        encodePushFrame: mod.encodePushFrame,
        decodePushResponse: mod.decodePushResponse,
        encodePullRequest: mod.encodePullRequest,
        decodePullResponse: mod.decodePullResponse,
      };
      return { store, protocol };
    })();
    storePromise = p;
    p.catch(() => {
      if (storePromise === p) storePromise = null;
    });
  }
  return storePromise;
}

export interface WasmNote {
  id: string;
  folderId: string | null;
  noteType: string;
  title: string;
  contentPlaintext: string;
  contentLoroBlob: Uint8Array;
  contentHash: Uint8Array;
  createdAt: string;
  updatedAt: string;
  isDeleted: boolean;
  deletedAt: string | null;
  sortOrder: number;
}

export interface PullResponseEntry {
  docId: string;
  noteType: string;
  loroBlob: Uint8Array;
}

export interface PullResponse {
  currentSeq: number;
  entries: PullResponseEntry[];
}

export async function getNotes(): Promise<WasmNote[]> {
  const store = await getStore();
  const raw = store.listNotes(null);
  return raw as WasmNote[];
}

export async function createNote(type: 'markdown' | 'list'): Promise<WasmNote> {
  const store = await getStore();
  const raw = store.createNote(type, null);
  return raw as WasmNote;
}

export async function updateNote(id: string, content: string): Promise<WasmNote> {
  const store = await getStore();
  const raw = store.updateNote(id, content);
  return raw as WasmNote;
}

export async function updateList(id: string, items: string[]): Promise<WasmNote> {
  const store = await getStore();
  const raw = store.updateList(id, JSON.stringify(items));
  return raw as WasmNote;
}

export async function listAddItem(id: string, item: string): Promise<WasmNote> {
  const store = await getStore();
  const raw = store.listAddItem(id, item);
  return raw as WasmNote;
}

export async function listRemoveItem(id: string, item: string): Promise<WasmNote> {
  const store = await getStore();
  const raw = store.listRemoveItem(id, item);
  return raw as WasmNote;
}

export async function softDelete(id: string): Promise<void> {
  const store = await getStore();
  store.softDelete(id);
}

export async function searchNotes(query: string): Promise<WasmNote[]> {
  const store = await getStore();
  const raw = store.searchNotes(query);
  return raw as WasmNote[];
}

export async function applyRemoteUpdate(
  noteId: string,
  noteType: string,
  updateBlob: Uint8Array,
): Promise<WasmNote> {
  const store = await getStore();
  const raw = store.applyRemoteUpdate(noteId, noteType, updateBlob);
  return raw as WasmNote;
}

export async function getSyncCursor(): Promise<number> {
  const store = await getStore();
  return Number(store.getSyncCursor());
}

export async function setSyncCursor(seq: number): Promise<void> {
  const store = await getStore();
  store.setSyncCursor(BigInt(seq));
}

export async function getDeviceId(): Promise<string> {
  const store = await getStore();
  return store.getDeviceId();
}

export async function exportNoteBlob(noteId: string): Promise<Uint8Array> {
  const store = await getStore();
  return store.exportNoteBlob(noteId);
}

export async function encodePushFrame(
  docId: string,
  deviceId: string,
  noteType: string,
  loroBlob: Uint8Array,
): Promise<Uint8Array> {
  const { protocol } = await getStoreAndProtocol();
  return protocol.encodePushFrame(docId, deviceId, noteType, loroBlob);
}

export async function decodePushResponse(data: Uint8Array): Promise<number> {
  const { protocol } = await getStoreAndProtocol();
  return Number(protocol.decodePushResponse(data));
}

export async function encodePullRequest(lastSeq: number): Promise<string> {
  const { protocol } = await getStoreAndProtocol();
  return protocol.encodePullRequest(BigInt(lastSeq));
}

export async function decodePullResponse(text: string): Promise<PullResponse> {
  const { protocol } = await getStoreAndProtocol();
  return protocol.decodePullResponse(text) as PullResponse;
}
