// Typed client for the FastAPI ai-layer (M3). The browser calls these; the ai-layer calls the
// Rust engine. Override the base URL with NEXT_PUBLIC_API_URL.

const BASE = process.env.NEXT_PUBLIC_API_URL || "http://127.0.0.1:8000";

async function post<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(BASE + path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    let detail = `${res.status} ${res.statusText}`;
    try {
      const j = await res.json();
      if (j?.detail) detail = typeof j.detail === "string" ? j.detail : JSON.stringify(j.detail);
    } catch {
      /* keep status text */
    }
    throw new Error(detail);
  }
  return res.json() as Promise<T>;
}

export interface SimResult {
  exact_rtp: number;
  line_rtp: number;
  scatter_rtp: number;
  capped_rtp: number | null;
  capped_provenance: string;
  mc_rtp: number;
  mc_ci95: number;
  hit_frequency: number;
  volatility: number;
  max_win_x: number;
  spins: number;
  throughput_mps: number;
}

export interface Metrics {
  exact_rtp: number;
  mc_rtp: number;
  hit_frequency: number;
  volatility: number;
}

export interface TuneResult {
  before: Metrics;
  after: Metrics;
  tuned_config: unknown;
  target_rtp: number;
  solver_total_rtp: number;
  generations: number;
  evaluations: number;
  realize_delta: number;
}

export interface TranslateResult {
  config: unknown;
  targets: { rtp: number | null; volatility: string | null; hit_frequency: number | null } | null;
}

// ---- Art binding (the art↔math bridge) ----
export interface SymbolBinding {
  symbol_id: number;
  skeleton?: string;
  atlas?: string;
  atlas_region?: string;
  spritesheet?: { sheet: string; frames?: string[]; fps?: number };
  sound_cue?: string;
  idle_anim?: string;
  land_anim?: string;
  win_anim?: string;
}
export interface EventBinding {
  event_type: string;
  anim?: string;
  sound?: string;
  blocking?: boolean;
  min_duration_ms?: number;
}
export interface PresentationBind {
  bind_version?: string;
  targets_config_hash?: string | null;
  symbols: SymbolBinding[];
  events: EventBinding[];
}
export interface BindReport {
  valid: boolean;
  missing_symbols: number[];
  unknown_symbols: number[];
  unrenderable_symbols: number[];
  missing_events: string[];
  unknown_events: string[];
  targets_config_hash_matches: boolean | null;
  errors: string[];
}

export const api = {
  health: () => fetch(BASE + "/health").then((r) => r.json()),
  translate: (prompt: string) => post<TranslateResult>("/translate", { prompt }),
  simulate: (config: unknown, spins = 500_000) => post<SimResult>("/simulate", { config, spins }),
  tune: (config: unknown, target_rtp: number, generations = 120) =>
    post<TuneResult>("/tune", { config, target_rtp, generations }),
  eventTypes: () =>
    fetch(BASE + "/bind/event-types").then((r) => r.json()) as Promise<{ event_types: string[] }>,
  validateBind: (config: unknown, binding: PresentationBind) =>
    post<BindReport>("/validate-bind", { config, binding }),
  configHash: (config: unknown) => post<{ config_hash: string }>("/config-hash", config),
};
