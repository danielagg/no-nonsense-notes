declare module '/wasm/no_nonsense_notes_wasm.js' {
  export class WasmStore {
    constructor();
    free(): void;
    createNote(note_type: string, folder_id?: string | null): any;
    getNote(id: string): any;
    updateNote(id: string, content: string): any;
    listAddItem(id: string, item: string): any;
    listRemoveItem(id: string, item: string): any;
    softDelete(id: string): void;
    listNotes(folder_id?: string | null): any;
    searchNotes(query: string): any;
  }
  export default function init(module_or_path?: string | URL): Promise<any>;
}
