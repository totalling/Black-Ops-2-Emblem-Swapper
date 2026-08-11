import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { readFile, writeFile } from "@tauri-apps/plugin-fs";
import type {
  DiagnosticCheck,
  EmblemInfo,
  EmblemRef,
  NetworkInfo,
  StatusResponse,
  PublicMode,
} from "./types";

const BIN_FILTER = [{ name: "Black Ops 2 Emblem", extensions: ["bin"] }];
const BACKUP_FILTER = [{ name: "Emblem Library Backup", extensions: ["zip"] }];

function sanitizeFilename(name: string): string {
  return name.replace(/[\\/:*?"<>|]+/g, "_").trim();
}

// The save/open dialogs can hand back an Android `content://` URI instead of
// a real filesystem path, which Rust's std::fs can't read or write directly.
// Bytes are moved across the Tauri IPC boundary as base64 and the actual
// file I/O against the picked location goes through the fs plugin, which
// knows how to talk to Android's Storage Access Framework.
function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunkSize = 0x8000;
  for (let i = 0; i < bytes.length; i += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunkSize));
  }
  return btoa(binary);
}

function base64ToBytes(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

export const api = {
  getStatus: () => invoke<StatusResponse>("get_status"),
  setMode: (mode: PublicMode) => invoke<void>("set_mode", { mode }),
  listEmblems: () => invoke<EmblemInfo[]>("list_emblems"),
  renderEmblemPng: (group: string, slot: number) =>
    invoke<string>("render_emblem_png", { group, slot }),
  setEmblemLabel: (group: string, slot: number, label: string) =>
    invoke<void>("set_emblem_label", { group, slot, label }),
  selectEmblem: (group: string, slot: number) =>
    invoke<void>("select_emblem", { group, slot }),
  clearSelection: () => invoke<void>("clear_selection"),
  deleteEmblem: (group: string, slot: number) => invoke<void>("delete_emblem", { group, slot }),
  deleteEmblems: (items: EmblemRef[]) => invoke<void>("delete_emblems", { items }),
  getNetworkInfo: () => invoke<NetworkInfo>("get_network_info"),

  exportEmblem: async (emblem: EmblemInfo): Promise<boolean> => {
    const base = sanitizeFilename(emblem.label || `emblem-${emblem.group}-${emblem.slot}`);
    const destPath = await save({
      defaultPath: `${base}.bin`,
      filters: BIN_FILTER,
    });
    if (!destPath) return false;
    const data = await invoke<string>("export_emblem", { group: emblem.group, slot: emblem.slot });
    await writeFile(destPath, base64ToBytes(data));
    return true;
  },

  importEmblem: async (): Promise<EmblemInfo | null> => {
    const srcPath = await open({ multiple: false, filters: BIN_FILTER });
    if (!srcPath) return null;
    const bytes = await readFile(srcPath);
    return invoke<EmblemInfo>("import_emblem", { data: bytesToBase64(bytes), label: null });
  },

  findDuplicateEmblems: () => invoke<EmblemInfo[][]>("find_duplicate_emblems"),
  runConnectionDiagnostics: () => invoke<DiagnosticCheck[]>("run_connection_diagnostics"),

  backupLibrary: async (): Promise<number | null> => {
    const destPath = await save({
      defaultPath: `bo2-emblem-backup-${new Date().toISOString().slice(0, 10)}.zip`,
      filters: BACKUP_FILTER,
    });
    if (!destPath) return null;
    const result = await invoke<{ count: number; data: string }>("backup_library");
    await writeFile(destPath, base64ToBytes(result.data));
    return result.count;
  },

  restoreLibrary: async (): Promise<number | null> => {
    const srcPath = await open({ multiple: false, filters: BACKUP_FILTER });
    if (!srcPath) return null;
    const bytes = await readFile(srcPath);
    return invoke<number>("restore_library", { data: bytesToBase64(bytes) });
  },
};
