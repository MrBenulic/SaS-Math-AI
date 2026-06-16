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

export default function BindingPanel() {
  const s = useStore();
  const symbols = symbolsOf(s.configText);
  const report = s.bindReport;

  const symStatus = (id: number): Status => {
    if (!s.symbolAsset[id]) return report?.missing_symbols.includes(id) ? "missing" : "idle";
    if (report?.unrenderable_symbols.includes(id)) return "warn";
    if (report?.missing_symbols.includes(id)) return "missing";
    return s.symbolAsset[id] ? "ok" : "idle";
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

          {/* Symbol coverage */}
          <div className="space-y-1">
            <div className="text-[10px] uppercase tracking-wide text-slate-500">Symbols → art</div>
            {symbols.map((sym) => (
              <div key={sym.id} className="flex items-center gap-2 text-xs" data-symbol-id={sym.id}>
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
        </>
      )}
    </div>
  );
}
