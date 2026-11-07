---
name: eqwec-quality-control
description: Use before and after any EQwec product change, and before merges or releases, to run an independent quality gate. Trigger when reviewing a diff, validating tests, checking performance or security regressions, deciding PASS/FAIL/BLOCKED, preparing a merge, or preparing a release. Do not self-approve because the author says the work is done; require evidence.
---

# EQwec Quality Control

You are the independent reviewer. You may review your own work only when no separate reviewer is available, and then you must state that independence was limited and apply the same evidence standard.

## Gate model

Every gate returns one of:

- `PASS` — the required evidence exists and the criteria are met.
- `FAIL` — evidence exists and shows a defect, regression, or unmet criterion.
- `BLOCKED` — the check cannot run in this environment, or a mandatory prerequisite is missing.
- `NOT_APPLICABLE` — the check does not apply, with a written rationale.

Never convert `BLOCKED` into `PASS`. Never convert a skipped test into a pass. Never accept "it compiles" as a functional verification.

## Pre-change gate

Before implementation:

- Read the scope, branch, worktree state, and intended diff.
- Load `eqwec-engineering-core` and every affected product skill.
- Read the relevant ADRs, product spec, acceptance criteria, known defects, and baseline metrics.
- Identify requirement drift: is the requested change consistent with the approved architecture and product scope?
- Identify the affected products, shared crates, contracts, documents, skills, versions, and release metadata.
- Define the verification plan and the evidence that will prove completion.
- If the baseline is unknown for a performance claim, measure it or mark the claim `BLOCKED`.

## Implementation gate

Inspect the actual code, not only the summary:

- Public API and contracts: is the change backward compatible or explicitly versioned?
- Logic: are edge cases, empty inputs, limits, and error paths handled?
- Failure handling: does it fail closed where required, and avoid partial writes or replay?
- Rust safety: ownership, lifetimes, `unsafe`, FFI, concurrency, cancellation, lock-across-await, bounded allocation.
- Security boundaries: untrusted input, path safety, secrets, permissions, IPC validation, dependency changes.
- Resource lifetimes: files, processes, GPU resources, threads, subscriptions, temp files.
- Documentation: are the impacted documents and skills updated or explicitly assessed?
- Real functionality: is the feature implemented, or is it a mock, placeholder, or hard-coded demo?
- Scope: is the diff limited to the stated task, with no unrelated refactor or cross-product contamination?

## Verification gate

Run the repository's actual commands. Discover them from `AGENTS.md`, CI, or the build files; do not invent them. Record the exact command and exit code.

Expected categories, once the workspace exists:

- Formatting.
- Static analysis and lints.
- Compilation for the supported targets.
- Unit tests.
- Integration and contract tests.
- End-to-end workflow on a real fixture.
- Property, fuzz, and adversarial input tests where applicable.
- Security checks: advisories, licenses, secret scanning, signature tests.
- Platform-specific checks where the change is platform-sensitive.

Then test a real functional workflow, not only isolated units. For example, install and launch a package, edit and reopen a document, or record and render audio, according to the affected product skill.

For each check, record `PASS`, `FAIL`, `BLOCKED`, or `NOT_APPLICABLE`, with evidence.

## Performance gate

- Compare representative measurements against a relevant baseline.
- Record hardware, OS, build profile, workload, command, target, observed result, and regression threshold.
- Investigate regressions before accepting them.
- Check allocation and copy hotspots, UI responsiveness, startup, shutdown, memory, and binary/install size where relevant.
- Do not accept a synthetic microbenchmark as proof of a real-workload improvement.
- If the measurement cannot be reproduced, mark the performance claim `BLOCKED` and remove it from release notes.

## Merge gate

- Inspect commit scope and Ukrainian Conventional Commit messages.
- Verify branch provenance and that the base is current.
- Verify the intended diff and that unrelated files are not staged.
- Verify conflict resolution preserved both sides' contracts.
- Verify dependency and shared-crate impact.
- Verify documentation, skills, versions, and release metadata are updated as needed.
- Reject cross-product contamination.
- A critical unresolved defect or missing essential gate blocks the merge.

## Release gate

- Independent product version and changelog are correct.
- Compatibility and dependency impact are assessed.
- Installers/packages build reproducibly.
- Integrity and signature verification pass.
- Install, launch, update, rollback, repair, and uninstall are verified as applicable.
- Supported platform checks pass.
- No unresolved critical security finding exists.
- Release notes are accurate and do not describe a prototype as stable.
- Authorization to publish exists.
- The released commit set and rollback plan are recorded.

## Findings

Report every finding with:

- Severity: `critical`, `high`, `medium`, `low`.
- Exact evidence: file, line, command, output, or reproduction.
- Impact: what can go wrong, for whom, and under what conditions.
- Remediation: the smallest effective fix.
- Verification state: fixed, open, blocked, or waived.

A critical finding blocks merge and release. A high finding blocks release unless an authorized waiver exists.

## Waivers

A waiver requires:

- Explicit authorization from the accountable owner.
- The reason the gate cannot be met now.
- The exact scope and affected versions.
- An expiration date or review trigger.
- The residual risk and the compensating control.

Critical security controls must not be waived merely to accelerate delivery. A waiver is not a `PASS`; it is a documented, time-bounded exception.

## Skill usage record

For each substantial task, record a matrix:

| Skill | State | Evidence |
|---|---|---|
| `eqwec-engineering-core` | APPLIED | Rules referenced, diff checked |
| `eqwec-quality-control` | APPLIED | Gates and commands run |
| `<product skill>` | APPLIED / AVAILABLE_NOT_RELEVANT / BLOCKED | Domain check or reason |
| External skill | APPLIED / AVAILABLE_NOT_RELEVANT / BLOCKED | Artifact or reason |

A missing mandatory skill is `BLOCKED`, not `NOT_RELEVANT`. A checker can verify that files exist; it cannot prove that the instructions were followed.

## Report

Use `references/report-template.md`. Keep the report factual and distinguish executed checks from unavailable and planned checks.

## Handoff

- Hand off to `eqwec-engineering-core` for policy conflicts.
- Hand off to the affected product skill for domain-specific fixtures and failure modes.
- If a product skill conflicts with the core security, versioning, or Git rules, the core rule wins and the product skill is defective.
