import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { EmblemInfo, EmblemRef, NetworkInfo, StatusResponse, PublicMode } from "./types";

const BIN_FILTER = [{ name: "Black Ops 2 Emblem", extensions: ["bin"] }];

function sanitizeFilename(name: string): string {
  return name.replace(/[\\/:*?"<>|]+/g, "_").trim();
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
    await invoke<void>("export_emblem", { group: emblem.group, slot: emblem.slot, destPath });
    return true;
  },

  importEmblem: async (): Promise<EmblemInfo | null> => {
    const srcPath = await open({ multiple: false, filters: BIN_FILTER });
    if (!srcPath) return null;
    return invoke<EmblemInfo>("import_emblem", { srcPath, label: null });
  },
};
