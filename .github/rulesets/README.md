# Rulesets

`main.json` protects the default branch (`main`). GitHub doesn't read this folder by itself:
import it once under **Settings → Rules → Rulesets → New ruleset → Import a ruleset**, and import it
again after changing it.

- **No deleting or force-pushing** `main`.
- **Changes land through a pull request** (no approvals required).
- **Required checks, on a branch that is up to date with `main`:**
  - **Release notes**: `VERSION` is raised and `notes/<version>.md` is well formed (`scripts/check-notes.mjs`).
  - **Vet, tests and builds**: the main CI job (`.github/workflows/ci.yml`).

  "Up to date" stops two pull requests from both claiming the same next version.

A merge to `main` then publishes `v<VERSION>` (`.github/workflows/release.yml`).
