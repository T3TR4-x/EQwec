---
name: eqwec-engineering-core
description: Use for every EQwec engineering task, including architecture, Rust implementation, reviews, documentation, Git, versioning, security, and cross-product changes. Trigger when working in the EQwec repository, when a change affects EQwec product boundaries, shared crates, release policy, or agent instructions, or when validating an EQwec claim. Do not use for unrelated repositories.
---

# EQwec Engineering Core

This is the project-wide rulebook. It applies before a product skill, and a product skill may add domain constraints but may not weaken this one.

## Core principle

Implement the simplest architecture that correctly satisfies the requirement, remains extensible, and has measurable correctness, security, and performance guarantees. Implementation volume is not engineering quality.

## Start every task

1. Read the repository's current `AGENTS.md`, `README.md`, and the relevant architecture decision records (ADRs). If they disagree with the code, stop and report the drift.
2. Load `eqwec-quality-control` before changing anything and again before claiming completion.
3. Load the skill for every affected product. A product skill is mandatory when its paths, crates, formats, or user workflows are touched.
4. Inspect the current branch, worktree status, recent commits, and the actual diff before staging or switching branches.
5. Identify the affected products, shared crates, contracts, documentation, skills, versions, and release metadata.
6. Establish the acceptance criteria and, where performance is claimed, a baseline measurement.
7. If a mandatory precondition is missing, stop with a diagnostic instead of inventing a result.

## Repository and ownership map

The approved default is a Rust-first modular monorepo with a Cargo workspace. The exact paths and crate names must match `docs/architecture/SYSTEM-ARCHITECTURE.md` and `docs/architecture/DEPENDENCY-MAP.md` once those documents exist. Until then, treat this map as planned and verify it against the repository:

- `apps/eqwec-hub`, `apps/eqwec-browser`, `apps/photocraft`, `apps/vectorcraft`, `apps/filmcraft`, `apps/effectcraft`, `apps/craftanything`, `apps/audiocraft` — independently buildable product entry points.
- `crates/eqwec-core` — shared domain primitives, typed identifiers, error and result conventions.
- `crates/eqwec-ui` — shared design tokens and UI primitives, not product workflow logic.
- `crates/eqwec-config` — configuration loading, validation, and migration.
- `crates/eqwec-storage` — tenant-scoped persistence, atomic writes, project storage.
- `crates/eqwec-logging` — structured diagnostics with secret redaction.
- `crates/eqwec-updater` — signed manifests, package verification, staged install, rollback.
- `crates/eqwec-security` — cryptographic verification, path safety, permission checks.
- `crates/eqwec-ipc` — versioned inter-process contracts with explicit trust boundaries.
- `crates/eqwec-graphics` — GPU device/resource management and rendering abstractions.
- `crates/eqwec-media` — bounded decode/encode and media pipeline primitives.

Do not create an empty crate merely to match this list. Create a crate when it has a real owner, API, tests, and a second consumer or a documented architectural reason.

Every query, file path, cache key, and IPC message that belongs to a user or tenant must carry an explicit tenant or profile scope. Never rely on a global default when the data is tenant-owned.

## Non-negotiable invariants

- Correctness and user-data integrity come before feature volume.
- Security and privacy are design inputs, not a review afterthought.
- Untrusted input is parsed with bounded resources and validated before use.
- Secrets, tokens, passwords, private keys, and prompts are never logged, returned, committed, or placed in fixtures.
- A truncated stream, partial write, or unverified package is a failure, never a success.
- Authentication, signature, and credential-store failures fail closed.
- A non-critical diagnostic or telemetry failure must not stop an authorized local workflow.
- No replay or duplicate side effect after a response is committed.
- Every non-trivial behavior has a regression test before the fix.
- A performance claim requires a reproducible baseline, workload, environment, and result.
- A feature is not complete because code was generated; it is complete when its acceptance workflow passes.

## Rust standards

- Use a stable, pinned toolchain from `rust-toolchain.toml`.
- Prefer `#![forbid(unsafe_code)]` in crates that do not need FFI or low-level native access.
- Isolate `unsafe` in a dedicated boundary. Document every safety invariant, pointer ownership rule, lifetime assumption, and thread-safety requirement.
- Do not use `unsafe` as a performance shortcut without measurements and a documented reason.
- Use explicit ownership and lifetimes. Shared state is `Arc` only when sharing is real; mutation is scoped to its owner.
- Do not hold a lock across an `await`. Prefer message passing or short critical sections.
- Use typed domain models and typed errors. Do not pass untyped maps or strings across a module boundary when a type can express the contract.
- Propagate `Result`; avoid `unwrap()` and `expect()` on recoverable production paths. If an invariant makes a panic correct, document it and test it.
- Bound allocations, queue lengths, parser depth, and cache growth. Define eviction and backpressure.
- Make cancellation and shutdown explicit. A cancelled operation must not leave a half-written project or a leaked process.
- Introduce an async runtime only where I/O concurrency or responsiveness justifies it. Do not put an async runtime into a latency-sensitive audio or render callback.
- Keep dependencies minimal and justified. Record the reason, license, maintenance status, and replacement cost in the dependency map.
- Run the repository's format, lint, test, and security commands. Treat warnings as failures where the repository's CI does.
- Do not suppress a lint broadly. If an exception is necessary, scope it narrowly and explain why.

See `references/rust-style.md` for the detailed rules and examples.

## Security and privacy baseline

- Maintain a threat model for local data, downloaded packages, updates, browser sessions, untrusted documents, plugins, IPC, native libraries, and release infrastructure.
- Validate paths against traversal, absolute-path injection, symlink escapes, and platform-specific separators.
- Create temporary files with restrictive permissions and unpredictable names; clean them on failure.
- Verify signatures and integrity before executing or installing a downloaded artifact.
- Use safe deserialization and bounded parsing. Reject unknown or inconsistent fields when the format contract requires it.
- Apply least privilege and preserve platform sandboxing. Do not disable a security control to reduce memory, improve startup, or simplify packaging.
- Keep signing keys out of the repository. Document key custody, rotation, and revocation.
- Provide explicit user control over diagnostics and telemetry. Never collect browsing history, project contents, personal files, audio recordings, credentials, or prompts by default.

See `references/security-baseline.md` and `docs/security/THREAT-MODEL.md` when it exists.

## Documentation lifecycle

`README.md`, `AGENTS.md`, and `CLAUDE.md` are living documents, not release decorations.

- `README.md` describes the current mission, products, architecture, status, supported platforms, and real documentation. Do not describe planned capabilities as shipped.
- `AGENTS.md` is the canonical AI-agent entry point: current facts, constraints, scope, skill activation map, workflow, test commands, and safe Git rules.
- `CLAUDE.md` is Claude-specific and must not contradict `AGENTS.md`. Link to the canonical rule instead of duplicating it.
- `main` contains only `README.md`. `AGENTS.md`, `CLAUDE.md`, skills, and technical documents belong on `integration`, `dev-*`, and eligible `production` releases.
- On a significant code, architecture, API, Git, skill, or release change, inspect the impacted Markdown and update the factual sections, or record why no change is needed.
- Do not rewrite every Markdown file on every commit. Update impacted documents only.
- Do not create fake links, commands, version numbers, or capabilities. Relative links must resolve.
- Keep historical ADRs and changelogs; mark superseded instructions instead of deleting history.

See `references/documentation-lifecycle.md` for the review checklist.

## Git governance

Branch roles:

- `main` — informational, `README.md` only.
- `integration` — shared workspace, shared crates, policies, skills, and cross-product contracts.
- `dev-<product>` — long-lived product workstreams.
- `feature/*`, `fix/*`, `debug/*`, `perf/*`, `security/*`, `docs/*`, `release/*`, `hotfix/*` — short-lived scoped work.
- `production` — reviewed release source and release metadata.

Rules:

- Inspect the worktree and index before staging, switching, merging, rebasing, or cherry-picking. Protect pre-existing uncommitted changes.
- Commit only a coherent logical change. Do not stage unrelated files blindly.
- Use Conventional Commits with a Ukrainian imperative description: `feat(hub): додати перевірку підпису пакетів`. Keep the type and scope tokens in English; write the subject and body in Ukrainian. Use `BREAKING CHANGE:` only when the standard requires that exact token.
- Amend only the immediately preceding local, unpushed commit when it belongs to the same logical task, no one depends on the old SHA, and the amended result passes verification. Never amend `production`, `integration`, or shared `dev-*` history.
- Never force-push or rewrite shared history without explicit authorization.
- Do not run `reset --hard`, destructive `clean`, unconditional checkout overwrite, or blanket deletion of lock files.
- Resolve conflicts by understanding both sides and preserving contracts. Never use blanket `-X ours` or `-X theirs`.
- Before merging, verify the base, upstream, intended diff, scope, and gate results. After merging, verify the resulting tree, impacted tests, branch state, and absence of accidental files.
- For a product release, create `release/<product>/vX.Y.Z` from current `production`, promote only reviewed product changes and required tested dependencies, verify that unrelated work did not enter, then merge with protected review. Reconcile back into `integration` and affected `dev-*` branches.

See `references/git-workflow.md` for the detailed procedure and conflict log format.

## Versions and releases

- Each product has an independent `MAJOR.MINOR.PATCH` version. Initial planning state is `0.0.0`.
- Use prerelease identifiers for early milestones: `0.0.1-alpha.1`, `0.1.0-beta.1`, `0.1.0-rc.1`.
- Do not bump a product merely because a shared crate changed. Assess dependency and compatibility impact first.
- Maintain one canonical version source per product and synchronize Cargo package version, release manifests, update metadata, changelogs, and tags through verified tooling.
- A release requires proven compatibility, changelog, tests, packaging and integrity checks, a release commit and tag, and authorization to publish.
- Use namespaced tags such as `eqwec-hub/v0.1.0`. Prefer signed tags when the infrastructure supports them.
- Never describe an alpha or prototype as stable.

See `references/versioning.md`.

## Cross-product contracts

- A shared crate has an owner, a public API, compatibility expectations, and tests that exercise every consumer.
- A product may depend on a shared crate; a shared crate must not depend on a product.
- Cross-application document exchange must preserve canonical ownership: one product owns the document format, and other products consume an explicit export or a documented interchange contract.
- IPC contracts are versioned, validated, and tested for malformed input and version mismatch.
- A change to a shared contract requires an impact assessment naming every affected product and a cross-product test.
- Do not couple products through undocumented files, global mutable state, or an implicit current-directory convention.

## Workflow

1. **Understand** — restate the requirement and acceptance criteria; identify unknowns and blockers.
2. **Inspect** — read the existing code, tests, ADRs, and the actual branch state.
3. **Design the smallest correct change** — prefer deletion, reuse, and standard-library solutions before new abstractions.
4. **Implement** — keep the diff scoped; add the regression test with the fix.
5. **Verify** — run the repository's real commands and record exit codes. Mark unavailable checks `BLOCKED`, not passed.
6. **Review** — apply `eqwec-quality-control`; resolve critical and high findings before merge.
7. **Document** — update only impacted documents and skills; record version and release impact.
8. **Commit** — stage the intended files and write a scoped Ukrainian Conventional Commit.
9. **Integrate** — merge into the correct `dev-*` branch, then `integration` with contract and impact checks.
10. **Release** — only after the release gate, packaging, signing, and authorization.

## Handoff

- Hand off to `eqwec-quality-control` for pre-change, implementation, verification, performance, merge, and release gates.
- Hand off to the affected product skill for domain invariants, fixtures, failure modes, and cross-platform caveats.
- If a product skill conflicts with this skill, this skill wins and the conflict must be recorded as a skill defect.
