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
- **Version / milestone:** `v0.4.0-beta.2` — **M3 complete (backend + frontend)** on `beta`: FastAPI ai-layer (provider-agnostic translator) + a **Next.js "Slot Studio"** UI (Tailwind/Zustand/Monaco) + a Rust JSON engine bridge. Full **prompt → JSON → simulate → tune works in the browser**. M1 + M2 complete.
- **Branch:** `beta` (active work). `main` holds stable `v0.1.0`.
- **Latest stable tag:** `v0.1.0` · **Latest beta tag:** `v0.4.0-beta.2`
- **Repo (private):** [`MrBenulic/SaS-Math-AI`](https://github.com/MrBenulic/SaS-Math-AI)
- **Local path:** `~/Desktop/SaS-Math AI app`
- **Verified (16 Rust + 6 Python tests green; browser-verified):** **M3 gate IN BROWSER** — prompt "5x3, 20 lines, 96% RTP, medium vol, wild+scatter" → Translate → Simulate (149.19%) → Auto-tune → **96.00%** (realize-delta 0.01%, 1680 evals), via Next.js UI → FastAPI → Rust engine. Run: build `core-math` engine, `ai-layer` uvicorn (:8000), `frontend` `npm run dev` (:3000).

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
- 🟦 **v0.4.0 M3 — AI + frontend** (on `beta`, tag `v0.4.0-beta.2`): FastAPI `ai-layer` (provider-
  agnostic translator + fixed mechanics library) + a Rust JSON engine bridge (`core-math/src/bin/
  engine.rs`) + a **Next.js "Slot Studio"** (Tailwind/Zustand/Monaco). prompt→JSON→simulate→tune works
  in the browser (16 Rust + 6 Python tests; browser-verified). **Remaining:** schema→TS codegen;
  Playwright E2E; Postgres/SQLite persistence; real LLM provider wiring.
- ⏳ v1.0.0 M4 advanced mechanics + GLI/BMM exporter.

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
- **2026-06-16 (M3 beta.1)** — AI layer backend. `core-math/src/bin/engine.rs`: JSON bridge
  (stdin/stdout) so Python drives the engine/solver without PyO3 — `simulate`/`tune` commands. Also
  changed `pays`/`weights` map keys to `String` (internally-tagged serde enums buffer content and
  break integer map keys). `ai-layer/` (FastAPI + Pydantic 2): `models.py` (schema mirror),
  `mechanics.py` (fixed library + base-config builder), `translator.py` (`RuleBasedTranslator` +
  provider stub, ADR-008), `engine.py` (subprocess bridge), `main.py` (`/translate /validate
  /simulate /tune /design /health`). 6 pytest green. **Gate:** `POST /design` runs prompt→JSON→
  simulate→tune; demo "5x3, 20 lines, 96% RTP, medium vol, wild+scatter" → 1.49 → **0.9600** RTP,
  realize-delta 0.0001. Tagged `v0.4.0-beta.1`. **Next M3:** Next.js/Tailwind/Zustand/Monaco UI;
  schema→TS codegen; Playwright; persistence; real LLM provider.
- **2026-06-16 (M3 frontend)** — Next.js 16 + React 19 + Tailwind v4 + Zustand + Monaco "Slot Studio"
  (`frontend/`): `app/page.tsx` (prompt → Monaco config → Simulate/Tune with RTP/vol/hit cards),
  `lib/api.ts` (ai-layer client), `store/useStore.ts`. CORS added to the ai-layer. **Browser-verified**
  via preview tools: Translate (POST /translate 200 → config in Monaco) → Simulate (149.19%) →
  Auto-tune → **96.00%** (delta 0.01%, 1680 evals); `next build` type-checks clean. Tagged
  `v0.4.0-beta.2`. **Next:** schema→TS codegen, Playwright E2E, persistence, real LLM provider; then
  M4. To run the UI: `.claude/launch.json` `sas-frontend`, or `cd frontend && npm run dev` (needs
  ai-layer uvicorn + core-math engine).
- **2026-06-16 (strategy + event-stream wedge)** — Studied the open-source incumbent
  **StakeEngine/math-sdk** (sim-first, Stake-RGS-coupled, MIT) and reframed strategy around the
  **animation-studio wedge** (art + certified math = one package) and the GLI/BMM whitespace; wrote
  `docs/product-strategy.md` (competitive landscape + prioritized "on-steroids" roadmap + clean-room
  guardrail: learn ideas only, NEVER launder OSS). Built the **deterministic event stream** — the
  art↔math bridge (roadmap rank 1): `core-math/src/outcome.rs` (`RoundOutcome`/`LineWin`/`ScatterWin`,
  `simulate_one_traced`), shared `fill_grid` so the MC hot path stays allocation-free and provably
  equals the trace, and a `{"command":"replay","spin_index":N}` variant on `bin/engine.rs` (emits the
  integer event stream a frontend can replay to drive Spine/atlas animations). Also fixed two
  line-eval correctness traps: `ways_243` now **fails loud** (was a silent zero-pay), `both_ways` no
  longer **double-pays a full-line** win (shared no-alloc `ltr_run`; new `line_wins` detail).
  **22 Rust tests green** (was 16; +6 in `tests/outcome.rs`: scalar==traced over 400k spins,
  deterministic/byte-stable replay, outcome invariants, both_ways full-line-once, ways_243 rejected).
  **UNCOMMITTED on `beta`** (rollback point is tag `v0.4.0-beta.2`); next: commit + tag a beta
  snapshot. **Next build:** finish rank 2/3 (exact `ways_243` + golden-vector CI gate), then the
  `presentation.bind.json` art-binding sidecar (rank 4), then the content-hashed published artifact.
- **2026-06-16 (committed + art-binding sidecar)** — Committed/tagged the above as `v0.4.0-beta.3`
  (pushed). Then built the **art-binding sidecar** (roadmap rank 4, the wedge): `schema/presentation-
  bind.schema.json` (the art↔math contract — symbol id + closed event-type enum → Spine skeleton /
  atlas region / spritesheet frames / sound; PRESENTATION ONLY, kept out of core-math so the cert
  artifact stays pure), `ai-layer/app/binding.py` (Pydantic models + `EVENT_TYPES` mirroring the
  engine RoundOutcome + `validate_binding` bidirectional-completeness check), and endpoints
  `POST /validate-bind` + `GET /bind/event-types` in `main.py`. **12 Python tests green** (was 6;
  +6 in `tests/test_binding.py`). To be tagged `v0.4.0-beta.4`. **Next build:** wire a "load my Spine
  files → validate binding" step into the Slot Studio UI (so the studio sees green/red coverage), the
  content-hashed published-math artifact (rank 5), and finish exact `ways_243` (rank 2 remainder).
- **2026-06-16 (cert fingerprint + golden vector)** — Two parked inputs (user-confirmed): a real LLM
  provider is deferred; a certified math/PAR sheet is expected ~late June 2026 — so we build
  everything NOT blocked on those. Built the **published-math artifact + config fingerprint** (rank 5,
  Python stdlib, no new deps): `ai-layer/app/artifact.py` (`canonical_config_bytes` →
  **SHA-256** `config_hash` → `build_published_artifact`), endpoints `POST /publish` + `POST
  /config-hash`; tied into the binding (`validate_binding` now verifies an optional
  `targets_config_hash`). Added a **golden replay vector** (rank 3): `tests/outcome.rs`
  `golden_replay_sample_5x3_spin1` pins an exact RoundOutcome so any seed-perturbing code change is
  caught (ADR-003). **23 Rust + 18 Python tests green** (+1 Rust golden, +4 artifact, +2 binding-hash).
  To be tagged `v0.4.0-beta.5`. **Next (no LLM/cert needed):** Studio UI to drop in Spine files and see
  binding coverage; expose the engine's exact integer rational into the artifact; `index.json` ledger;
  typed `NoFeasibleSolution`; then exact `ways_243` with hand-verified tests.
- **2026-06-16 (Studio art-binding UI)** — Added **"4 · Art binding"** to the Next.js Slot Studio:
  `frontend/components/BindingPanel.tsx` (drag-drop / browse / paste file names → auto-match each
  symbol to an asset by name → live **green/red coverage matrix** per symbol + per event, calling
  `POST /validate-bind`), plus `lib/api.ts` binding types + `validateBind`/`eventTypes`, and
  `store/useStore.ts` bind state/actions (auto-match + validate). **Browser-verified** via preview
  tools: Translate → load `wild/scatter/h1/h2/l1.json` (omitting L2/L3) → red banner "✗ 2 coverage
  gaps", L2/L3 dots red, others green; add `l2/l3.json` → green "✓ Complete — Art + certified math =
  one package". No console errors; `tsc --noEmit` clean. To be tagged `v0.4.0-beta.6`. **Next (no
  LLM/cert needed):** richer per-symbol binding (idle/land/win anims, sound) + export the bind file;
  expose the engine's exact integer rational into the published artifact; `index.json` ledger; typed
  `NoFeasibleSolution`; then exact `ways_243` with hand-verified tests. Run: ai-layer uvicorn :8000 +
  `frontend npm run dev` :3000 (engine only needed for simulate/tune/publish, not bind).
- **2026-06-16 (richer bindings + export)** — Extended the Slot Studio binding panel: per-symbol
  **idle/land/win animation + sound** inputs (collapsible per row) and an **Export
  presentation.bind.json** button that assembles the full binding, **pins the SHA-256
  `targets_config_hash`** (via `POST /config-hash`), downloads the file, and shows a collapsible JSON
  preview. Store refactored to a shared `buildBinding()` (used by validate + export) + `symbolAnims`
  + `doExportBind`. **Browser-verified** (preview tools): set WILD `win_anim=wild_explode`, exported →
  "7 symbol bindings · 6 events · fingerprint pinned", JSON contains `{skeleton:"wild.json",
  win_anim:"wild_explode"}` + `targets_config_hash:sha256:f0f39…`; banner green; `tsc --noEmit` clean,
  no console errors. To be tagged `v0.4.0-beta.7`. Also evaluated the owner's link
  https://probability.infarom.ro/slots.html (Barboianu, "Mathematics of Slots") — an academic
  *framework* (Type A/B reels, simple/complex event taxonomy), **not** worked numbers/parity data;
  recorded as a reference (the certified sheet ~late June 2026 is still the ADR-010 parity gate).
  **Next (no LLM/cert needed):** fold the engine's exact integer rational into the published artifact;
  `index.json` ledger of published games; typed `NoFeasibleSolution`; then exact `ways_243`.
- **2026-06-16 (real LLM — Groq wired)** — Owner supplied a **Groq API key** (in `API/Api_Grok.docx`,
  extracted to gitignored `ai-layer/.env`; `/API/` added to `.gitignore` so the key doc is never
  committed). Wired a real **GroqTranslator** (`ai-layer/app/translator.py`): ADR-008-correct — the
  LLM (Groq `llama-3.3-70b-versatile`, JSON mode) emits ONLY a constrained `GameIntent`
  (grid/lines/rtp/volatility/wild/scatter/both-ways); our `build_from_intent` clamps it and
  `build_base_config` builds the validated config from the fixed library; math stays in Rust. Key read
  from env or `ai-layer/.env` (never logged). `httpx` promoted to a main dep; `/translate` returns 502
  on provider failure so the UI can fall back. Frontend: a **"✨ Use AI (Groq)"** toggle (store `useAI`
  → provider `groq|rule`). **Verified live**: backend call parsed "6x4, 40 lines, 94% RTP, high vol,
  both ways" exactly; browser toggle parsed a WORDS-ONLY prompt ("seven reels by two rows, thirty
  paylines, ninety-four percent, low vol, no scatter") → 7x2 / 30 lines / no SCATTER / rtp 0.94 / vol
  low — things the rule-based regex cannot do. **23 Python tests** (+5 translator, incl. a mocked-Groq
  test) + frontend `tsc` clean, no console errors. To be tagged `v0.4.0-beta.8`. Default translator is
  still rule-based (no key needed); Groq is opt-in. **Next:** richer LLM intent (cap_x, grid presets),
  exact integer rational in the published artifact, `index.json` ledger, typed `NoFeasibleSolution`.
- **2026-06-16 (exact rational + ledger)** — Strengthened the cert artifact: `bin/engine.rs`
  `SimResponse` now emits the **integer-exact line RTP as a rational** (`exact_line_ratio_num/den` as
  u128 decimal strings, ADR-002/009) + `exact_combos`; verified 584886030/671088640 == line_rtp on the
  sample (33.5M combos). Added a **file-backed published-math ledger** `ai-layer/app/ledger.py`
  (`record`/`read_index`, content-addressed, upsert-by-fingerprint), `POST /publish {persist:true}` +
  `GET /ledger`; `published/` gitignored. **Live-verified**: publish→ledger round-trip lists the game
  with its exact_rtp + exact_line_ratio. **27 Python tests** (+4 ledger). Tagged `v0.4.0-beta.9`.
  **Next:** typed `NoFeasibleSolution` from the solver; exact `ways_243`.
- **2026-06-16 (solver feasibility)** — Added a typed **`Feasibility`** to `solver.rs` (`Feasible` /
  `Infeasible { rtp_error, missed }`) computed in `finalize` (RTP tol 1pt; vol/hit 15% rel);
  `SolveResult.feasible()`; surfaced via `bin/engine.rs` `TuneResponse.feasible` + `missed_targets`.
  An over-constrained tune now reports a diagnostic instead of a silently-off best effort (ADR-006
  anti-degeneracy). **25 Rust tests** (+2: infeasible target flagged, feasible target passes).
  Tagged `v0.4.0-beta.10`. **Next:** exact `ways_243` (whole-grid ways eval + exact enumeration,
  hand-verified) — the last non-cert engine item; then optional richer LLM intent / Studio publish UI.
- **2026-06-16 (243-ways support, end to end)** — Implemented **`ways_243`** (was a fail-loud stub):
  whole-grid all-ways evaluation. `mechanics.rs` `ways_pay`/`ways_wins` (per-symbol left-anchored run,
  ways = product of per-reel symbol-or-wild counts, pay = `pays[s][run]*ways`); `simulate.rs` +
  `outcome.rs` branch on `is_ways()`; **exact closed-form** `enumerate.rs::ways_rtp_exact` (product of
  per-reel mean counts × tail `P(count==0)`; works for Strips via empirical per-reel count dist and
  Weights via Binomial — reels independent so the expectation factorizes; zero sampling error, f64).
  Translators expose it (`GameIntent.ways`, rule detects "243"/"all-ways", Groq prompt). **Verified**:
  exact == BY-HAND `29/8 = 3.625` (direct 8-grid enumeration) for BOTH reel models; exact == MC within
  CI; thread-independent; all-A 3x3 pays 243×; engine-bridge live `exact 3.625`, MC `3.6198±0.0089`.
  **28 Rust + 29 Python tests** (+3 ways, +2 translator). Tagged `v0.4.0-beta.11`. **Engine roadmap is
  now feature-complete for the non-cert path** (lines/both-ways/ways, scatter, wild, cap, exact+MC,
  solver+feasibility, event stream, artifact+ledger, Groq LLM). **Next (non-cert):** Studio Publish
  button (UX over /publish + /ledger); optional: scale exact capped RTP (rank 6), schema→TS codegen,
  PyO3. **Still parked:** certified-game parity (ADR-010) — the only thing gating a stable release.
- **2026-06-16 (Studio Publish UI)** — Added **"5 · Publish"** to the Slot Studio (`page.tsx` +
  `lib/api.ts` `publish`/`ledger` + store `doPublish`/`published`/`ledgerCount`): one click runs
  `/publish {persist}` → shows exact RTP + the SHA-256 **fingerprint** + "ledger holds N games".
  **Browser-verified**: publish → "Exact RTP 149.19% · fingerprint sha256:f0f39eeb… · ledger holds 2
  games"; no console errors; `tsc` clean. Tagged `v0.4.0-beta.12`. The full studio loop is now visible
  end to end: describe (AI or rule) → simulate → tune → bind art → **publish a fingerprinted package**.
  **Next (non-cert, optional):** scale exact capped RTP (rank 6), schema→TS codegen, PyO3, Playwright
  E2E. **Blocking stable:** certified-game parity sheet (ADR-010) — owner supplying ~late June 2026.
- **2026-06-16 (ADR-004 schema conformance)** — Added `ai-layer/tests/test_schema_conformance.py`
  (jsonschema, added to dev deps): validates EVERY `core-math/examples/*.json` config + the
  Pydantic-built + rule-translator output against `schema/game-config.schema.json` — catching
  schema↔model↔engine drift (the ADR-004 anti-drift guarantee) without needing codegen tooling.
  **44 Python tests** (+15). Tagged `v0.4.0-beta.13`. **Non-cert roadmap is now exhausted of safe,
  self-contained work.** Remaining items each need a decision or the cert sheet: (a) scale exact
  capped RTP (rank 6) — risky to do without the certified sheet to validate (could corrupt a cert
  number); (b) full schema→TS/Pydantic CODEGEN replacing hand-written models (refactor; the
  conformance test already guards the risk); (c) PyO3 native bindings (ADR-007, perf infra — the JSON
  bridge works); (d) Playwright E2E (heavy browser install; preview-tool verification already covers
  it). **Blocking stable:** certified-game parity (ADR-010), owner supplying ~late June 2026.
- **2026-06-16/17 (UI redesign + hardening kickoff)** — Owner asked for a modern "awwward" UI +
  bug-proofing. **(1) Redesign** of the Slot Studio (`frontend`): new design system in `globals.css`
  — **complementary palette: cool indigo (231°) ↔ warm gold (43°)** on deep neutral ink; depth/glass
  panels, ambient indigo+gold aura + grid texture (lightweight CSS, NO three.js — chose craft over 3D
  gimmick for a serious tool), Geist fonts (was Arial). **GSAP** (core, 3.15 — `@gsap/react` has a
  React-19 peer conflict so used core via `useEffect`) for entrance stagger + `AnimatedNumber` count-up
  on metrics. 5 numbered step cards; gold reserved for the hero RTP + Publish CTA (premium via
  scarcity). NOTE: Tailwind v4 did NOT emit `@layer components` classes → moved component classes to
  top-level CSS and dropped `width:100%` from `.field` (callers use w-full/flex-1/w-24). **Browser +
  prod-build verified** (`npm run build` clean; no console errors; gold EXACT RTP 149.19% rendered).
  Tagged `v0.4.0-beta.14`. **(2) Hardening** — launched an adversarial multi-agent audit workflow
  (Rust+Python+frontend, each finding independently verified); fixes + property/fuzz/edge tests land
  next once it reports. **GSAP yes; three.js deferred to a future *meaningful* 3D viz (reels / win-dist
  surface), not chrome.**
- **2026-06-17 (hardening pass — audit + fixes)** — Ran a 38-agent adversarial audit (each finding
  independently verified); **20 confirmed bugs** (1 critical, 7 high). Removed stray probe test files
  the audit agents left (`tests/audit_zzz*.rs`, `probe*_test.rs` — had been swept into beta.14 by a
  concurrent `git add -A`). **Fixed:** (critical) solver closed-form ignored ways_243 → added
  `ways_rtp` so ways games tune correctly + agree with exact; (high) engine validations in
  `Prepared::new` — reject bet_units=0, cap_x≤0/non-finite (was: negative cap silently capped every
  spin to 0), reel weight-sum > u32::MAX (was: `as u32` truncation/panic), multiple scatters, empty
  paylines for line games; scatter `scatter_pay` now guards count≥1 + paytable rejects "0"/count<1
  key (was: silent RTP inflation); (high) ledger now locked + atomic (temp+os.replace) + preserves a
  corrupt index instead of overwriting; ways `0/0` line-ratio recorded as null; (high) frontend
  validate-bind latest-wins seq guard; autoMatch uses exact-stem/word-boundary (no more H1→H10);
  tune target clamped. (medium) `solve_multi` epsilon-floors zero targets. **Tests added/green:**
  `core-math/tests/hardening.rs` — 8 regression + a 40-config **fuzz** asserting invariants
  (exact==MC-when-comparable, capped≤uncapped, cap never exceeded, scalar==traced, determinism,
  thread-independence, no panics); +2 ledger tests. **37 Rust + 46 Python tests green**; frontend
  `tsc` + `npm run build` clean; app browser-verified functional (simulate renders). Tagged
  `v0.4.0-beta.15`. **Deferred (documented, lower-risk/edge):** exact-capped DP scaling (best with the
  cert sheet), u128 denominator saturation on absurd reel spaces, full schema→codegen, PyO3,
  Playwright. **Stable still gated only by certified-game parity (ADR-010).**
- **2026-06-17 (Excel math sheet — the deliverable)** — Owner asked for "an output like an Excel file
  like mathematicians deliver". Built it (first slice of the M4 GLI/BMM exporter; no cert sheet
  needed): engine `SimResponse` now exposes the **win-distribution histogram** (14 bands from the
  ADR-012 accumulator); `ai-layer/app/mathsheet.py` (`build_math_sheet_xlsx`, openpyxl) produces a
  styled multi-sheet **.xlsx**: Summary (game, exact RTP + line/scatter/capped, MC±CI, volatility, hit
  freq, max win, integer line-ratio, fingerprint, RNG, generated-at), Symbols, Reels (strips or
  weights+probabilities), Paytable matrix, Win distribution. Endpoint `POST /math-sheet` returns the
  file; Studio step 5 has a **"⬇ Download math sheet (.xlsx)"** button. openpyxl added to deps.
  **Verified**: endpoint returns a valid 9 KB 5-sheet workbook (reopened with openpyxl; Exact RTP
  95.2955%); browser button → `POST /math-sheet → 200 OK` → download. **49 Python tests** (+3
  mathsheet); frontend tsc + build clean. Tagged `v0.4.0-beta.16`. This is now the tangible thing a
  studio/operator can open and read.
- **2026-06-17 (RNG honesty fix)** — Owner challenged the RNG. Found the `rng.generator` field offered
  `philox`/`threefry`/`chacha` (schema **defaulted to philox**) but the engine NEVER reads it — it
  always uses ChaCha8, and the math sheet printed the raw config string, so a sheet could claim
  "philox" while ChaCha8 ran. Fixed the dishonesty: schema + Pydantic enum restricted to `["chacha"]`
  (default chacha); `Prepared::new` now **rejects any non-chacha generator** (no silent substitution);
  math sheet reports the REAL primitive ("ChaCha8 — counter-based, namespaced (subsystem·spin·draw)").
  `master_seed` was and is real/used. **38 Rust** (+1 reject) **+ 49 Python** green. Tagged
  `v0.4.0-beta.17`. (Honest note recorded: the engine/RNG/solver were built before the human-readable
  Excel deliverable — prioritization miss; the .xlsx now exists, beta.16.)

---

## 10. ⭐ Fresh-chat prompt (paste this into a new chat)

```
We are continuing the SaS-Math AI project (B2B slot-math generation & tuning engine).
Local path: ~/Desktop/SaS-Math AI app  (private GitHub repo MrBenulic/SaS-Math-AI).

FIRST: read reboot.md, CLAUDE.md, and docs/design-decisions.md in that folder — they hold the live
status and the locked engineering decisions (ADRs). Then read docs/roadmap.md for what's next.

Current state: M1 engine + M2 solver + M3 (ai-layer + Next.js frontend) are BUILT on `beta` (tag
v0.4.0-beta.2). Rust core: exact closed-form + capped enumeration, independent-cell weight-reels,
deterministic MC, multi-objective ES auto-tuner (RTP + volatility + hit-frequency). M3: a FastAPI
ai-layer (provider-agnostic translator) + a Next.js "Slot Studio" UI (Tailwind/Zustand/Monaco) calling
the Rust engine via a JSON bridge (`bin/engine.rs`) — full prompt→JSON→simulate→tune works IN THE
BROWSER. 16 Rust + 6 Python tests green. v0.1.0 Foundation is on `main`. We work on `beta` and promote
to `main` (tag vX.Y.Z) only after a milestone gate passes; we cut beta pre-releases as vX.Y.Z-beta.N.
Always tag a stable point before risky changes.

Non-negotiables (see CLAUDE.md / ADRs): integer/u128 money accounting; counter-based NAMESPACED RNG
(subsystem, spin, draw); the JSON schema is the single source of truth for TS + Pydantic + Rust serde;
exact combinatorial math is the certification source of truth and MC only validates the harness +
non-enumerable features; max-win cap is modeled inside the math; the LLM never writes code.

Next: M3 polish — schema→TS codegen, Playwright E2E, Postgres/SQLite persistence, and wiring a real LLM
provider into the translator stub; OR start M4 — advanced mechanics (cascades, expanding wilds,
multipliers) + the GLI/BMM compliance exporter. (M3 ai-layer + Next.js Slot Studio are DONE and
browser-verified.) Other open: M2 polish (CMA-ES covariance, PyO3); stable needs external CERTIFIED-game
parity. To run the ai-layer: `cd ai-layer && python3 -m venv .venv &&
.venv/bin/pip install -e '.[dev]'`, build `core-math` engine (`cargo build --release --bin engine`),
then `.venv/bin/python -m pytest -q` or `.venv/bin/uvicorn app.main:app`. Do work on `beta`; keep
reboot.md + the Excel tracker current.
```
