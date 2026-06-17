"use client";

import { useStore } from "@/store/useStore";

interface Sym {
  id: number;
  name: string;
  kind: string;
}

function symbolsOf(configText: string): Sym[] {
  try {
    const c = JSON.parse(configText);
    return Array.isArray(c?.symbols) ? c.symbols : [];
  } catch {
    return [];
  }
}

type Status = "ok" | "warn" | "missing" | "idle";
const STATUS_COLOR: Record<Status, string> = {
  ok: "hsl(var(--ok))",
  warn: "hsl(var(--gold))",
  missing: "hsl(var(--bad))",
  idle: "hsl(var(--ink-600))",
};

function Dot({ status }: { status: Status }) {
  return <span className="dot" style={{ color: STATUS_COLOR[status] }} />;
}

const ANIM_FIELDS: { key: string; label: string }[] = [
  { key: "idle_anim", label: "idle" },
  { key: "land_anim", label: "land" },
  { key: "win_anim", label: "win" },
  { key: "sound_cue", label: "sound" },
];

function exportCounts(json: string | null): { symbols: number; events: number; hash: boolean } | null {
  if (!json) return null;
  try {
    const b = JSON.parse(json);
    return { symbols: b.symbols?.length ?? 0, events: b.events?.length ?? 0, hash: Boolean(b.targets_config_hash) };
  } catch {
    return null;
  }
}

export default function BindingPanel() {
  const s = useStore();
  const symbols = symbolsOf(s.configText);
  const report = s.bindReport;

  const symStatus = (id: number): Status => {
    if (!s.symbolAsset[id]) return report?.missing_symbols.includes(id) ? "missing" : "idle";
    if (report?.unrenderable_symbols.includes(id)) return "warn";
    if (report?.missing_symbols.includes(id)) return "missing";
    return "ok";
  };

  const gaps = report
    ? report.missing_symbols.length +
      report.unrenderable_symbols.length +
      report.unknown_symbols.length +
      report.missing_events.length +
      report.unknown_events.length +
      (report.targets_config_hash_matches === false ? 1 : 0)
    : 0;

  const loadFromText = () => s.loadAssetNames(s.assetText.split(/[\n,]+/));
  const onDrop = (e: React.DragEvent) => {
    e.preventDefault();
    const names = Array.from(e.dataTransfer.files).map((f) => f.name);
    if (names.length) s.loadAssetNames(names);
  };
  const onPick = (e: React.ChangeEvent<HTMLInputElement>) => {
    const names = Array.from(e.target.files ?? []).map((f) => f.name);
    if (names.length) s.loadAssetNames(names);
  };

  const counts = exportCounts(s.exportPreview);

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2">
        <span className="step-num">4</span>
        <span className="step-label">Art binding</span>
      </div>

      {symbols.length === 0 ? (
        <p className="text-xs text-[hsl(var(--faint))]">
          Translate a game first — then load your Spine / atlas / spritesheet files to bind art to the math.
        </p>
      ) : (
        <>
          <div
            onDrop={onDrop}
            onDragOver={(e) => e.preventDefault()}
            className="rounded-xl border border-dashed border-[hsl(var(--line))] bg-[hsl(var(--ink-950)/0.4)] p-3.5 text-center text-xs text-[hsl(var(--muted))]"
          >
            Drop your Spine / atlas / spritesheet files here
            <label className="mx-1 cursor-pointer font-semibold text-[hsl(var(--brand))] hover:opacity-80">
              {" "}or browse
              <input type="file" multiple className="hidden" onChange={onPick} />
            </label>
          </div>
          <textarea
            id="asset-names"
            className="field h-16 w-full resize-none p-2 text-xs"
            value={s.assetText}
            onChange={(e) => s.setAssetText(e.target.value)}
            placeholder="…or paste file names, one per line (e.g. wild.json, scatter.json, ten.png)"
          />
          <div className="flex gap-2">
            <button id="load-assets-btn" onClick={loadFromText} className="btn btn-soft flex-1 py-1.5 text-xs">
              Load file names
            </button>
            <button id="clear-assets-btn" onClick={s.clearAssets} className="btn btn-ghost px-3 py-1.5 text-xs">
              Clear
            </button>
          </div>

          {report && (
            <div
              id="bind-banner"
              className="rounded-xl px-3 py-2 text-sm font-semibold"
              style={
                report.valid
                  ? { border: "1px solid hsl(var(--ok)/0.4)", background: "hsl(var(--ok)/0.1)", color: "hsl(var(--ok))" }
                  : { border: "1px solid hsl(var(--bad)/0.4)", background: "hsl(var(--bad)/0.1)", color: "hsl(var(--bad))" }
              }
            >
              {report.valid
                ? "✓ Complete — every symbol & event is bound. Art + certified math = one package."
                : `✗ ${gaps} coverage gap${gaps === 1 ? "" : "s"} — see the red items below.`}
            </div>
          )}

          <div className="space-y-1.5">
            <div className="stat-k">Symbols → art</div>
            {symbols.map((sym) => (
              <div key={sym.id} className="rounded-lg bg-[hsl(var(--ink-950)/0.35)] px-2 py-1.5" data-symbol-id={sym.id}>
                <div className="flex items-center gap-2 text-xs">
                  <Dot status={symStatus(sym.id)} />
                  <span className="w-20 truncate text-[hsl(var(--text))]" title={sym.name}>
                    {sym.name}
                    <span className="text-[hsl(var(--faint))]"> ({sym.kind})</span>
                  </span>
                  <select
                    data-symbol-select={sym.id}
                    value={s.symbolAsset[sym.id] ?? ""}
                    onChange={(e) => s.setSymbolAsset(sym.id, e.target.value)}
                    className="field min-w-0 flex-1 px-1.5 py-1 text-xs"
                  >
                    <option value="">— no art —</option>
                    {s.assets.map((a) => (
                      <option key={a} value={a}>
                        {a}
                      </option>
                    ))}
                  </select>
                </div>
                <details className="mt-1 pl-5">
                  <summary className="cursor-pointer text-[10px] text-[hsl(var(--faint))] hover:text-[hsl(var(--muted))]">
                    animations &amp; sound
                  </summary>
                  <div className="mt-1 grid grid-cols-2 gap-1">
                    {ANIM_FIELDS.map((f) => (
                      <input
                        key={f.key}
                        id={`anim-${sym.id}-${f.key}`}
                        value={s.symbolAnims[sym.id]?.[f.key] ?? ""}
                        onChange={(e) => s.setSymbolAnim(sym.id, f.key, e.target.value)}
                        placeholder={f.label}
                        className="field w-full px-1.5 py-0.5 text-[11px]"
                      />
                    ))}
                  </div>
                </details>
              </div>
            ))}
          </div>

          {s.eventTypes.length > 0 && (
            <div className="space-y-1.5">
              <div className="stat-k">Events → animation</div>
              <div className="flex flex-wrap gap-1.5">
                {s.eventTypes.map((et) => {
                  const ok = !report?.missing_events.includes(et);
                  return (
                    <span
                      key={et}
                      className="flex items-center gap-1.5 rounded-md px-2 py-0.5 text-[11px]"
                      style={
                        ok
                          ? { background: "hsl(var(--ok)/0.12)", color: "hsl(var(--ok))" }
                          : { background: "hsl(var(--bad)/0.12)", color: "hsl(var(--bad))" }
                      }
                    >
                      <Dot status={ok ? "ok" : "missing"} />
                      {et}
                    </span>
                  );
                })}
              </div>
            </div>
          )}

          <button id="export-bind-btn" onClick={s.doExportBind} className="btn btn-primary w-full py-2 text-sm">
            Export presentation.bind.json
          </button>
          {counts && (
            <details className="rounded-xl border border-[hsl(var(--line))] bg-[hsl(var(--ink-950)/0.4)] px-3 py-2">
              <summary id="export-summary" className="cursor-pointer text-xs text-[hsl(var(--muted))]">
                Exported ✓ — {counts.symbols} symbol bindings · {counts.events} events ·{" "}
                {counts.hash ? "fingerprint pinned" : "no fingerprint"}
              </summary>
              <pre
                id="bind-export-preview"
                className="mt-2 max-h-48 overflow-auto whitespace-pre-wrap break-all text-[10px] text-[hsl(var(--muted))]"
              >
                {s.exportPreview}
              </pre>
            </details>
          )}
        </>
      )}
    </div>
  );
}
