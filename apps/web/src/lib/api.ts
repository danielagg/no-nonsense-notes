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

// Auth API
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

// Local-only note storage (localStorage) for this throwaway sandbox
const NOTES_KEY = 'nnn-notes';

export function getNotes(): Note[] {
  const raw = localStorage.getItem(NOTES_KEY);
  return raw ? JSON.parse(raw) : [];
}

export function saveNotes(notes: Note[]) {
  localStorage.setItem(NOTES_KEY, JSON.stringify(notes));
}

export function createNote(type: 'markdown' | 'list'): Note {
  const note: Note = {
    id: crypto.randomUUID(),
    title: type === 'markdown' ? 'Untitled' : 'Untitled List',
    type,
    content: type === 'markdown' ? '# Hello\n\nStart writing...' : '',
    items: type === 'list' ? ['First item'] : undefined,
    updated_at: new Date().toISOString(),
  };
  const notes = getNotes();
  notes.unshift(note);
  saveNotes(notes);
  return note;
}

export function updateNote(id: string, updates: Partial<Note>): Note | null {
  const notes = getNotes();
  const idx = notes.findIndex((n) => n.id === id);
  if (idx === -1) return null;
  notes[idx] = { ...notes[idx], ...updates, updated_at: new Date().toISOString() };
  saveNotes(notes);
  return notes[idx];
}

export function deleteNote(id: string) {
  const notes = getNotes().filter((n) => n.id !== id);
  saveNotes(notes);
}
