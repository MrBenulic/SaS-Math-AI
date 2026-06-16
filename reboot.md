# reboot.md — SaS-Math AI context handoff

> **Living document.** Updated at the end of every working session. When the chat is running low on
> context, say *"we are moving to new chat"* and Claude will refresh this file and hand you the
> **Fresh-chat prompt** at the bottom to paste into a new session — continuing bug-free without
> losing workflow context.

---

## 1. One-liner
B2B SaaS **AI-Driven Slot Math Generation & Tuning Engine**: NL/UI → validated JSON game config →
deterministic **exact + Monte Carlo** math engine → auto-tuning solver → GLI/BMM certification export.

## 2. Current status
- **Date:** 2026-06-16
- **Version / milestone:** `v0.1.0` — **Foundation** (design + scaffolding). _No engine code yet._
- **Branch:** `main` (stable). `beta` created for M1 work.
- **Latest stable tag:** `v0.1.0` · **Latest beta tag:** _(none yet — first beta cut during M1)_
- **Repo (private):** [`MrBenulic/SaS-Math-AI`](https://github.com/MrBenulic/SaS-Math-AI)
- **Local path:** `~/Desktop/SaS-Math AI app`

## 3. How to resume
```bash
cd "$HOME/Desktop/SaS-Math AI app"
git status && git branch && git tag           # confirm clean, on a branch, see checkpoints
git checkout beta                             # do new work on beta, never directly on main
# next milestone is M1 (core-math). See docs/roadmap.md "v0.2.0 — M1".
```

## 4. Architecture & locked decisions (read these first)
- Overview: `docs/architecture.md` — 5 modules (Frontend, AI translator, Core engine, Solver, Compliance).
- **ADRs (load-bearing): `docs/design-decisions.md`.** The five "lock-before-M1":
  1. **ADR-001** Exact combinatorial math is the certification source of truth; MC validates the
     harness + covers non-enumerable features. (The old "1B spins <0.05%" gate is unreachable.)
  2. **ADR-002** Integer/`u128` accounting; render decimal only at the boundary.
  3. **ADR-003** Counter-based **namespaced** RNG `(subsystem, spin, draw)` — the keystone; enables
     determinism + stability under code change + CRN for the solver.
  4. **ADR-004** One schema → TS + Pydantic + **Rust serde**; versioned; old configs reproduce math forever.
  5. **ADR-005** Max-win cap modeled **inside** the math.
- Parity methodology: `docs/parity-test.md`. Versioning/rollback: `docs/versioning.md`.

## 5. Roadmap snapshot
- ✅ **v0.1.0 Foundation** (this) — repo, branching, docs, draft schema, Excel, plugins, toolchains.
- ⏭️ **v0.2.0 M1 — Core Math Engine** (next): namespaced RNG + integer accounting + exact
  enumeration + line-eval Lego blocks + 10M-spin MC + minimal math sheet + CLI. **Gate:** Math
  Parity Test passes against a certified game; seed-replay identical to the unit; 10M spins < 2s.
- ⏳ v0.3.0 M2 solver · v0.4.0 M3 AI+frontend · v1.0.0 M4 advanced mechanics + GLI/BMM exporter.

## 6. Open decisions / TODO
- **LLM provider** for Module 2 — *deferred* (decide in M3). Code a provider-agnostic interface.
- Choose counter-based PRNG concretely in M1 (Philox vs Threefry vs ChaCha) + the output→reel-stop
  mapping (rejection sampling, no modulo bias).
- Pick a known **certified reference game** to recreate for the M1 Parity Test.
- `playwright` + Postgres/SQLite connector: install at M3.

## 7. Key file map
`README.md` · `CLAUDE.md` (rules) · `docs/{architecture,design-decisions,roadmap,versioning,parity-test}.md`
· `schema/game-config.schema.json` (DRAFT v0.1) · `core-math|ai-layer|frontend|tools/README.md`
· `SaS-Math-AI_Features_and_Metrics.xlsx` (feature/metric tracker).

## 8. Environment / toolchain
- Installed: Node v24.16, npm 11.13, Python 3.12.10 (openpyxl 3.1.5), git 2.50.1, **Rust (rustup)**,
  **gh** (GitHub CLI, in `~/.local/bin`).
- macOS arm64. Git identity set locally in the repo.
- Plugins enabled: commit-commands, claude-md-management, security-guidance/sweep, pr-review-toolkit,
  typescript-lsp.

## 9. Session log
- **2026-06-16** — v0.1.0 Foundation: studied the blueprint, ran an independent architecture review,
  redefined the path (exact-first math, integer accounting, namespaced RNG, tri-target schema,
  cap-inside-math, reframed parity test). Created folder, private repo, stable/beta branching, design
  docs/ADRs, draft schema, feature/metrics workbook, this file, plugins, Rust+gh toolchains. Tagged
  `v0.1.0`.

---

## 10. ⭐ Fresh-chat prompt (paste this into a new chat)

```
We are continuing the SaS-Math AI project (B2B slot-math generation & tuning engine).
Local path: ~/Desktop/SaS-Math AI app  (private GitHub repo MrBenulic/SaS-Math-AI).

FIRST: read reboot.md, CLAUDE.md, and docs/design-decisions.md in that folder — they hold the live
status and the locked engineering decisions (ADRs). Then read docs/roadmap.md for what's next.

Current state: v0.1.0 Foundation is complete and tagged (design + scaffolding only, no engine code).
We work on the `beta` branch and promote to `main` (tag vX.Y.Z) only after a milestone gate passes;
we cut beta pre-releases as vX.Y.Z-beta.N. Always tag a stable point before risky changes.

Non-negotiables (see CLAUDE.md / ADRs): integer/u128 money accounting; counter-based NAMESPACED RNG
(subsystem, spin, draw); the JSON schema is the single source of truth for TS + Pydantic + Rust serde;
exact combinatorial math is the certification source of truth and MC only validates the harness +
non-enumerable features; max-win cap is modeled inside the math; the LLM never writes code.

Next milestone is M1 (v0.2.0) — the Rust core-math engine; see docs/roadmap.md and core-math/README.md.
Start by confirming the plan for M1, then proceed. Keep reboot.md and the Excel tracker up to date.
```
