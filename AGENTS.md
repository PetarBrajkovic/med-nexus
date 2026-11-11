# MedNexus — agent instructions

Read `PLAN.md` (scope) and `docs/adr/` (decisions) first. The human hand-codes `backend/` (Rust) and `ai-service/` (Python): **never write code there**. Review and teach instead. The agent writes only `frontend/` (plain HTML/CSS/JS).

## Agent skills

### Issue tracker

Issues live on GitHub (`PetarBrajkovic/med-nexus`), driven by the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Domain docs

Single-context: `GLOSSARY.md` (created lazily) + `docs/adr/` at the repo root.
