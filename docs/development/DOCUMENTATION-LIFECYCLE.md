# Documentation Lifecycle

**Status:** approved policy. It defines what is authoritative, when it must change, and how to avoid stale or contradictory documentation.

## 1. Priority order

1. `README.md` — mission, products, architecture summary, real status, supported platforms, links to real documentation.
2. `AGENTS.md` — canonical AI-agent entry point.
3. `CLAUDE.md` — Claude-specific entry point; defers to `AGENTS.md`.
4. Skills in `.agents/skills/` — procedural rules for engineering and products.
5. Architecture, security, performance, research, and development documents.
6. Product specs and release records.

A lower-priority document must not contradict a higher-priority one. If it does, the higher-priority rule wins and the contradiction is a defect.

## 2. Document classes

| Class | Examples | Authority | Update trigger |
|---|---|---|---|
| Public status | `README.md` | Project state | Product status, platform, release, architecture summary changes |
| Agent policy | `AGENTS.md`, `CLAUDE.md`, this file | Agent behavior | Git, skill, workflow, security, or scope changes |
| Skills | `.agents/skills/*/SKILL.md` | Procedural engineering rules | Product domain, architecture, tooling, or defect changes |
| Research | `docs/research/*` | Evidence and feasibility | New evidence, changed upstream facts, decision review |
| Architecture | `docs/architecture/*` | Approved system design | Architecture decision or contract change |
| ADR | `docs/adr/*` | Decision record | New decision; supersede, never delete |
| Security | `docs/security/*` | Threat model and baseline | New trust boundary, finding, or control change |
| Performance | `docs/performance/*` | Budgets and methodology | New baseline, workload, or regression |
| Development | `docs/development/*` | Process and gates | Workflow, branch, version, test, or release change |
| Product spec | `docs/product-specs/*` | Product scope and acceptance | Scope, format, workflow, or release change |
| Release record | Release manifest/changelog | Shipped artifact | Every release |

## 3. Rules

- Describe the **actual** state. Planned work is labeled planned; future goals are not presented as implemented capabilities.
- Do not create fake links, commands, version numbers, or benchmarks.
- Verify commands in the current environment before documenting them. If a command cannot run, label it `BLOCKED`.
- Keep relative links valid. Re-check links when moving files.
- Do not rewrite every Markdown file on every commit. Update impacted documents only.
- Do not maintain competing copies of the same rule. Keep one authoritative location and link to it.
- Preserve historical ADRs, changelogs, and release records. Mark superseded instructions instead of deleting history.
- Remove or mark obsolete instructions that could mislead an agent; do not leave contradictory guidance in place.
- Keep `main` README-only. Development documents belong on `integration`, `dev-*`, and eligible release branches.

## 4. Update triggers

Update the impacted documentation when a change affects:

- Product scope, user workflow, supported format, or acceptance criteria.
- Architecture, crate boundary, public API, IPC contract, or shared dependency.
- Security boundary, threat, control, permission, or secret handling.
- Performance budget, workload, benchmark method, or observed regression.
- Git workflow, branch role, version policy, test command, or release process.
- Skill trigger, domain rule, or mandatory check.
- Supported platform, installer, signing, or update behavior.
- Public status in `README.md`.

If no document changes, record the reason in the task's quality report. "No docs needed" is a decision that must be justified, not an assumption.

## 5. Review checklist

Before committing documentation:

- [ ] Facts match the repository, not an earlier plan.
- [ ] Planned and implemented states are clearly separated.
- [ ] No invented commands, paths, versions, or results.
- [ ] Relative links resolve.
- [ ] Markdown renders correctly.
- [ ] `AGENTS.md` and `CLAUDE.md` do not contradict each other.
- [ ] Impacted skills were reviewed and revalidated.
- [ ] Version and release metadata are accurate.
- [ ] No secrets, private keys, personal paths, or sensitive data.
- [ ] The change is on the correct branch and `main` remains README-only.

## 6. ADR lifecycle

```text
PROPOSED -> ACCEPTED -> SUPERSEDED
                    \-> REJECTED
```

- An ADR records context, problem, alternatives, evidence, decision, consequences, security/performance/licensing impact, and reconsideration criteria.
- Accepted ADRs are normative until superseded.
- Never delete an ADR. A new ADR supersedes it and links both ways.
- An ADR that depends on unverified facts is `PROPOSED`, not `ACCEPTED`.

## 7. Changelog and release notes

- A changelog entry describes user-visible and compatibility-relevant changes, not every commit.
- Release notes must match the shipped artifact, supported platforms, known limitations, and version.
- Do not describe a prototype or alpha as stable.
- Security fixes are described accurately without disclosing exploit details before users can update.

## 8. Current documentation state

| Document | State |
|---|---|
| `README.md` | Current; bootstrap and planning status |
| `AGENTS.md`, `CLAUDE.md` | Current; skills gate closed |
| `docs/development/SKILLS-AUDIT.md` | Current; `SKILLS_GATE = PASS` for bootstrap scope |
| `docs/development/SKILLS-LOCK.json` | Current; generated from the working tree |
| `docs/development/SKILL-USAGE.md` | Current for the bootstrap task |
| Research, architecture, security, performance, product specs | Not yet written; planning phase in progress |

Missing planning documents are not "empty placeholders"; they must be written from real research and reviewed before the related gate can pass.
