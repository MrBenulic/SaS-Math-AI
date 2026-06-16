# frontend — Slot Studio (Module 1)

Next.js config UI for SaS-Math AI. **M3 implemented** (on `beta`, tagged `v0.4.0-beta.2`).

The browser-facing studio: describe a slot in natural language → get a validated JSON config (shown
in **Monaco**) → **simulate** and **auto-tune** it. The UI only talks to the FastAPI `ai-layer`; the
ai-layer calls the Rust engine. The math never runs in the browser.

## Stack
Next.js 16 (App Router) · React 19 · Tailwind v4 · Zustand · `@monaco-editor/react`.

## Layout
```
frontend/
├─ app/page.tsx       the Slot Studio (prompt → Monaco → simulate/tune)
├─ lib/api.ts         typed client for the ai-layer (/translate /simulate /tune)
├─ store/useStore.ts  Zustand state + actions
└─ .env.local         NEXT_PUBLIC_API_URL (defaults to http://127.0.0.1:8000)
```

## Run (needs the ai-layer + Rust engine)
```bash
# 1. Rust engine
(cd ../core-math && cargo build --release --bin engine)
# 2. ai-layer API (separate terminal, with its venv)
(cd ../ai-layer && .venv/bin/uvicorn app.main:app --port 8000)
# 3. this app
npm install && npm run dev      # http://localhost:3000
```

## Verified (M3 gate)
Prompt *"Make a 5x3 slot, 20 paylines, 96% RTP, medium volatility, with wild and scatter"* →
Translate → Simulate (149.19% RTP) → **Auto-tune → 96.00%** (realize-delta 0.01%, 1680 solver evals),
all in the browser. Next: schema→TS codegen, Playwright E2E, persistence.
