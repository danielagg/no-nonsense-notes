import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockStore = {
  createNote: vi.fn(),
  updateNote: vi.fn(),
  updateList: vi.fn(),
  listNotes: vi.fn(),
  searchNotes: vi.fn(),
  softDelete: vi.fn(),
  listAddItem: vi.fn(),
  listRemoveItem: vi.fn(),
  applyRemoteUpdate: vi.fn(),
  applyRemoteDelete: vi.fn(),
  getSyncCursor: vi.fn(() => BigInt(0)),
  setSyncCursor: vi.fn(),
  getDeviceId: vi.fn(() => 'test-device-id'),
  exportNoteBlob: vi.fn(() => new Uint8Array()),
  free: vi.fn(),
};

vi.mock('../wasm-pkg/no_nonsense_notes_wasm.js', () => ({
  default: vi.fn(async () => {}),
  WasmStore: vi.fn(() => mockStore),
  encodePushFrame: vi.fn(() => new Uint8Array()),
  encodeDeleteFrame: vi.fn(() => new Uint8Array()),
  decodePushResponse: vi.fn(() => BigInt(0)),
  encodePullRequest: vi.fn((seq: bigint) => `pull:${seq}`),
  decodePullResponse: vi.fn(() => ({ currentSeq: 0, entries: [] })),
}));

import {
  getNotes,
  createNote,
  updateMarkdownNote,
  updateListNote,
  deleteNote,
  searchNotes,
} from '../api';
import { setActiveAccount } from '../wasm';
import { loadPendingPushes, registerPush } from '../sync-manager';

beforeEach(() => {
  vi.clearAllMocks();
  const accountId = `test-account-${crypto.randomUUID()}`;
  localStorage.setItem('nnn-account', accountId);
  setActiveAccount(accountId);
  registerPush(null);
  loadPendingPushes(accountId);
});

describe('api routing', () => {
  it('createNote calls wasm createNote with correct type', async () => {
    mockStore.createNote.mockReturnValue({ id: '1', noteType: 'markdown', title: 'Untitled', contentPlaintext: '', updatedAt: '2025-01-01T00:00:00Z' });
    await createNote('markdown');
    expect(mockStore.createNote).toHaveBeenCalledWith('markdown', null);
  });

  it('updateMarkdownNote calls wasm updateNote (not updateList)', async () => {
    mockStore.updateNote.mockReturnValue({ id: '1', noteType: 'markdown', title: 'Test', contentPlaintext: 'hello', updatedAt: '2025-01-01T00:00:00Z' });
    await updateMarkdownNote('1', 'hello', 'Renamed note');
    expect(mockStore.updateNote).toHaveBeenCalledWith('1', 'hello', 'Renamed note');
    expect(mockStore.updateList).not.toHaveBeenCalled();
  });

  it('updateListNote calls wasm updateList (not updateNote)', async () => {
    mockStore.updateList.mockReturnValue({ id: '1', noteType: 'list', title: 'Shopping', contentPlaintext: 'milk\neggs', updatedAt: '2025-01-01T00:00:00Z' });
    await updateListNote('1', ['milk', 'eggs'], 'Shopping');
    expect(mockStore.updateList).toHaveBeenCalledWith('1', JSON.stringify(['milk', 'eggs']), 'Shopping');
    expect(mockStore.updateNote).not.toHaveBeenCalled();
  });

  it('deleteNote soft-deletes locally and queues a tombstone', async () => {
    mockStore.softDelete.mockReturnValue(undefined);
    const push = vi.fn(async () => {});
    registerPush(push);
    await deleteNote('1');
    expect(mockStore.softDelete).toHaveBeenCalledWith('1');
    expect(push).toHaveBeenCalledWith('1', 'delete');
  });

  it('searchNotes calls wasm searchNotes', async () => {
    mockStore.searchNotes.mockReturnValue([]);
    await searchNotes('test');
    expect(mockStore.searchNotes).toHaveBeenCalledWith('test');
  });

  it('getNotes calls wasm listNotes with null folder', async () => {
    mockStore.listNotes.mockReturnValue([]);
    await getNotes();
    expect(mockStore.listNotes).toHaveBeenCalledWith(null);
  });
});

describe('wasmToNote mapping', () => {
  it('maps markdown note correctly', async () => {
    mockStore.listNotes.mockReturnValue([{
      id: 'abc',
      folderId: null,
      noteType: 'markdown',
      title: 'My Note',
      contentPlaintext: '# Hello',
      contentLoroBlob: new Uint8Array(),
      contentHash: new Uint8Array(),
      createdAt: '2025-01-01T00:00:00Z',
      updatedAt: '2025-01-01T00:00:00Z',
      isDeleted: false,
      deletedAt: null,
      sortOrder: 0,
    }]);
    const notes = await getNotes();
    expect(notes).toHaveLength(1);
    expect(notes[0].type).toBe('markdown');
    expect(notes[0].items).toBeUndefined();
    expect(notes[0].content).toBe('# Hello');
  });

  it('maps list note with items array', async () => {
    mockStore.listNotes.mockReturnValue([{
      id: 'abc',
      folderId: null,
      noteType: 'list',
      title: 'milk',
      contentPlaintext: 'milk\neggs',
      contentLoroBlob: new Uint8Array(),
      contentHash: new Uint8Array(),
      createdAt: '2025-01-01T00:00:00Z',
      updatedAt: '2025-01-01T00:00:00Z',
      isDeleted: false,
      deletedAt: null,
      sortOrder: 0,
    }]);
    const notes = await getNotes();
    expect(notes[0].type).toBe('list');
    expect(notes[0].items).toEqual(['milk', 'eggs']);
  });
});
