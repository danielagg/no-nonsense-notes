const API_BASE = import.meta.env.VITE_API_URL ?? '';

export interface AuthResponse {
  token: string;
  account_id: string;
}

export interface Note {
  id: string;
  title: string;
  type: 'markdown' | 'list';
  content: string;
  items?: string[];
  updated_at: string;
}

// Auth API (server-backed)
export async function signup(email: string, password: string): Promise<AuthResponse> {
  const res = await fetch(`${API_BASE}/auth/signup`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password }),
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || `Signup failed: ${res.status}`);
  }
  return res.json();
}

export async function signin(email: string, password: string): Promise<AuthResponse> {
  const res = await fetch(`${API_BASE}/auth/signin`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password }),
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || `Signin failed: ${res.status}`);
  }
  return res.json();
}

// WASM-backed note storage
import {
  getNotes as wasmGetNotes,
  createNote as wasmCreateNote,
  updateNote as wasmUpdateNote,
  listAddItem as wasmListAddItem,
  listRemoveItem as wasmListRemoveItem,
  softDelete as wasmSoftDelete,
  searchNotes as wasmSearchNotes,
  type WasmNote,
} from './wasm';

function wasmToNote(w: WasmNote): Note {
  const isList = w.noteType === 'list';
  const items = isList
    ? w.contentPlaintext.split('\n').filter((s) => s.length > 0)
    : undefined;
  return {
    id: w.id,
    title: w.title,
    type: w.noteType as 'markdown' | 'list',
    content: w.contentPlaintext,
    items,
    updated_at: w.updatedAt,
  };
}

export async function getNotes(): Promise<Note[]> {
  const notes = await wasmGetNotes();
  return notes.map(wasmToNote);
}

export async function createNote(type: 'markdown' | 'list'): Promise<Note> {
  const note = await wasmCreateNote(type);
  return wasmToNote(note);
}

export async function updateNote(
  id: string,
  updates: Partial<Pick<Note, 'content' | 'items'>>,
): Promise<Note | null> {
  try {
    if (updates.items !== undefined) {
      const content = updates.items.join('\n');
      const note = await wasmUpdateNote(id, content);
      return wasmToNote(note);
    }
    if (updates.content !== undefined) {
      const note = await wasmUpdateNote(id, updates.content);
      return wasmToNote(note);
    }
    return null;
  } catch {
    return null;
  }
}

export async function deleteNote(id: string): Promise<void> {
  await wasmSoftDelete(id);
}

export async function searchNotes(query: string): Promise<Note[]> {
  const notes = await wasmSearchNotes(query);
  return notes.map(wasmToNote);
}
