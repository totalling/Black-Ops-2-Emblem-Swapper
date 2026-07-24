export type PublicMode = "off" | "capture" | "inject";

export interface Selection {
  group: string;
  slot: number;
}

export interface StatusResponse {
  mode: PublicMode;
  selected: Selection | null;
}

export interface EmblemInfo {
  id: string;
  group: string;
  slot: number;
  label: string;
  captured_at: string;
}

export interface NetworkInfo {
  lan_ip: string | null;
  proxy_port: number;
}

export interface EmblemCapturedPayload {
  group: string;
  slot: number;
}

export interface EmblemRef {
  group: string;
  slot: number;
}
