import { create } from "zustand";
import { api, SimResult, TuneResult } from "@/lib/api";

interface State {
  prompt: string;
  configText: string;
  sim: SimResult | null;
  tune: TuneResult | null;
  targetRtp: number;
  loading: "translate" | "simulate" | "tune" | null;
  error: string | null;

  setPrompt: (s: string) => void;
  setConfigText: (s: string) => void;
  setTargetRtp: (n: number) => void;
  doTranslate: () => Promise<void>;
  doSimulate: () => Promise<void>;
  doTune: () => Promise<void>;
}

const msg = (e: unknown) => (e instanceof Error ? e.message : String(e));

export const useStore = create<State>((set, get) => ({
  prompt: "Make a 5x3 slot, 20 paylines, 96% RTP, medium volatility, with wild and scatter",
  configText: "",
  sim: null,
  tune: null,
  targetRtp: 0.96,
  loading: null,
  error: null,

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
}));
