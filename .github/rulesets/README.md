# Rulesets

`main.json` protects the default branch (`main`). GitHub doesn't read this folder by itself:
import it once under **Settings → Rules → Rulesets → New ruleset → Import a ruleset**, and import it
again after changing it.

- **No deleting or force-pushing** `main`.
- **Changes land through a pull request** (no approvals required).
- **Required checks, on a branch that is up to date with `main`:**
  - **Release notes**: `VERSION` is raised and `notes/<version>.md` is well formed (`scripts/check-notes.mjs`).
  - **Go vet and tests**, **Smoke test**, **Go builds**, **Release scripts** and **Host neutral**: the CI jobs (`.github/workflows/ci.yml`). The tests, the smoke test and the neutrality check run on every merge request, so what the release job runs on main has passed before the merge; the others run only when their own files change, and a skipped job counts as passed.

  "Up to date" stops two pull requests from both claiming the same next version.

A merge to `main` then publishes `v<VERSION>` (`.github/workflows/release.yml`).
