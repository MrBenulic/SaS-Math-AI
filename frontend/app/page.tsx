"use client";

import dynamic from "next/dynamic";
import { useStore } from "@/store/useStore";
import BindingPanel from "@/components/BindingPanel";

const Editor = dynamic(() => import("@monaco-editor/react"), { ssr: false });

const pct = (x: number | null | undefined) => (x == null ? "—" : (x * 100).toFixed(2) + "%");
const num = (x: number | null | undefined, d = 2) => (x == null ? "—" : x.toFixed(d));

function Stat({ label, value, accent }: { label: string; value: string; accent?: boolean }) {
  return (
    <div className="rounded-lg bg-slate-800/60 px-3 py-2">
      <div className="text-[10px] uppercase tracking-wide text-slate-400">{label}</div>
      <div className={`text-lg font-semibold ${accent ? "text-emerald-400" : "text-slate-100"}`}>{value}</div>
    </div>
  );
}

export default function Home() {
  const s = useStore();
  const busy = s.loading !== null;

  return (
    <main className="flex h-screen flex-col bg-slate-950 text-slate-100">
      <header className="flex items-center justify-between border-b border-slate-800 px-6 py-3">
        <div>
          <h1 className="text-lg font-bold">
            SaS-Math AI <span className="font-normal text-slate-400">— Slot Studio</span>
          </h1>
          <p className="text-xs text-slate-500">prompt → JSON → simulate → tune · math by the Rust engine</p>
        </div>
        {s.loading && <span className="animate-pulse text-sm text-emerald-400">● {s.loading}…</span>}
      </header>

      <div className="grid min-h-0 flex-1 grid-cols-1 lg:grid-cols-[420px_1fr]">
        {/* Controls + results */}
        <section className="min-h-0 space-y-4 overflow-y-auto border-r border-slate-800 p-5">
          {s.error && (
            <div className="rounded-lg border border-red-800 bg-red-950/60 px-3 py-2 text-sm text-red-300">{s.error}</div>
          )}

          {/* 1. Describe */}
          <div className="space-y-2">
            <div className="text-xs font-semibold uppercase tracking-wide text-slate-400">1 · Describe</div>
            <textarea
              className="h-24 w-full resize-none rounded-lg border border-slate-700 bg-slate-900 p-3 text-sm outline-none focus:border-emerald-600"
              value={s.prompt}
              onChange={(e) => s.setPrompt(e.target.value)}
              placeholder="e.g. 5x3 slot, 20 paylines, 96% RTP, high volatility, with wild and scatter"
            />
            <button
              onClick={s.doTranslate}
              disabled={busy}
              className="w-full rounded-lg bg-indigo-600 py-2 text-sm font-semibold hover:bg-indigo-500 disabled:opacity-50"
            >
              Translate → config
            </button>
          </div>

          {/* 2. Simulate */}
          <div className="space-y-2">
            <div className="text-xs font-semibold uppercase tracking-wide text-slate-400">2 · Simulate</div>
            <button
              onClick={s.doSimulate}
              disabled={busy || !s.configText}
              className="w-full rounded-lg bg-slate-700 py-2 text-sm font-semibold hover:bg-slate-600 disabled:opacity-50"
            >
              Simulate (exact + Monte Carlo)
            </button>
            {s.sim && (
              <div className="grid grid-cols-2 gap-2">
                <Stat label="Exact RTP" value={pct(s.sim.exact_rtp)} accent />
                <Stat label="MC RTP" value={pct(s.sim.mc_rtp)} />
                <Stat label="Hit frequency" value={pct(s.sim.hit_frequency)} />
                <Stat label="Volatility σ" value={num(s.sim.volatility)} />
                <Stat label="Max win" value={num(s.sim.max_win_x, 1) + "×"} />
                <Stat label="Throughput" value={num(s.sim.throughput_mps, 1) + "M/s"} />
              </div>
            )}
          </div>

          {/* 3. Tune */}
          <div className="space-y-2">
            <div className="text-xs font-semibold uppercase tracking-wide text-slate-400">3 · Tune</div>
            <label className="flex items-center gap-2 text-sm text-slate-300">
              Target RTP
              <input
                type="number"
                step="0.01"
                min="0.5"
                max="1.5"
                value={s.targetRtp}
                onChange={(e) => s.setTargetRtp(parseFloat(e.target.value) || 0.96)}
                className="w-24 rounded-md border border-slate-700 bg-slate-900 px-2 py-1 outline-none focus:border-emerald-600"
              />
              <span className="text-slate-500">({pct(s.targetRtp)})</span>
            </label>
            <button
              onClick={s.doTune}
              disabled={busy || !s.configText}
              className="w-full rounded-lg bg-emerald-600 py-2 text-sm font-semibold hover:bg-emerald-500 disabled:opacity-50"
            >
              Auto-tune to target
            </button>
            {s.tune && (
              <div className="space-y-2">
                <div className="grid grid-cols-2 gap-2">
                  <Stat label="Before RTP" value={pct(s.tune.before.exact_rtp)} />
                  <Stat label="After RTP" value={pct(s.tune.after.exact_rtp)} accent />
                  <Stat label="Hit (after)" value={pct(s.tune.after.hit_frequency)} />
                  <Stat label="Volatility (after)" value={num(s.tune.after.volatility)} />
                </div>
                <p className="text-xs text-slate-500">
                  realize-delta {pct(s.tune.realize_delta)} · {s.tune.evaluations} solver evals · the config is now the
                  tuned weighted-reel game.
                </p>
              </div>
            )}
          </div>

          {/* 4. Art binding */}
          <BindingPanel />
        </section>

        {/* Config editor */}
        <section className="min-h-0">
          {s.configText ? (
            <Editor
              height="100%"
              defaultLanguage="json"
              theme="vs-dark"
              value={s.configText}
              onChange={(v) => s.setConfigText(v ?? "")}
              options={{ minimap: { enabled: false }, fontSize: 12, scrollBeyondLastLine: false }}
            />
          ) : (
            <div className="flex h-full items-center justify-center text-sm text-slate-600">
              Describe a game and hit <span className="mx-1 font-semibold text-indigo-400">Translate</span> to generate a
              validated config.
            </div>
          )}
        </section>
      </div>
    </main>
  );
}
