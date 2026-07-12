import { ensureLoroReady, getLoroDocClass } from './loro-init';
import type { LoroDoc as LoroDocType } from './loro-init';

await ensureLoroReady();
const LoroDoc = getLoroDocClass();

export interface Note {
  id: string;
  title: string;
  type: 'markdown' | 'list';
  content: string;
  items?: string[];
  updated_at: string;
}

const INDEX_KEY = 'nnn-notes-index';
const SNAPSHOT_PREFIX = 'nnn-snap-';
const DEVICE_ID_KEY = 'nnn-device-id';

export function getDeviceId(): string {
  let id = localStorage.getItem(DEVICE_ID_KEY);
  if (!id) {
    id = crypto.randomUUID();
    localStorage.setItem(DEVICE_ID_KEY, id);
  }
  return id;
}

function getIndex(): Note[] {
  const raw = localStorage.getItem(INDEX_KEY);
  return raw ? JSON.parse(raw) : [];
}

function saveIndex(notes: Note[]) {
  localStorage.setItem(INDEX_KEY, JSON.stringify(notes));
}

function snapshotKey(id: string) {
  return `${SNAPSHOT_PREFIX}${id}`;
}

function saveSnapshot(id: string, doc: LoroDocType) {
  const bytes = doc.export({ mode: 'snapshot' });
  const b64 = btoa(String.fromCharCode(...bytes));
  localStorage.setItem(snapshotKey(id), b64);
}

function loadSnapshot(id: string): LoroDocType | null {
  const b64 = localStorage.getItem(snapshotKey(id));
  if (!b64) return null;
  const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
  const doc = new LoroDoc();
  doc.import(bytes);
  return doc;
}

function readMeta(doc: LoroDocType): { title: string; type: 'markdown' | 'list'; updated_at: string } {
  const meta = doc.getMap('meta');
  const title = (meta.get('title') as string) ?? 'Untitled';
  const type = (meta.get('type') as string) ?? 'markdown';
  const updated_at = (meta.get('updated_at') as string) ?? new Date().toISOString();
  return { title, type: type as 'markdown' | 'list', updated_at };
}

function readContent(doc: LoroDocType, type: string): { content: string; items?: string[] } {
  if (type === 'list') {
    const list = doc.getList('items');
    const items = list.toJSON().map((v: unknown) => String(v));
    return { content: '', items };
  }
  const text = doc.getText('content');
  return { content: text.toString(), items: undefined };
}

function docToNote(doc: LoroDocType, id: string): Note {
  const meta = readMeta(doc);
  const { content, items } = readContent(doc, meta.type);
  return { id, ...meta, content, items };
}

function createDoc(type: 'markdown' | 'list', title?: string): LoroDocType {
  const doc = new LoroDoc();
  const meta = doc.getMap('meta');
  meta.set('title', title ?? (type === 'markdown' ? 'Untitled' : 'Untitled List'));
  meta.set('type', type);
  meta.set('updated_at', new Date().toISOString());

  if (type === 'markdown') {
    const text = doc.getText('content');
    text.insert(0, '# Hello\n\nStart writing...');
  } else {
    const list = doc.getList('items');
    list.insert(0, 'First item');
  }

  return doc;
}

// --- Public API ---

export function getNotes(): Note[] {
  return getIndex();
}

export function createNote(type: 'markdown' | 'list'): Note {
  const id = crypto.randomUUID();
  const doc = createDoc(type);
  saveSnapshot(id, doc);
  const note = docToNote(doc, id);
  const index = getIndex();
  index.unshift(note);
  saveIndex(index);
  return note;
}

export function updateNote(id: string, updates: Partial<Note>): Note | null {
  const doc = loadSnapshot(id);
  if (!doc) return null;

  const meta = doc.getMap('meta');
  if (updates.title !== undefined) meta.set('title', updates.title);
  if (updates.updated_at !== undefined) meta.set('updates', updates.updated_at);
  meta.set('updated_at', new Date().toISOString());

  const currentType = (meta.get('type') as string) ?? 'markdown';

  if (currentType === 'markdown' && updates.content !== undefined) {
    const text = doc.getText('content');
    text.update(updates.content);
  } else if (currentType === 'list' && updates.items !== undefined) {
    const list = doc.getList('items');
    // Clear and re-insert
    const len = list.length;
    if (len > 0) list.delete(0, len);
    for (const item of updates.items) {
      list.insert(list.length, item);
    }
  }

  saveSnapshot(id, doc);
  const note = docToNote(doc, id);

  const index = getIndex();
  const idx = index.findIndex((n) => n.id === id);
  if (idx !== -1) index[idx] = note;
  saveIndex(index);

  return note;
}

export function deleteNote(id: string) {
  localStorage.removeItem(snapshotKey(id));
  const index = getIndex().filter((n) => n.id !== id);
  saveIndex(index);
}

export function importSnapshot(id: string, snapshot: Uint8Array): Note {
  const doc = new LoroDoc();
  doc.import(snapshot);
  saveSnapshot(id, doc);
  const note = docToNote(doc, id);

  const index = getIndex();
  const idx = index.findIndex((n) => n.id === id);
  if (idx !== -1) {
    index[idx] = note;
  } else {
    index.unshift(note);
  }
  saveIndex(index);
  return note;
}

export function exportSnapshot(id: string): Uint8Array | null {
  const doc = loadSnapshot(id);
  if (!doc) return null;
  return doc.export({ mode: 'snapshot' });
}

export function getDoc(id: string): LoroDocType | null {
  return loadSnapshot(id);
}

export function getNote(id: string): Note | null {
  const doc = loadSnapshot(id);
  if (!doc) return null;
  return docToNote(doc, id);
}

/**
 * Clear all local data. Used when logging out.
 * Also clears sync cursor so next login pulls everything fresh from server.
 */
export function clearAll() {
  const index = getIndex();
  for (const note of index) {
    localStorage.removeItem(snapshotKey(note.id));
  }
  localStorage.removeItem(INDEX_KEY);
  localStorage.removeItem('nnn-last-seq');
}
