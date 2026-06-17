import { create } from "zustand";
import {
  api,
  BindReport,
  PresentationBind,
  PublishResult,
  SimResult,
  SymbolBinding,
  TuneResult,
} from "@/lib/api";

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
  useAI: boolean; // use the Groq LLM translator instead of the rule-based one
  published: PublishResult | null;
  ledgerCount: number | null;
  loading: "translate" | "simulate" | "tune" | "publish" | null;
  error: string | null;

  // Art binding (the art↔math bridge)
  assetText: string;
  assets: string[]; // loaded asset file names
  symbolAsset: Record<number, string>; // symbol id -> chosen asset file name
  symbolAnims: Record<number, Record<string, string>>; // symbol id -> {idle_anim, land_anim, win_anim, sound_cue}
  eventTypes: string[];
  bindReport: BindReport | null;
  exportPreview: string | null; // last exported presentation.bind.json (for preview + verification)

  setPrompt: (s: string) => void;
  setConfigText: (s: string) => void;
  setTargetRtp: (n: number) => void;
  setUseAI: (b: boolean) => void;
  doTranslate: () => Promise<void>;
  doSimulate: () => Promise<void>;
  doTune: () => Promise<void>;
  doPublish: () => Promise<void>;

  setAssetText: (s: string) => void;
  loadAssetNames: (names: string[]) => Promise<void>;
  setSymbolAsset: (id: number, name: string) => Promise<void>;
  setSymbolAnim: (id: number, field: string, value: string) => void;
  clearAssets: () => void;
  doValidateBind: () => Promise<void>;
  doExportBind: () => Promise<void>;
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
const stem = (f: string) => base(f).replace(/\.[^.]+$/, ""); // filename, lowercased, no extension

// Monotonic guard so a slow/stale validate-bind response can't overwrite a newer one.
let bindSeq = 0;

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
// Match a symbol to an asset by EXACT stem or a whole-word boundary — never a bare substring, so
// "H1" no longer wrongly grabs "H10"/"H11". Skeletons win ties.
const autoMatch = (
  symbols: ConfigSymbol[],
  assets: string[],
  current: Record<number, string>
): Record<number, string> => {
  const next = { ...current };
  for (const sym of symbols) {
    if (next[sym.id]) continue;
    const key = sym.name.toLowerCase();
    const esc = key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const wb = new RegExp(`(^|[^a-z0-9])${esc}([^a-z0-9]|$)`);
    const scored = assets
      .map((a) => {
        const st = stem(a);
        const score = st === key ? 3 : wb.test(st) ? 2 : 0;
        return { a, score, skel: /\.(json|skel)$/.test(base(a)) };
      })
      .filter((x) => x.score > 0)
      .sort((p, q) => q.score - p.score || Number(q.skel) - Number(p.skel));
    if (scored.length) next[sym.id] = scored[0].a;
  }
  return next;
};

// Assemble a full PresentationBind from the chosen assets + per-symbol animations/sounds.
// A symbol is included if it has an asset OR any animation/sound set (so anim-only symbols still
// surface as "unrenderable" in validation). Pays of all events are auto-named after the event.
const buildBinding = (
  configText: string,
  symbolAsset: Record<number, string>,
  symbolAnims: Record<number, Record<string, string>>,
  eventTypes: string[]
): PresentationBind => {
  const symbols = configSymbols(configText)
    .map((sym): SymbolBinding | null => {
      const asset = symbolAsset[sym.id];
      const anims = symbolAnims[sym.id] ?? {};
      const animEntries = Object.entries(anims).filter(([, v]) => v && v.trim());
      if (!asset && animEntries.length === 0) return null;
      const sb: SymbolBinding = asset ? assetToBinding(sym.id, asset) : { symbol_id: sym.id };
      for (const [k, v] of animEntries) (sb as unknown as Record<string, unknown>)[k] = v.trim();
      return sb;
    })
    .filter((x): x is SymbolBinding => x !== null);
  return {
    bind_version: "0.1.0",
    symbols,
    events: eventTypes.map((et) => ({ event_type: et, anim: et })),
  };
};

export const useStore = create<State>((set, get) => ({
  prompt: "Make a 5x3 slot, 20 paylines, 96% RTP, medium volatility, with wild and scatter",
  configText: "",
  sim: null,
  tune: null,
  targetRtp: 0.96,
  useAI: false,
  published: null,
  ledgerCount: null,
  loading: null,
  error: null,

  assetText: "",
  assets: [],
  symbolAsset: {},
  symbolAnims: {},
  eventTypes: [],
  bindReport: null,
  exportPreview: null,

  setPrompt: (s) => set({ prompt: s }),
  setConfigText: (s) => set({ configText: s }),
  setTargetRtp: (n) => set({ targetRtp: n }),
  setUseAI: (b) => set({ useAI: b }),

  doTranslate: async () => {
    set({ loading: "translate", error: null });
    try {
      const r = await api.translate(get().prompt, get().useAI ? "groq" : "rule");
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
      // Clamp the target before it reaches the solver (the input can hold out-of-range typed values).
      const target = Math.min(Math.max(get().targetRtp || 0.96, 0.01), 5);
      const r = await api.tune(config, target, 120);
      set({ tune: r, configText: JSON.stringify(r.tuned_config, null, 2), sim: null, published: null });
    } catch (e) {
      set({ error: "Tune failed: " + msg(e) });
    } finally {
      set({ loading: null });
    }
  },

  doPublish: async () => {
    set({ loading: "publish", error: null });
    try {
      const config = JSON.parse(get().configText);
      const r = await api.publish(config, 200_000, true);
      const led = await api.ledger();
      set({ published: r, ledgerCount: led.published.length });
    } catch (e) {
      set({ error: "Publish failed: " + msg(e) });
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

  setSymbolAnim: (id, field, value) =>
    set((st) => ({
      symbolAnims: { ...st.symbolAnims, [id]: { ...(st.symbolAnims[id] ?? {}), [field]: value } },
    })),

  clearAssets: () =>
    set({ assets: [], symbolAsset: {}, symbolAnims: {}, bindReport: null, exportPreview: null, assetText: "" }),

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
      const binding = buildBinding(get().configText, get().symbolAsset, get().symbolAnims, eventTypes);
      const seq = ++bindSeq;
      const report = await api.validateBind(config, binding);
      if (seq !== bindSeq) return; // a newer validate-bind superseded this response
      set({ bindReport: report, error: null });
    } catch (e) {
      set({ error: "Bind check failed: " + msg(e) });
    }
  },

  doExportBind: async () => {
    let config: unknown;
    try {
      config = JSON.parse(get().configText);
    } catch {
      set({ error: "Translate or load a valid config before exporting a binding." });
      return;
    }
    try {
      let eventTypes = get().eventTypes;
      if (eventTypes.length === 0) {
        eventTypes = (await api.eventTypes()).event_types;
        set({ eventTypes });
      }
      const binding = buildBinding(get().configText, get().symbolAsset, get().symbolAnims, eventTypes);
      // Pin the binding to the exact config it targets (the certification fingerprint).
      try {
        binding.targets_config_hash = (await api.configHash(config)).config_hash;
      } catch {
        /* hash is best-effort; export still works without it */
      }
      const json = JSON.stringify(binding, null, 2);
      set({ exportPreview: json, error: null });
      if (typeof document !== "undefined") {
        const blob = new Blob([json], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = "presentation.bind.json";
        document.body.appendChild(a);
        a.click();
        a.remove();
        URL.revokeObjectURL(url);
      }
    } catch (e) {
      set({ error: "Export failed: " + msg(e) });
    }
  },
}));
