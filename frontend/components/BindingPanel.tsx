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

function Dot({ status }: { status: Status }) {
  const map: Record<Status, string> = {
    ok: "bg-emerald-500",
    warn: "bg-amber-500",
    missing: "bg-red-500",
    idle: "bg-slate-600",
  };
  return <span className={`inline-block h-2.5 w-2.5 rounded-full ${map[status]}`} />;
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
    return {
      symbols: b.symbols?.length ?? 0,
      events: b.events?.length ?? 0,
      hash: Boolean(b.targets_config_hash),
    };
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
    <div className="space-y-2">
      <div className="text-xs font-semibold uppercase tracking-wide text-slate-400">4 · Art binding</div>

      {symbols.length === 0 ? (
        <p className="text-xs text-slate-500">
          Translate a game first — then load your Spine / atlas / spritesheet files to bind art to the
          math.
        </p>
      ) : (
        <>
          {/* Load assets: drag-drop or paste file names */}
          <div
            onDrop={onDrop}
            onDragOver={(e) => e.preventDefault()}
            className="rounded-lg border border-dashed border-slate-700 bg-slate-900/40 p-3 text-center text-xs text-slate-400"
          >
            Drop your Spine / atlas / spritesheet files here
            <label className="mx-1 cursor-pointer font-semibold text-indigo-400 hover:text-indigo-300">
              {" "}or browse
              <input type="file" multiple className="hidden" onChange={onPick} />
            </label>
          </div>
          <textarea
            id="asset-names"
            className="h-16 w-full resize-none rounded-lg border border-slate-700 bg-slate-900 p-2 text-xs outline-none focus:border-emerald-600"
            value={s.assetText}
            onChange={(e) => s.setAssetText(e.target.value)}
            placeholder="…or paste file names, one per line (e.g. wild.json, scatter.json, ten.png)"
          />
          <div className="flex gap-2">
            <button
              id="load-assets-btn"
              onClick={loadFromText}
              className="flex-1 rounded-lg bg-slate-700 py-1.5 text-xs font-semibold hover:bg-slate-600"
            >
              Load file names
            </button>
            <button
              id="clear-assets-btn"
              onClick={s.clearAssets}
              className="rounded-lg border border-slate-700 px-3 py-1.5 text-xs text-slate-400 hover:bg-slate-800"
            >
              Clear
            </button>
          </div>

          {/* Coverage banner */}
          {report && (
            <div
              id="bind-banner"
              className={`rounded-lg px-3 py-2 text-sm font-semibold ${
                report.valid
                  ? "border border-emerald-800 bg-emerald-950/50 text-emerald-300"
                  : "border border-red-800 bg-red-950/50 text-red-300"
              }`}
            >
              {report.valid
                ? "✓ Complete — every symbol & event is bound. Art + certified math = one package."
                : `✗ ${gaps} coverage gap${gaps === 1 ? "" : "s"} — see the red items below.`}
            </div>
          )}

          {/* Symbol coverage + per-symbol animations */}
          <div className="space-y-1">
            <div className="text-[10px] uppercase tracking-wide text-slate-500">Symbols → art</div>
            {symbols.map((sym) => (
              <div key={sym.id} className="rounded-md bg-slate-900/30 px-1.5 py-1" data-symbol-id={sym.id}>
                <div className="flex items-center gap-2 text-xs">
                  <Dot status={symStatus(sym.id)} />
                  <span className="w-20 truncate text-slate-300" title={sym.name}>
                    {sym.name}
                    <span className="text-slate-600"> ({sym.kind})</span>
                  </span>
                  <select
                    data-symbol-select={sym.id}
                    value={s.symbolAsset[sym.id] ?? ""}
                    onChange={(e) => s.setSymbolAsset(sym.id, e.target.value)}
                    className="min-w-0 flex-1 rounded-md border border-slate-700 bg-slate-900 px-1.5 py-1 text-xs text-slate-200 outline-none focus:border-emerald-600"
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
                  <summary className="cursor-pointer text-[10px] text-slate-500 hover:text-slate-300">
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
                        className="w-full rounded border border-slate-800 bg-slate-900 px-1.5 py-0.5 text-[11px] text-slate-200 outline-none focus:border-emerald-700"
                      />
                    ))}
                  </div>
                </details>
              </div>
            ))}
          </div>

          {/* Event coverage */}
          {s.eventTypes.length > 0 && (
            <div className="space-y-1">
              <div className="text-[10px] uppercase tracking-wide text-slate-500">Events → animation</div>
              <div className="flex flex-wrap gap-1.5">
                {s.eventTypes.map((et) => {
                  const ok = !report?.missing_events.includes(et);
                  return (
                    <span
                      key={et}
                      className={`flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] ${
                        ok ? "bg-emerald-950/50 text-emerald-300" : "bg-red-950/50 text-red-300"
                      }`}
                    >
                      <Dot status={ok ? "ok" : "missing"} />
                      {et}
                    </span>
                  );
                })}
              </div>
            </div>
          )}

          {/* Export */}
          <button
            id="export-bind-btn"
            onClick={s.doExportBind}
            className="w-full rounded-lg bg-indigo-600 py-2 text-sm font-semibold hover:bg-indigo-500"
          >
            Export presentation.bind.json
          </button>
          {counts && (
            <details className="rounded-lg border border-slate-800 bg-slate-900/40 px-3 py-2">
              <summary id="export-summary" className="cursor-pointer text-xs text-slate-400">
                Exported ✓ — {counts.symbols} symbol bindings · {counts.events} events ·{" "}
                {counts.hash ? "fingerprint pinned" : "no fingerprint"}
              </summary>
              <pre
                id="bind-export-preview"
                className="mt-2 max-h-48 overflow-auto whitespace-pre-wrap break-all text-[10px] text-slate-300"
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
