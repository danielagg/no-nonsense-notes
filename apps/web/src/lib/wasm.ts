let storePromise: Promise<import('../generated-wasm/no_nonsense_notes_wasm').WasmStore> | null = null;

async function getStore() {
  if (!storePromise) {
    storePromise = (async () => {
      const wasmUrl = new URL('./wasm-pkg/no_nonsense_notes_wasm_bg.wasm', import.meta.url).href;
      const { default: init, WasmStore } = await import('./wasm-pkg/no_nonsense_notes_wasm.js');
      await init(wasmUrl);
      return new WasmStore();
    })();
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
