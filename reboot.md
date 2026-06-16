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
- **Version / milestone:** `v0.2.0-beta.1` — **M1 Core Math Engine** built on `beta` (exact + deterministic Monte Carlo). Promotion to stable `v0.2.0` pending an external certified-game parity check.
- **Branch:** `beta` (active work). `main` holds stable `v0.1.0`.
- **Latest stable tag:** `v0.1.0` · **Latest beta tag:** `v0.2.0-beta.1`
- **Repo (private):** [`MrBenulic/SaS-Math-AI`](https://github.com/MrBenulic/SaS-Math-AI)
- **Local path:** `~/Desktop/SaS-Math AI app`
- **M1 verified:** exact RTP 95.30% == Monte Carlo within 95% CI; thread-independent determinism (identical to the unit); hand-verifiable exact ratio 72/27; ~19M spins/s (10M < 0.6s). `cargo test --release` green (6 tests). Demo: `cd core-math && cargo run --release --bin cli -- examples/sample_5x3.json 10000000`.

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
- ✅ **v0.1.0 Foundation** — repo, branching, docs, draft schema, Excel, plugins, toolchains.
- 🟦 **v0.2.0 M1 — Core Math Engine** (built on `beta`, tag `v0.2.0-beta.1`): namespaced ChaCha8
  counter-RNG, integer/u128 accounting, exact enumeration (line marginals + scatter convolution),
  LTR+wild line-eval, deterministic Rayon Monte Carlo, streaming stats, CLI math sheet, 6 passing
  tests. **Remaining for stable v0.2.0:** parity vs an external certified game; fold the max-win cap
  into the exact math; add weight-reels (currently strips only).
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
- **2026-06-16 (M1)** — Built the Rust `core-math` engine on `beta`: namespaced ChaCha8 counter-RNG
  (`rng.rs`), integer/u128 accounting + streaming stats (`accounting.rs`, `stats.rs`), exact
  enumeration line+scatter (`enumerate.rs`), LTR+wild line-eval + scatter counting (`mechanics.rs`),
  deterministic Rayon Monte Carlo (`simulate.rs`), CLI math sheet (`bin/cli.rs`). 6 tests green
  (determinism thread-independent, exact==MC within CI, hand-verifiable 72/27). Reference game
  (`examples/sample_5x3.json`): exact 95.30% (line 87.15 + scatter 8.14), MC 95.34%±0.11, ~19M
  spins/s. Tagged `v0.2.0-beta.1`. **Remaining for stable v0.2.0:** external certified-game parity;
  cap-inside-exact-math; weight-reels.

---

## 10. ⭐ Fresh-chat prompt (paste this into a new chat)

```
We are continuing the SaS-Math AI project (B2B slot-math generation & tuning engine).
Local path: ~/Desktop/SaS-Math AI app  (private GitHub repo MrBenulic/SaS-Math-AI).

FIRST: read reboot.md, CLAUDE.md, and docs/design-decisions.md in that folder — they hold the live
status and the locked engineering decisions (ADRs). Then read docs/roadmap.md for what's next.

Current state: M1 core-math engine is BUILT on `beta` (tag v0.2.0-beta.1): exact enumeration +
deterministic Monte Carlo, 6 tests green, ~19M spins/s, exact==MC within CI. v0.1.0 Foundation is on
`main`. We work on `beta` and promote to `main` (tag vX.Y.Z) only after a milestone gate passes; we
cut beta pre-releases as vX.Y.Z-beta.N. Always tag a stable point before risky changes.

Non-negotiables (see CLAUDE.md / ADRs): integer/u128 money accounting; counter-based NAMESPACED RNG
(subsystem, spin, draw); the JSON schema is the single source of truth for TS + Pydantic + Rust serde;
exact combinatorial math is the certification source of truth and MC only validates the harness +
non-enumerable features; max-win cap is modeled inside the math; the LLM never writes code.

Next: finish M1 -> stable v0.2.0 (needs parity vs an external CERTIFIED reference game; then fold the
max-win cap into the exact math and add weight-reels), then M2 (the solver). See docs/roadmap.md and
core-math/README.md; run `cargo test --release` in core-math/. Do work on `beta`; keep reboot.md and
the Excel tracker up to date.
```
