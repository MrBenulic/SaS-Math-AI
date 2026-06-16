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
- **Version / milestone:** `v0.3.0-beta.2` — **M2 Auto-Tuner/Solver** on `beta`, now **scatter-consistent + multi-objective** (tunes RTP + volatility + hit-frequency). Weighted reels use the independent-cell model so the solver objective and re-verify agree. M1 engine complete. Stable promotion still pending external certified-game parity.
- **Branch:** `beta` (active work). `main` holds stable `v0.1.0`.
- **Latest stable tag:** `v0.1.0` · **Latest beta tag:** `v0.3.0-beta.2`
- **Repo (private):** [`MrBenulic/SaS-Math-AI`](https://github.com/MrBenulic/SaS-Math-AI)
- **Local path:** `~/Desktop/SaS-Math AI app`
- **Verified (16 tests green):** M1 exact+capped+determinism; M2 — gate PASS (139%→96%); **scatter game now re-verifies** with <1% delta (was 5.8%); weighted exact scatter == MC; **multi-objective** hits RTP + a chosen volatility (4.0 and 2.0 both achieved at ~96% RTP, hit-freq trading off). Demos: `cd core-math && cargo run --release --bin tune -- examples/broken_143.json 0.96 250` (scatter); `... examples/broken_lineonly.json 0.96 150 3.0` (RTP+vol).

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
- 🟦 **v0.2.0 M1 — Core Math Engine** (built on `beta`, tag `v0.2.0-beta.2`): namespaced ChaCha8
  counter-RNG, integer/u128 accounting, **closed-form + exact-capped** enumeration, LTR+wild
  line-eval, **weight-reels** (realized to strips, ADR-006), deterministic Rayon Monte Carlo,
  streaming stats, provenance-labeled CLI math sheet, 9 passing tests. **Remaining for stable
  v0.2.0:** parity vs an external certified game (and `ways_243` line-eval is still a stub).
- 🟦 **v0.3.0 M2 — Auto-Tuner/Solver** (built on `beta`, tag `v0.3.0-beta.2`): `(mu/mu_w,lambda)`-ES
  with CSA tunes weights to target **RTP + volatility + hit-frequency** (RTP exact closed-form; vol/hit
  via CRN Monte Carlo). Weighted reels are **independent-cell** so the solver objective == re-verify
  (scatter games re-verify with <1% delta). Gate PASS. **Remaining:** full CMA-ES covariance; PyO3
  bindings; `ways_243`/buy-feature mechanics; (stable promotion needs external certified-game parity).
- ⏳ v0.4.0 M3 AI+frontend · v1.0.0 M4 advanced mechanics + GLI/BMM exporter.

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
- **2026-06-16 (M1 cont.)** — Cap fold-in + weight-reels. `enumerate.rs`: exact capped RTP via
  column-tuple enumeration (`min(win,cap)` per atom), feasibility-bounded (`ENUM_LIMIT`) with a
  numeric-MC fallback labeled by provenance; built-in cross-check (enumeration-uncapped ==
  closed-form). `mechanics.rs`: weight-reels realized to a deterministic spread strip (ADR-006).
  9 tests green (added: cap_fold_in_is_exact 40/27, enumeration==closed-form, weights==strips).
  CLI shows capped RTP + cap-erosion + provenance (sample 50x cap → 0.057% erosion, exact over
  33.5M combos). Tagged `v0.2.0-beta.2`. **Remaining for stable v0.2.0:** external certified-game
  parity only (`ways_243` is a minor stub).
- **2026-06-16 (M2)** — Auto-tuner/solver. `solver.rs`: closed-form RTP as a smooth function of
  per-reel weights (line marginals + binomial scatter), reusing `Prepared::line_pay/scatter_pay`;
  `(mu/mu_w,lambda)`-ES with CSA step-size control (deterministic); bounds for non-degeneracy +
  light regularizer; `realized_config` realizes tuned weights → integer strips for re-verification
  (ADR-006). `bin/tune.rs`: load config + target RTP → tune → re-verify (exact + MC) → write tuned
  config. Gate PASS: `examples/broken_lineonly.json` 139.2% → 95.9% (within 0.09%), realize-delta
  0.02%, hit-freq 42%→45%. 13 tests green (4 solver). Tagged `v0.3.0-beta.1`. **Next M2:** make the
  engine's weighted-reel scatter independent-cell (binomial) to remove the solver↔engine scatter
  delta; volatility/hit-freq objectives; CMA-ES covariance; PyO3.
- **2026-06-16 (M2 finished)** — (1) **Scatter consistency:** reworked the engine to carry an explicit
  `ReelModel{Strips,Weights}` (`mechanics.rs`). Strips keep positional window scatter + stop-based MC;
  Weights use **independent-cell**: integer line marginal + binomial scatter (`enumerate.rs`) +
  independent-cell MC (`simulate.rs`). Solver objective and re-verify now use the SAME model →
  `broken_143` scatter re-verify delta 5.8% → 0.32% (rounding only). (2) **Multi-objective:**
  `solver.rs` refactored to a shared `run_es` core; added `solve_multi` + `SolveTargets` to tune
  **RTP + volatility + hit-frequency** (RTP exact; vol/hit via CRN Monte Carlo, fixed seed). `tune`
  CLI takes `[target_vol] [target_hit]`. Demo: vol 4.0→4.029 (hit-freq 29%) and 2.0→1.970 (36%) at
  ~96% RTP. 16 tests green. Tagged `v0.3.0-beta.2`. **Next:** full CMA-ES covariance, PyO3; or M3
  (AI + frontend). Stable needs external certified-game parity.

---

## 10. ⭐ Fresh-chat prompt (paste this into a new chat)

```
We are continuing the SaS-Math AI project (B2B slot-math generation & tuning engine).
Local path: ~/Desktop/SaS-Math AI app  (private GitHub repo MrBenulic/SaS-Math-AI).

FIRST: read reboot.md, CLAUDE.md, and docs/design-decisions.md in that folder — they hold the live
status and the locked engineering decisions (ADRs). Then read docs/roadmap.md for what's next.

Current state: M1 engine + M2 solver are BUILT on `beta` (tag v0.3.0-beta.2): exact closed-form +
capped enumeration, weight-reels (independent-cell), deterministic Monte Carlo, and a multi-objective
ES auto-tuner that hits target RTP + volatility + hit-frequency and re-verifies against the integer
engine. 16 tests green. Gate PASS (139%→96%); scatter games re-verify with <1% delta. v0.1.0
Foundation is on `main`. We work on `beta` and promote to `main` (tag vX.Y.Z) only after a milestone
gate passes; we cut beta pre-releases as vX.Y.Z-beta.N. Always tag a stable point before risky changes.

Non-negotiables (see CLAUDE.md / ADRs): integer/u128 money accounting; counter-based NAMESPACED RNG
(subsystem, spin, draw); the JSON schema is the single source of truth for TS + Pydantic + Rust serde;
exact combinatorial math is the certification source of truth and MC only validates the harness +
non-enumerable features; max-win cap is modeled inside the math; the LLM never writes code.

Next options: M2 polish — full CMA-ES covariance adaptation and PyO3 bindings (Python access to the
engine/solver); OR start M3 — AI Context Translator (NL -> validated JSON, provider-agnostic) +
FastAPI orchestration + Next.js config UI. (Scatter consistency + multi-objective vol/hit are DONE.)
Stable promotion still needs external CERTIFIED-game parity. See docs/roadmap.md, core-math/README.md;
run `cargo test --release` in core-math/. Do work on `beta`; keep reboot.md + the Excel tracker current.
```
