import { create } from "zustand";
import { api, BindReport, PresentationBind, SimResult, SymbolBinding, TuneResult } from "@/lib/api";

interface ConfigSymbol {
  id: number;
  name: string;
  kind: string;
}

interface State {
  prompt: string;
  configText: string;
  sim: SimResult | null;
  tune: TuneResult | null;
  targetRtp: number;
  loading: "translate" | "simulate" | "tune" | null;
  error: string | null;

  // Art binding (the art↔math bridge)
  assetText: string;
  assets: string[]; // loaded asset file names
  symbolAsset: Record<number, string>; // symbol id -> chosen asset file name
  eventTypes: string[];
  bindReport: BindReport | null;

  setPrompt: (s: string) => void;
  setConfigText: (s: string) => void;
  setTargetRtp: (n: number) => void;
  doTranslate: () => Promise<void>;
  doSimulate: () => Promise<void>;
  doTune: () => Promise<void>;

  setAssetText: (s: string) => void;
  loadAssetNames: (names: string[]) => Promise<void>;
  setSymbolAsset: (id: number, name: string) => Promise<void>;
  clearAssets: () => void;
  doValidateBind: () => Promise<void>;
}

const msg = (e: unknown) => (e instanceof Error ? e.message : String(e));

const configSymbols = (configText: string): ConfigSymbol[] => {
  try {
    const c = JSON.parse(configText);
    return Array.isArray(c?.symbols) ? c.symbols : [];
  } catch {
    return [];
  }
};

const base = (f: string) => f.split(/[\\/]/).pop()!.toLowerCase();

// Map one asset file to the right renderable field on a symbol binding.
const assetToBinding = (id: number, f: string): SymbolBinding => {
  const lower = base(f);
  const sb: SymbolBinding = { symbol_id: id };
  if (lower.endsWith(".json") || lower.endsWith(".skel")) sb.skeleton = f;
  else if (/\.(png|jpg|jpeg|webp)$/.test(lower)) sb.spritesheet = { sheet: f };
  else if (lower.endsWith(".atlas")) sb.atlas = f; // companion only (not renderable alone)
  else sb.atlas_region = f;
  return sb;
};

// Auto-match: assign each unbound symbol the asset whose file name contains the symbol name.
// Prefer a Spine skeleton, else any renderable asset.
const autoMatch = (
  symbols: ConfigSymbol[],
  assets: string[],
  current: Record<number, string>
): Record<number, string> => {
  const next = { ...current };
  for (const sym of symbols) {
    if (next[sym.id]) continue;
    const key = sym.name.toLowerCase();
    const hits = assets.filter((a) => base(a).includes(key));
    if (hits.length === 0) continue;
    const skel = hits.find((a) => /\.(json|skel)$/.test(base(a)));
    next[sym.id] = skel ?? hits[0];
  }
  return next;
};

export const useStore = create<State>((set, get) => ({
  prompt: "Make a 5x3 slot, 20 paylines, 96% RTP, medium volatility, with wild and scatter",
  configText: "",
  sim: null,
  tune: null,
  targetRtp: 0.96,
  loading: null,
  error: null,

  assetText: "",
  assets: [],
  symbolAsset: {},
  eventTypes: [],
  bindReport: null,

  setPrompt: (s) => set({ prompt: s }),
  setConfigText: (s) => set({ configText: s }),
  setTargetRtp: (n) => set({ targetRtp: n }),

  doTranslate: async () => {
    set({ loading: "translate", error: null });
    try {
      const r = await api.translate(get().prompt);
      set({ configText: JSON.stringify(r.config, null, 2), sim: null, tune: null });
      if (r.targets?.rtp) set({ targetRtp: r.targets.rtp });
    } catch (e) {
      set({ error: "Translate failed: " + msg(e) });
    } finally {
      set({ loading: null });
    }
  },

  doSimulate: async () => {
    set({ loading: "simulate", error: null });
    try {
      const config = JSON.parse(get().configText);
      const r = await api.simulate(config, 500_000);
      set({ sim: r });
    } catch (e) {
      set({ error: "Simulate failed: " + msg(e) });
    } finally {
      set({ loading: null });
    }
  },

  doTune: async () => {
    set({ loading: "tune", error: null });
    try {
      const config = JSON.parse(get().configText);
      const r = await api.tune(config, get().targetRtp, 120);
      set({ tune: r, configText: JSON.stringify(r.tuned_config, null, 2), sim: null });
    } catch (e) {
      set({ error: "Tune failed: " + msg(e) });
    } finally {
      set({ loading: null });
    }
  },

  setAssetText: (s) => set({ assetText: s }),

  loadAssetNames: async (names) => {
    const clean = names.map((n) => n.trim()).filter(Boolean);
    if (clean.length === 0) return;
    const assets = Array.from(new Set([...get().assets, ...clean]));
    const symbolAsset = autoMatch(configSymbols(get().configText), assets, get().symbolAsset);
    set({ assets, symbolAsset });
    await get().doValidateBind();
  },

  setSymbolAsset: async (id, name) => {
    const symbolAsset = { ...get().symbolAsset };
    if (name) symbolAsset[id] = name;
    else delete symbolAsset[id];
    set({ symbolAsset });
    await get().doValidateBind();
  },

  clearAssets: () => set({ assets: [], symbolAsset: {}, bindReport: null, assetText: "" }),

  doValidateBind: async () => {
    let config: unknown;
    try {
      config = JSON.parse(get().configText);
    } catch {
      set({ error: "Translate or load a valid config before binding art." });
      return;
    }
    try {
      let eventTypes = get().eventTypes;
      if (eventTypes.length === 0) {
        eventTypes = (await api.eventTypes()).event_types;
        set({ eventTypes });
      }
      const { symbolAsset } = get();
      const symbols = configSymbols(get().configText)
        .filter((sym) => symbolAsset[sym.id])
        .map((sym) => assetToBinding(sym.id, symbolAsset[sym.id]));
      const binding: PresentationBind = {
        symbols,
        events: eventTypes.map((et) => ({ event_type: et, anim: et })),
      };
      const report = await api.validateBind(config, binding);
      set({ bindReport: report, error: null });
    } catch (e) {
      set({ error: "Bind check failed: " + msg(e) });
    }
  },
}));
