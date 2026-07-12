// loro-init.ts
// Single entry point for Loro WASM + LoroDoc class.
//
// TS 6.0.3 can't resolve `export *` + `export type *` from loro-crdt/web types,
// so we use `import type` from the bundler entry (same types, TS resolves fine)
// and grab the runtime class via dynamic import from the web entry.
//
// IMPORTANT: LoroDoc constructor and init MUST come from the same entry
// (loro-crdt/web) because each entry has its own WASM instance.

import type { LoroDoc } from 'loro-crdt/bundler';

export type { LoroDoc };

let loroReady: {
  init: () => Promise<void>;
  LoroDoc: typeof LoroDoc;
} | null = null;

async function loadLoro() {
  if (loroReady) return loroReady;
  const mod = await import('loro-crdt/web');
  loroReady = {
    init: mod.default as unknown as () => Promise<void>,
    LoroDoc: (mod as Record<string, unknown>).LoroDoc as typeof LoroDoc,
  };
  return loroReady;
}

export async function ensureLoroReady(): Promise<void> {
  const loro = await loadLoro();
  await loro.init();
}

export function getLoroDocClass(): typeof LoroDoc {
  if (!loroReady) throw new Error('Loro not initialized — call ensureLoroReady() first');
  return loroReady.LoroDoc;
}
