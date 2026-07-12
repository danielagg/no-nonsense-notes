const API_BASE = import.meta.env.VITE_API_URL ?? '';

export interface AuthResponse {
  token: string;
  account_id: string;
}

export type { Note } from './loro-store';
export { getNotes, createNote, updateNote, deleteNote } from './loro-store';

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
