# Documentation Lifecycle

## Priority order

1. `README.md` — mission, products, architecture summary, status, supported platforms, real documentation links.
2. `AGENTS.md` — canonical AI-agent entry point: facts, constraints, scope, skill activation map, workflow, test commands, safe Git rules.
3. `CLAUDE.md` — Claude-specific instructions aligned with `AGENTS.md`; no contradictory duplicate rules.
4. Product specs, architecture, ADRs, security, performance, development, and research documents under `docs/`.

## Branch constraint

`main` contains only `README.md`. `AGENTS.md`, `CLAUDE.md`, `.agents/skills/`, and technical documents live on `integration`, `dev-*`, and eligible `production` releases. Keep the `main` README synchronized selectively through a README-only update; never merge a whole development tree into `main`.

## When to update

Update an impacted document when a change affects:

- Product scope, status, supported platforms, or user-visible behavior.
- Architecture, crate ownership, public API, or a shared contract.
- Security boundary, threat model, permissions, signing, or data handling.
- Performance budget, benchmark method, or a measured result.
- Build, test, packaging, release, branch, or versioning policy.
- Skill triggers, responsibilities, or handoff rules.
- A command, path, dependency, or version that the document states as fact.

Do not rewrite every Markdown file on every commit. Do not update a document cosmetically to create the appearance of activity.

## Truth rules

- Do not describe a planned feature as implemented.
- Do not claim a performance improvement without a reproducible measurement.
- Do not create a link that does not resolve.
- Do not state a command that has not been validated in the current environment.
- Do not state a version, platform support level, or release status that is not in the repository metadata.
- Mark superseded instructions clearly; preserve historical ADRs and changelogs.

## Review checklist

- Does `README.md` still describe the actual project state?
- Does `AGENTS.md` name the current branches, commands, skills, and constraints?
- Does `CLAUDE.md` agree with `AGENTS.md` and link to the canonical rule instead of duplicating it?
- Are relative links valid?
- Are code fences, tables, and headings well formed?
- Did the change affect a product skill? If yes, was its trigger or domain content updated and re-tested?
- Is the version/release impact recorded?
- Is there a justified no-change decision for an impacted document?

## Documentation debt

When a document is stale but cannot be corrected in the current change, record the gap in the task report with the affected path, the reason, and the follow-up owner. Do not silently leave a false statement.
