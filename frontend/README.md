# frontend (Next.js) — Module 1

The designer-facing configuration UI. **Not yet implemented** — built in **M3** (`v0.4.0`).

## Stack
Next.js (React) · Tailwind CSS · Zustand (state) · Monaco editor (raw JSON view/tweak).

## Principles
- The **form is the source of truth**; it is **generated from the schema** (ADR-004), so UI and
  backend validation never drift.
- **Lock hard boundaries** in the UI (e.g. "max win = 5000×") so the solver always gets a feasible
  target.
- The natural-language prompt is a convenience that produces the *same* JSON via the ai-layer.

## Planned layout
```
frontend/
├─ package.json
├─ app/                 Next.js app router
├─ components/          schema-driven form, Monaco JSON panel, results/analytics views
├─ lib/types.ts         TypeScript types generated from schema/game-config.schema.json
├─ store/               Zustand stores
└─ e2e/                 Playwright tests (blueprint-mandated)
```
