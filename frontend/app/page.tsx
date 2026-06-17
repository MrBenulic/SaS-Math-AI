"use client";

import dynamic from "next/dynamic";
import { useEffect, useRef } from "react";
import gsap from "gsap";
import { useStore } from "@/store/useStore";
import BindingPanel from "@/components/BindingPanel";
import AnimatedNumber from "@/components/AnimatedNumber";

const Editor = dynamic(() => import("@monaco-editor/react"), { ssr: false });

function Stat({
  label,
  children,
  hero,
}: {
  label: string;
  children: React.ReactNode;
  hero?: boolean;
}) {
  return (
    <div className="stat">
      <div className="stat-k">{label}</div>
      <div className={`stat-v ${hero ? "hero" : ""}`}>{children}</div>
    </div>
  );
}

function StepHeader({ n, label }: { n: number; label: string }) {
  return (
    <div className="flex items-center gap-2">
      <span className="step-num">{n}</span>
      <span className="step-label">{label}</span>
    </div>
  );
}

export default function Home() {
  const s = useStore();
  const busy = s.loading !== null;
  const colRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!colRef.current) return;
    const ctx = gsap.context(() => {
      gsap.from(".js-reveal", { y: 18, opacity: 0, duration: 0.5, stagger: 0.07, ease: "power2.out" });
    }, colRef);
    return () => ctx.revert();
  }, []);

  return (
    <main className="flex h-screen flex-col">
      {/* Header */}
      <header className="flex items-center justify-between px-6 py-3.5">
        <div className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-xl border border-[hsl(var(--brand)/0.35)] bg-[hsl(var(--brand)/0.12)] text-[hsl(var(--brand))]">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round">
              <path d="M4 18 L9 8 L13 14 L20 5" />
            </svg>
          </div>
          <div>
            <h1 className="text-[15px] font-semibold tracking-tight">
              SaS<span className="text-[hsl(var(--gold))]">·</span>Math{" "}
              <span className="font-normal text-[hsl(var(--muted))]">Slot Studio</span>
            </h1>
            <p className="text-[11px] text-[hsl(var(--faint))]">
              prompt → JSON → simulate → tune → bind → publish · exact math by the Rust engine
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2 rounded-full border border-[hsl(var(--line))] bg-[hsl(var(--ink-900)/0.6)] px-3 py-1.5">
          <span
            className="dot"
            style={{ color: busy ? "hsl(var(--gold))" : "hsl(var(--ok))" }}
          />
          <span className="text-[11px] font-medium text-[hsl(var(--muted))]">
            {busy ? `${s.loading}…` : "ready"}
          </span>
        </div>
      </header>

      <div className="grid min-h-0 flex-1 grid-cols-1 gap-4 px-4 pb-4 lg:grid-cols-[440px_1fr]">
        {/* Controls */}
        <section ref={colRef} className="min-h-0 space-y-3.5 overflow-y-auto pr-1 pb-4">
          {s.error && (
            <div className="rounded-xl border border-[hsl(var(--bad)/0.4)] bg-[hsl(var(--bad)/0.1)] px-3 py-2 text-sm text-[hsl(var(--bad))]">
              {s.error}
            </div>
          )}

          {/* 1 · Describe */}
          <div className="js-reveal panel space-y-3 p-4">
            <StepHeader n={1} label="Describe" />
            <textarea
              className="field h-24 w-full resize-none p-3 text-sm"
              value={s.prompt}
              onChange={(e) => s.setPrompt(e.target.value)}
              placeholder="e.g. 5×3 slot, 20 paylines, 96% RTP, high volatility, with wild and scatter"
            />
            <label className="flex items-center gap-2 text-xs text-[hsl(var(--muted))]">
              <input
                id="use-ai"
                type="checkbox"
                checked={s.useAI}
                onChange={(e) => s.setUseAI(e.target.checked)}
                className="h-3.5 w-3.5 accent-[hsl(var(--brand))]"
              />
              <span className="text-[hsl(var(--gold))]">✨</span> Use AI (Groq){" "}
              <span className="text-[hsl(var(--faint))]">· off = rule-based parser</span>
            </label>
            <button onClick={s.doTranslate} disabled={busy} className="btn btn-primary w-full py-2.5 text-sm">
              {s.useAI ? "Translate with AI → config" : "Translate → config"}
            </button>
          </div>

          {/* 2 · Simulate */}
          <div className="js-reveal panel space-y-3 p-4">
            <StepHeader n={2} label="Simulate" />
            <button
              onClick={s.doSimulate}
              disabled={busy || !s.configText}
              className="btn btn-soft w-full py-2.5 text-sm"
            >
              Simulate · exact + Monte Carlo
            </button>
            {s.sim && (
              <div className="grid grid-cols-2 gap-2">
                <Stat label="Exact RTP" hero>
                  <AnimatedNumber value={s.sim.exact_rtp} format="pct" />
                </Stat>
                <Stat label="MC RTP">
                  <AnimatedNumber value={s.sim.mc_rtp} format="pct" />
                </Stat>
                <Stat label="Hit frequency">
                  <AnimatedNumber value={s.sim.hit_frequency} format="pct" />
                </Stat>
                <Stat label="Volatility σ">
                  <AnimatedNumber value={s.sim.volatility} format="num" />
                </Stat>
                <Stat label="Max win">
                  <AnimatedNumber value={s.sim.max_win_x} format="x" decimals={1} />
                </Stat>
                <Stat label="Throughput">
                  <AnimatedNumber value={s.sim.throughput_mps} format="mps" decimals={1} />
                </Stat>
              </div>
            )}
          </div>

          {/* 3 · Tune */}
          <div className="js-reveal panel space-y-3 p-4">
            <StepHeader n={3} label="Tune" />
            <label className="flex items-center gap-2 text-sm text-[hsl(var(--muted))]">
              Target RTP
              <input
                type="number"
                step="0.01"
                min="0.5"
                max="1.5"
                value={s.targetRtp}
                onChange={(e) => s.setTargetRtp(parseFloat(e.target.value) || 0.96)}
                className="field w-24 px-2 py-1 text-sm"
              />
              <span className="text-[hsl(var(--faint))]">({(s.targetRtp * 100).toFixed(2)}%)</span>
            </label>
            <button
              onClick={s.doTune}
              disabled={busy || !s.configText}
              className="btn btn-primary w-full py-2.5 text-sm"
            >
              Auto-tune to target
            </button>
            {s.tune && (
              <div className="space-y-2">
                <div className="grid grid-cols-2 gap-2">
                  <Stat label="Before RTP">
                    <AnimatedNumber value={s.tune.before.exact_rtp} format="pct" />
                  </Stat>
                  <Stat label="After RTP" hero>
                    <AnimatedNumber value={s.tune.after.exact_rtp} format="pct" />
                  </Stat>
                  <Stat label="Hit (after)">
                    <AnimatedNumber value={s.tune.after.hit_frequency} format="pct" />
                  </Stat>
                  <Stat label="Volatility (after)">
                    <AnimatedNumber value={s.tune.after.volatility} format="num" />
                  </Stat>
                </div>
                <p className="text-[11px] text-[hsl(var(--faint))]">
                  realize-delta {(s.tune.realize_delta * 100).toFixed(2)}% · {s.tune.evaluations} solver evals · config
                  is now the tuned weighted-reel game.
                </p>
              </div>
            )}
          </div>

          {/* 4 · Art binding */}
          <div className="js-reveal panel p-4">
            <BindingPanel />
          </div>

          {/* 5 · Publish */}
          <div className="js-reveal panel space-y-3 p-4">
            <StepHeader n={5} label="Publish" />
            <button
              id="publish-btn"
              onClick={s.doPublish}
              disabled={busy || !s.configText}
              className="btn btn-gold w-full py-2.5 text-sm"
            >
              Publish — fingerprint &amp; record
            </button>
            <button
              id="mathsheet-btn"
              onClick={s.doDownloadSheet}
              disabled={busy || !s.configText}
              className="btn btn-soft w-full py-2 text-sm"
            >
              ⬇ Download math sheet (.xlsx)
            </button>
            {s.published && (
              <div
                id="publish-result"
                className="space-y-1.5 rounded-xl border border-[hsl(var(--gold)/0.3)] bg-[hsl(var(--gold)/0.07)] px-3 py-2.5 text-xs"
              >
                <div className="flex items-center justify-between">
                  <span className="text-[hsl(var(--muted))]">Exact RTP</span>
                  <span className="font-semibold text-[hsl(var(--gold))]">
                    <AnimatedNumber value={s.published.results.exact_rtp} format="pct" />
                  </span>
                </div>
                <div className="truncate text-[hsl(var(--faint))]" title={s.published.config_hash}>
                  fingerprint <span className="font-mono text-[hsl(var(--muted))]">{s.published.config_hash.slice(0, 24)}…</span>
                </div>
                <div className="text-[hsl(var(--faint))]">
                  recorded · ledger holds {s.ledgerCount ?? "?"} game{s.ledgerCount === 1 ? "" : "s"}
                </div>
              </div>
            )}
          </div>
        </section>

        {/* Config editor */}
        <section className="panel min-h-0 overflow-hidden p-0">
          <div className="flex items-center justify-between border-b border-[hsl(var(--line)/0.8)] px-4 py-2.5">
            <span className="step-label">Game config · JSON</span>
            <span className="text-[10px] text-[hsl(var(--faint))]">single source of truth · TS · Pydantic · Rust serde</span>
          </div>
          {s.configText ? (
            <Editor
              height="calc(100% - 42px)"
              defaultLanguage="json"
              theme="vs-dark"
              value={s.configText}
              onChange={(v) => s.setConfigText(v ?? "")}
              options={{ minimap: { enabled: false }, fontSize: 12.5, scrollBeyondLastLine: false, padding: { top: 14 } }}
            />
          ) : (
            <div className="flex h-[calc(100%-42px)] items-center justify-center px-8 text-center text-sm text-[hsl(var(--faint))]">
              Describe a game and hit{" "}
              <span className="mx-1 font-semibold text-[hsl(var(--brand))]">Translate</span> to generate a validated
              config.
            </div>
          )}
        </section>
      </div>
    </main>
  );
}
