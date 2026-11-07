# CLAUDE.md — EQwec Claude Entry Point

`AGENTS.md` is the canonical agent rulebook for this repository. Read it first and follow it. This file adds only Claude-specific loading notes and must never contradict `AGENTS.md`.

## Required reading order

1. `AGENTS.md` — project facts, branch model, skills gate, workflow, security, definition of done.
2. `.agents/skills/eqwec-engineering-core/SKILL.md` — global engineering rulebook.
3. `.agents/skills/eqwec-quality-control/SKILL.md` — independent gate model.
4. The skill for each affected product under `.agents/skills/eqwec-<product>/SKILL.md`.
5. The relevant ADR, product spec, and security/performance document when it exists.

## Skill loading

- Project skills live in `.agents/skills/` and are version-controlled on `integration`, `dev-*`, and eligible `production` releases. They are not on `main`.
- Load a skill through the harness's skill mechanism when available; do not merely cite its name.
- `eqwec-engineering-core` applies to every task. `eqwec-quality-control` applies before and after every change and before merge or release.
- Load only the product skills that the task actually touches. A missing mandatory skill is `BLOCKED`, not `NOT_RELEVANT`.
- External skills are installed under `~/.agents/skills/`. Their provenance, versions, and limitations are recorded in `docs/development/SKILLS-AUDIT.md` and `docs/development/SKILLS-LOCK.json`.
- `skill-creator` is the authority for authoring or changing EQwec skills. Validate with `quick_validate.py` and record trigger evidence.

## Repository constraints

- Current phase: Phase 3 Hub MVP at `0.0.0`. The verification core, durable-state primitives, activation engine, package staging, and a non-production in-process fixture workflow exist; no installer, launcher UI, or usable product exists.
- Do not create applications for other products or add product features outside the Hub MVP. Shared infrastructure changes still require an owner, tests, and an impacted-document update.
- `main` contains only `README.md`. Never add skills, `AGENTS.md`, `CLAUDE.md`, or technical documents to `main`.
- Work on `integration` or an appropriate scoped branch. Inspect the worktree before switching, staging, merging, or rebasing.
- Never force-push or rewrite shared history without explicit authorization.
- Use Conventional Commits with Ukrainian imperative descriptions, for example `docs(skills): додати перевірку тригерів`.

## Evidence standard

- Run real commands and record exit codes. Mark unavailable checks `BLOCKED`; never report them as `PASS`.
- Do not claim a capability is implemented because code or documentation was generated.
- Do not invent commands, paths, versions, benchmarks, or test results.
- Apply `eqwec-quality-control` honestly, including the `SKILL_USAGE` matrix.

## Current preflight

```bash
python3 scripts/verify-skills.py

# Official schema validation (requires PyYAML in the validation environment):
python3 -m venv .venv-skillcheck
.venv-skillcheck/bin/pip install -r scripts/skill-validation-requirements.txt
.venv-skillcheck/bin/python scripts/verify-skills.py \
  --validator ~/.agents/skills/skill-creator/scripts/quick_validate.py
```

The Cargo workspace exists. The verified local commands are `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo build --workspace --all-targets --locked`, and `cargo test --workspace --all-features --locked`. CI is configured on GitHub Actions; the first run is pending.

If this file and `AGENTS.md` disagree, `AGENTS.md` wins. Report the conflict and fix this file.
