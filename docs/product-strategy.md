# Product Strategy & Competitive Landscape

> Why we build what we build, who we're really competing with, and the order of operations.
> Companion to `roadmap.md` (the milestone map) and `design-decisions.md` (the locked ADRs).

## 1. The wedge — who we are

SaS-Math AI is built by an **online-slot animation studio**. The studio already ships the full
**presentation package** — Spine skeletons, JSON, texture atlases, spritesheets — everything needed
to *implement* a slot game visually. The one thing it does **not** produce is the **math/logic**
(RTP, reel strips, paytables, win evaluation, certified RNG).

**The moat is the combination, not the math alone.** "An animation studio that also ships *certified
math*" is a more complete, stickier offering than a pure-art house — and no pure math vendor has the
art pipeline. The product's job is to make **art + certified math = one verifiable package**.

## 2. The incumbent — StakeEngine / math-sdk

The dominant open-source tool in this space is [`StakeEngine/math-sdk`](https://github.com/StakeEngine/math-sdk)
("Carrot math engine"): MIT-licensed, Python sim engine + optional Rust optimizer, production behind
Stake.com, coupled to Stake's own **Carrot RGS** and the stake-engine.com publishing platform (10%
GGR). It is mature and battle-tested. Verified characteristics:

- **Simulation-first**: 100k–1M-spin "books"/lookup tables an optimizer reweights to a target RTP
  (full enumeration is called "infeasible"). RTP is exact only over the *sampled* outcome set.
- **RNG**: Python global Mersenne Twister, reseeded per spin — per-sim reproducible but fragile
  under code change.
- Integer payout (`×100`) + integer probabilities, hashable (validates our ADR-002).
- **No AI/LLM layer** in the SDK (Stake ships a separate consumer product, *SlotGPT*).
- **Zero** GLI/BMM / third-party-certification support.

### Where we differentiate (and where they're ahead)

| | SaS-Math AI | StakeEngine |
|---|---|---|
| Math source of truth | **Exact** enumeration/convolution (ADR-001) | Simulation-first |
| RNG | **Namespaced counter-RNG**, stable under code change (ADR-003) | Global reseed per spin |
| Config | One **schema** → TS+Pydantic+Rust serde (ADR-004) | Hand-written Python |
| AI | NL→config, **off the certified-math critical path** (ADR-008) | None in SDK |
| Target | **Provider-agnostic GLI/BMM** export | Stake's own RGS |
| Maturity | Pre-market, rigorous | Live, proven, free |

**Honest gaps:** they're more mature, more expressive (any mechanic via Python), have a shipped
build→publish→earn loop + a player-facing runtime, and are free. We win on certification rigor and
the art-studio wedge — not on publishing.

### Strategic posture

1. **Differentiate on certification; don't compete on publishing.** Make the GLI/BMM exporter the
   headline (it's whitespace they don't address). Don't try to beat their publishing loop.
2. **Lead with the two hardest-to-copy ADRs:** exact-math parity (ADR-001/010) and
   stability-under-code-change RNG (ADR-003) — both directly serve auditable re-certification.
3. **AI is UX, not the moat.** The durable edge is *AI authoring you can take to a regulator*
   (ADR-008) + provider-agnosticism.
4. **Clean-room only — hard guardrail.** Learn ideas/algorithms/formats; reimplement independently.
   **Never copy-then-reword their source to conceal provenance** — it's an MIT violation and
   acquisition IP scanners (the likely buyer is Stake) catch reworded OSS. If literal MIT code is
   ever reused, keep the attribution notice. Mine their public bug history as a free regression
   checklist.

## 3. Prioritized roadmap ("on steroids")

Ordered by impact-vs-effort and the wedge. Ranks 1–3 are the foundation; the rest build on them.

1. **Deterministic event stream (the art↔math bridge)** — ✅ *first slice landed*. A seed →
   `RoundOutcome` (reel stops, landed grid, per-line/scatter win detail with the exact cells to
   animate, integer money, cap event), replayable with no math re-run. `core-math/src/outcome.rs`,
   `simulate_one_traced`, `replay` command on the engine bridge. This is the physical artifact that
   connects the math to the Spine/atlas/spritesheet package.
2. **Fix line-eval correctness traps** — *partially landed*: `ways_243` now fails loud instead of
   silently paying zero; `both_ways` no longer double-pays a full-line win. **Remaining:** a real,
   exact `ways_243` (all-ways) implementation validated against a certified sheet.
3. **Reproducibility as CI invariants** — *partially landed*: `simulate_one == simulate_one_traced`
   over many spins is a test; a **golden replay vector** (exact `RoundOutcome` for a pinned
   config/seed/spin) and a **deterministic config fingerprint** are now enforced. **Remaining:** pin a
   reference MC accumulator; typed `NoFeasibleSolution` from the solver.
4. **`presentation.bind.json` sidecar** — ✅ *landed*: binds `symbol.id` + a closed event-type set →
   Spine skeleton / atlas region / spritesheet frames / sound cues; a bidirectional completeness
   check (every symbol and event must bind) **is** the "one package" guarantee; `validate_binding`
   also verifies an optional `targets_config_hash`. **Remaining:** a Studio UI to load Spine files and
   show green/red coverage; an optional non-math `presentation_ref` pointer on game-config.
5. **Content-addressed published-math artifact** — ✅ *first slice landed*: canonicalize config →
   **SHA-256** fingerprint (regulator-recognized; in `ai-layer/app/artifact.py`) + engine results +
   version stamps → a reproducible audit record; `POST /publish`, `POST /config-hash`. **Remaining:**
   integer-exact rational in the record (expose the engine's `line_ratio`); an `index.json` ledger.
6. **Scale exact capped RTP** — replace the `ENUM_LIMIT` cliff with win-value convolution/DP + a
   typed `Exact | ExactDp | NumericTailMC{ci}` boundary so large/Weights games keep an honest number.
7. **Solver objective DSL + volatility/money-back score** — target bands instead of opaque weights,
   surfaced in Slot Studio.
8. **Full GLI/BMM PAR-sheet exporter (M4)** — last: biggest effort, most prerequisites.

## 4. Watch-outs

- **Hot-path purity:** the MC inner loop must stay allocation-free — `simulate_one` keeps the fast
  path; `simulate_one_traced` is the (allocating) replay path; an equality test guarantees they agree.
- **Weights have no physical reel-stop** (independent-cell draws), so reel-stop replay is Strips-only;
  the event stream emits the landed grid for both, and tuned games realize to Weights.
- **Exact-math scaling** is our biggest claim *and* liability — document the exact-vs-MC-with-CI
  boundary clearly before any "exact" claim survives a lab.
- **ADR-009 float discipline** is not yet honored (everything renders via `as_f64`); the cert
  artifact needs a fixed-decimal render layer before any cross-machine digit-for-digit claim.
- Stable promotion still needs **external certified-game parity** (ADR-010) — pick the reference game.
