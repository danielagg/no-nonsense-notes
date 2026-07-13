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

import {
  getNotes as wasmGetNotes,
  createNote as wasmCreateNote,
  updateNote as wasmUpdateNote,
  updateList as wasmUpdateList,
  softDelete as wasmSoftDelete,
  searchNotes as wasmSearchNotes,
  type WasmNote,
} from './wasm';
import { pushNote } from './sync-manager';

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
  const result = wasmToNote(note);
  await pushNote(result.id, type);
  return result;
}

export async function updateMarkdownNote(id: string, content: string): Promise<Note> {
  const note = await wasmUpdateNote(id, content);
  const result = wasmToNote(note);
  await pushNote(id, 'markdown');
  return result;
}

export async function updateListNote(id: string, items: string[]): Promise<Note> {
  const note = await wasmUpdateList(id, items);
  const result = wasmToNote(note);
  await pushNote(id, 'list');
  return result;
}

export async function deleteNote(id: string): Promise<void> {
  await wasmSoftDelete(id);
}

export async function searchNotes(query: string): Promise<Note[]> {
  const notes = await wasmSearchNotes(query);
  return notes.map(wasmToNote);
}
