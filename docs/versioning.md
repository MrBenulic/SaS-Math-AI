# Versioning, Branching & Rollback

The whole point: **we can always roll back to a known-good checkpoint.**

## Branches
- **`main`** — *stable*. Always releasable. Only updated by promoting `beta` after its milestone
  gate passes. Every commit on `main` that ships is tagged `vX.Y.Z`.
- **`beta`** — *active integration*. All milestone work lands here first. Snapshots tagged
  `vX.Y.Z-beta.N`.
- **`feature/*`** — short-lived branches off `beta`, merged back via PR.

```
feature/* ──PR──▶ beta ──(gate passes)──▶ main ──tag vX.Y.Z──▶ GitHub Release (Latest)
                   │
                   └── tag vX.Y.Z-beta.N ──▶ GitHub Release (Pre-release)
```

## SemVer (pre-1.0 convention here)
- **MINOR** (`0.X.0`) = a completed milestone (see `roadmap.md`).
- **PATCH** (`0.x.Y`) = fixes/tweaks within a milestone.
- **`-beta.N`** = pre-release snapshot on `beta`.
- `1.0.0` = M4 commercial-grade prototype.

## Releasing a stable version (end of a milestone)
```bash
git checkout beta && <run the milestone gate / tests>
git checkout main && git merge --no-ff beta -m "release: vX.Y.0 (Mn)"
git tag -a vX.Y.0 -m "Mn: <summary>"
git push origin main --tags
gh release create vX.Y.0 --title "vX.Y.0 — Mn" --notes "<highlights>" --latest
git checkout beta && git merge main          # keep beta current
```

## Cutting a beta snapshot
```bash
git checkout beta
git tag -a vX.Y.0-beta.N -m "beta: <what's in it>"
git push origin beta --tags
gh release create vX.Y.0-beta.N --title "vX.Y.0-beta.N" --notes "<notes>" --prerelease
```

## Rolling back
- **Inspect any checkpoint:** `git checkout vX.Y.Z` (detached) or download its GitHub Release zip.
- **Revert `main` to a known-good stable tag** (safe, keeps history):
  ```bash
  git checkout main
  git revert --no-commit <bad>..HEAD && git commit -m "revert: back to vX.Y.Z"
  # or, to hard-reset a not-yet-shared mistake:  git reset --hard vX.Y.Z
  ```
- **Always** tag a stable point *before* a risky change (per `CLAUDE.md`), so the rollback target
  exists.

## Conventions
- **Conventional Commits**: `feat: …`, `fix: …`, `docs: …`, `refactor: …`, `test: …`, `chore: …`.
- Commit trailer: `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.
- One logical change per commit; keep `main` green.
