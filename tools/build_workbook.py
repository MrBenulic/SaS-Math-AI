#!/usr/bin/env python3
"""Build SaS-Math-AI_Features_and_Metrics.xlsx (feature catalogue + metric targets).

Reproducible generator for the project tracking workbook. Re-run after updating the seed rows
below. Output is written to the project root. Requires: openpyxl.
"""
from pathlib import Path
from openpyxl import Workbook
from openpyxl.styles import Font, PatternFill, Alignment, Border, Side
from openpyxl.utils import get_column_letter

OUT = Path(__file__).resolve().parent.parent / "SaS-Math-AI_Features_and_Metrics.xlsx"

HEADER_FILL = PatternFill("solid", fgColor="1F2937")
HEADER_FONT = Font(bold=True, color="FFFFFF")
TITLE_FONT = Font(bold=True, size=13, color="1F2937")
WRAP = Alignment(vertical="top", wrap_text=True)
THIN = Side(style="thin", color="D1D5DB")
BORDER = Border(left=THIN, right=THIN, top=THIN, bottom=THIN)
STATUS_FILL = {
    "Done": PatternFill("solid", fgColor="DCFCE7"),
    "In progress": PatternFill("solid", fgColor="FEF9C3"),
    "Planned": PatternFill("solid", fgColor="F3F4F6"),
}

SHEETS = {
    "Features": {
        "title": "Feature & Lego-block catalogue",
        "headers": ["Module", "Feature / Lego-block", "Description", "Status", "Priority", "Milestone", "Notes"],
        "widths": [16, 26, 42, 13, 10, 11, 26],
        "rows": [
            ["Core engine", "Namespaced counter-RNG", "Philox/Threefry keyed on (subsystem, spin, draw)", "Planned", "P0", "M1", "ADR-003 keystone"],
            ["Core engine", "Integer / u128 accounting", "smallest-unit money; decimal only at boundary", "Planned", "P0", "M1", "ADR-002"],
            ["Core engine", "Exact enumeration RTP", "full reel-stop enumeration / convolution", "Planned", "P0", "M1", "ADR-001 cert source of truth"],
            ["Core engine", "Line-eval Lego blocks", "LTR / both-ways / ways-243, scatter & wild rules", "Planned", "P0", "M1", "parity-critical semantics"],
            ["Core engine", "Deterministic Monte Carlo", "Rayon; seed-replay identical to the unit", "Planned", "P0", "M1", "validates harness"],
            ["Core engine", "Streaming integer stats", "fixed histogram + per-line/symbol counters", "Planned", "P0", "M1", "ADR-012"],
            ["Core engine", "Max-win cap inside math", "min(win, cap) inside convolution & sim", "Planned", "P0", "M1", "ADR-005"],
            ["Core engine", "Minimal numeric math sheet", "CSV/JSON exact + MC output", "Planned", "P1", "M1", ""],
            ["Core engine", "CLI", "run exact + MC on a config", "Planned", "P1", "M1", ""],
            ["Solver", "CMA-ES + penalty constraints", "optimize against the exact engine", "Planned", "P0", "M2", "ADR-006"],
            ["Solver", "Common Random Numbers", "variance reduction for MC parts", "Planned", "P0", "M2", "depends on ADR-003"],
            ["Solver", "Weights -> strip realize + re-verify", "preserve positional/adjacency constraints", "Planned", "P1", "M2", "ADR-006"],
            ["Solver", "Anti-degeneracy constraints", "gameplay-shape targets as hard constraints", "Planned", "P0", "M2", "no boring math"],
            ["Solver", "RTP-by-feature decomposition", "contributions reconcile to the unit", "Planned", "P1", "M2", "anti-degeneracy signal"],
            ["Solver", "PyO3 bindings", "Rust core in-process to Python", "Planned", "P1", "M2", "ADR-007"],
            ["AI layer", "LLM context translator", "constrained decoding from fixed Lego library", "Planned", "P0", "M3", "ADR-008; never writes code"],
            ["AI layer", "Pydantic validation", "models generated from schema", "Planned", "P0", "M3", "ADR-004"],
            ["AI layer", "FastAPI orchestration", "translate / validate / simulate / tune", "Planned", "P1", "M3", ""],
            ["Frontend", "Schema-driven form", "generated from schema; locks hard bounds", "Planned", "P0", "M3", "ADR-004"],
            ["Frontend", "Monaco JSON editor", "raw config view / tweak", "Planned", "P2", "M3", ""],
            ["Frontend", "Persistence (Postgres/SQLite)", "store configs + results", "Planned", "P1", "M3", "connector at M3"],
            ["Frontend", "Playwright E2E", "end-to-end tests", "Planned", "P1", "M3", "blueprint-mandated"],
            ["Compliance", "Advanced mechanics", "cascades / expanding wilds / multipliers", "Planned", "P0", "M4", "MC + Markov"],
            ["Compliance", "GLI/BMM exporter", "PDF/Excel certification math sheet", "Planned", "P0", "M4", ""],
            ["Compliance", "Confidence intervals", "on every Monte Carlo metric", "Planned", "P1", "M4", "ADR-009"],
        ],
    },
    "Metrics": {
        "title": "Metrics — definitions, targets, and how we test them",
        "headers": ["Metric", "Definition", "Target", "Current", "Unit", "Source module", "Test method"],
        "widths": [22, 38, 16, 10, 10, 14, 28],
        "rows": [
            ["RTP (exact)", "total_won / total_bet, computed exactly", "per design (e.g. 96.00)", "", "%", "core-math", "enumeration vs certified sheet"],
            ["RTP (Monte Carlo)", "MC estimate of RTP, with CI", "CI contains exact value", "", "%", "core-math", "10M+ spins"],
            ["Volatility", "std dev of per-spin return", "per design (e.g. med-high)", "", "sigma", "core-math", "integer sum-of-squares"],
            ["Hit frequency", "share of spins with any win", "per design", "", "%", "core-math", "per-spin counter"],
            ["Bonus frequency", "share of spins triggering bonus", "per design (e.g. 1/180)", "", "1/x", "core-math", "counter"],
            ["Max-win probability", "P(total win reaches the cap)", "small", "", "prob", "core-math", "counter"],
            ["% RTP at cap", "RTP delivered at the max-win cap", "low (guard metric)", "", "%", "core-math", "ADR-005 guard"],
            ["Spin throughput", "Monte Carlo spins per second", "10M < 2s (>=5M/s)", "", "spins/s", "core-math", "benchmark"],
            ["Determinism", "seed-replay reproducibility", "50/50 identical to the unit", "", "pass/fail", "core-math", "seed replay across threads"],
            ["Exact-vs-MC delta", "|exact - MC| relative to CI", "within CI", "", "%", "core-math", "parity harness"],
            ["Parity deviation (exact)", "exact RTP vs certified sheet", "< 0.001", "", "%", "core-math", "Math Parity Test"],
            ["Schema validation pass-rate", "stylized prompts -> schema-valid JSON", "100", "", "%", "ai-layer", "fuzz test (1000 prompts)"],
            ["RTP reconciliation", "sum of feature contributions = total", "exact to the unit", "", "units", "core-math", "decomposition check"],
        ],
    },
    "Milestones": {
        "title": "Milestones & gates",
        "headers": ["Milestone", "Version", "Deliverable", "Gate / acceptance", "Status"],
        "widths": [14, 11, 40, 44, 13],
        "rows": [
            ["Foundation", "v0.1.0", "repo, branching, ADRs, draft schema, tooling", "clean documented checkpoint", "Done"],
            ["M1", "v0.2.0", "Core Math Engine (5x3 lines)", "Parity Test passes; seed-replay identical; 10M < 2s", "In progress"],
            ["M2", "v0.3.0", "Auto-Tuner / Solver", "broken 140% RTP config -> fixed playable 96%", "In progress"],
            ["M3", "v0.4.0", "AI translator + FastAPI + frontend", "prompt -> JSON -> simulate -> tune in browser", "Planned"],
            ["M4", "v1.0.0", "Advanced mechanics + GLI/BMM exporter", "exported sheet matches a certified live game", "Planned"],
        ],
    },
    "Versions": {
        "title": "Release / rollback log",
        "headers": ["Version", "Type", "Date", "Summary", "Rollback notes"],
        "widths": [12, 12, 13, 46, 34],
        "rows": [
            ["v0.1.0", "stable", "2026-06-16", "Foundation: repo, stable/beta branching, ADRs, draft schema, workbook, plugins", "baseline checkpoint; git checkout v0.1.0"],
            ["v0.2.0-beta.1", "beta", "2026-06-16", "M1 core-math engine: namespaced RNG, integer accounting, exact+MC, CLI; 6 tests green (~19M spins/s)", "last stable = v0.1.0; git checkout v0.1.0"],
            ["v0.2.0-beta.2", "beta", "2026-06-16", "M1 cont.: exact capped enumeration (cap-erosion) + weight-reels; provenance-labeled; 9 tests green", "last stable = v0.1.0; git checkout v0.1.0"],
            ["v0.3.0-beta.1", "beta", "2026-06-16", "M2 auto-tuner: ES weight-tuning to target RTP + realize/re-verify; gate PASS 139%->96% line game; 13 tests green", "last stable = v0.1.0; git checkout v0.1.0"],
            ["v0.3.0-beta.2", "beta", "2026-06-16", "M2 finished: scatter-consistent (independent-cell) + multi-objective (RTP+vol+hit); scatter re-verify delta 5.8%->0.3%; 16 tests green", "last stable = v0.1.0; git checkout v0.1.0"],
        ],
    },
}

# --- live updates applied at build time (M1 engine built on beta) ---
_metric_current = {
    "RTP (exact)": "95.30",
    "RTP (Monte Carlo)": "95.34 +/- 0.11",
    "Hit frequency": "51.33",
    "Volatility": "2.47",
    "Spin throughput": "~19M/s (10M in 0.55s)",
    "Determinism": "PASS (thread-independent)",
    "Exact-vs-MC delta": "within 95% CI",
    "Parity deviation (exact)": "pending external game",
}
for _r in SHEETS["Metrics"]["rows"]:
    if _r[0] in _metric_current:
        _r[3] = _metric_current[_r[0]]
for _r in SHEETS["Features"]["rows"]:
    if _r[5] == "M1":
        _r[3] = "Done"


def build():
    wb = Workbook()
    wb.remove(wb.active)
    for name, spec in SHEETS.items():
        ws = wb.create_sheet(name)
        ws["A1"] = spec["title"]
        ws["A1"].font = TITLE_FONT
        header_row = 2
        for c, h in enumerate(spec["headers"], start=1):
            cell = ws.cell(row=header_row, column=c, value=h)
            cell.fill = HEADER_FILL
            cell.font = HEADER_FONT
            cell.alignment = Alignment(vertical="center", wrap_text=True)
            cell.border = BORDER
        for r, row in enumerate(spec["rows"], start=header_row + 1):
            for c, val in enumerate(row, start=1):
                cell = ws.cell(row=r, column=c, value=val)
                cell.alignment = WRAP
                cell.border = BORDER
            status_idx = spec["headers"].index("Status") + 1 if "Status" in spec["headers"] else None
            if status_idx:
                sval = row[status_idx - 1]
                if sval in STATUS_FILL:
                    ws.cell(row=r, column=status_idx).fill = STATUS_FILL[sval]
        for i, w in enumerate(spec["widths"], start=1):
            ws.column_dimensions[get_column_letter(i)].width = w
        ws.freeze_panes = ws.cell(row=header_row + 1, column=1)
        ws.sheet_view.showGridLines = False
    wb.save(OUT)
    print(f"wrote {OUT}")


if __name__ == "__main__":
    build()
