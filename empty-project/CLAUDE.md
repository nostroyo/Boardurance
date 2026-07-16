# empty-project — Boardurance frontend

This is the **web client** (the name is legacy). Vite + React 19 + TypeScript + Tailwind 3, React Router 7, Vitest + Testing Library, ESLint + Prettier.

## Commands (run from this directory)

- Dev server: `npm run dev` (Vite)
- Build: `npm run build` (runs `tsc -b` then `vite build`)
- Type-check only: `npx tsc --noEmit`
- Tests: `npm run test` (watch) or `npm run test -- --run` (one-shot, CI mode)
- Lint: `npm run lint` — Format: `npm run format` (check-only: `npm run format:check`)

First time / after dependency changes: `npm ci`.

## Verify loop (definition of "done")

Run both before considering a frontend change complete (mirrors CI's hard gates):

```
npx tsc --noEmit
npm run test -- --run
```

Lint and format are CI soft gates (`continue-on-error`), but run `npm run lint` and
`npm run format:check` and fix what you reasonably can before finishing.
