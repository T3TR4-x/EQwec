# Test Strategy

**Status:** approved strategy. No test harness exists yet; all Rust test commands are `BLOCKED` until the Cargo workspace is created.

## 1. Principles

- Test behavior and contracts, not implementation details.
- Every non-trivial bug fix gets a regression test before the fix is accepted.
- Test complete workflows, not only isolated units.
- Use deterministic fixtures with clear licensing and no personal data.
- A passing unit test or successful compile is not proof of a functional workflow.
- Do not suppress, skip, or weaken a failing test to obtain a green build.
- Record the exact command, environment, fixture, and result. Mark unavailable checks `BLOCKED`.

## 2. Test levels

| Level | Purpose | Examples |
|---|---|---|
| Unit | Pure logic and invariants | Path math, manifest parsing, color conversion, state transitions |
| Property | General rules over generated inputs | Geometry predicates, parser bounds, serialization round-trips |
| Integration | Component contracts and I/O | Storage atomicity, IPC validation, media decode, plugin isolation |
| Contract | Shared-crate and cross-product compatibility | Manifest schema, interchange package, IPC versioning |
| End-to-end | Real user workflow | Install/update/rollback, edit/save/reopen/export, record/render |
| Regression | Previously fixed defects | One test per confirmed bug |
| Performance | Budgets and regressions | Startup, memory, frame time, export throughput, xrun count |
| Security | Trust boundaries and adversarial input | Signature failure, traversal, malformed media, fuzz corpora |
| Recovery | Interruption and corruption | Kill during save/update/render, corrupt cache/journal |
| Compatibility | Platforms and formats | Windows/macOS/Linux, x86_64/ARM64, SVG/media variants |

## 3. Product workflow tests

Each product skill defines the required end-to-end workflows and fixtures. The test strategy adopts them as acceptance tests:

| Product | Minimum workflow evidence |
|---|---|
| Hub | Install, verify, launch, update, interrupt, recover, repair, roll back, uninstall, offline install |
| Browser | Create workspace/profile, open tabs, restart, restore session, download, permission isolation, memory measurement |
| PhotoCraft | Open image, edit layers/masks non-destructively, save, reopen, export |
| VectorCraft | Create/edit paths, boolean operations, save, reopen, SVG import/export, hostile SVG rejection |
| FilmCraft | Import media, edit timeline, save, reopen, render playable output, A/V sync |
| EffectCraft | Build graph, keyframe, cache, render, interchange with FilmCraft |
| CraftAnything | Open asset in canonical owner, preserve provenance, resolve conflict, export/return |
| AudioCraft | Record/import, arrange, automate, save, reopen, render without unacceptable audio errors |

The product skill is authoritative for the detailed success criteria. This table is a planning summary, not a replacement.

## 4. Fixtures and data

- Fixtures are repository-owned, deterministic, license-clean, and small enough for CI.
- Do not use personal photos, private audio, user documents, or unknown-license fonts.
- Test signing keys are non-production and never shipped.
- Malformed and hostile fixtures are isolated from normal fixtures and clearly labeled.
- Large media fixtures use generated or public-domain sources; record provenance.
- Fixture hashes are recorded where reproducibility matters.

## 5. Determinism and flakiness

- Control time, randomness, locale, timezone, display scale, and GPU availability where possible.
- A flaky test is a defect. Quarantine it only with an owner, an issue, and a deadline; do not delete it silently.
- Native UI tests must tolerate platform timing differences without weakening assertions.
- Performance tests require a stable machine profile and a recorded baseline; CI variance must be measured before a threshold is enforced.

## 6. Native UI and web content

- Playwright applies to web content and browser-based workflows, not to native desktop chrome. Do not claim it tests a native app.
- Native UI automation depends on the selected GUI framework and platform; the technology-selection ADR must identify a real, testable path.
- Prefer testable command/state layers beneath the UI, with a smaller number of platform UI smoke tests.
- Accessibility checks include keyboard navigation, focus order, screen-reader labels where the framework exposes them, and contrast.

## 7. Security testing

- Fuzz parsers for manifests, project files, SVG, media containers, and IPC.
- Test signature verification with wrong key, revoked key, expired metadata, replay, downgrade, and tampered bytes.
- Test path traversal, symlink escape, archive bombs, and unsafe deserialization.
- Run dependency advisory and license checks when the workspace exists.
- A clean automated scan is not proof of security; pair it with threat-model review and targeted tests.

## 8. Performance testing

- Define workloads and budgets per product before optimizing.
- Record hardware, OS, build profile, command, baseline, target, observed result, and regression threshold.
- Test cold/warm startup, idle/active memory, CPU/GPU utilization, frame time, export throughput, audio xruns, disk footprint, and install size where relevant.
- Use release builds for release claims. Debug-build numbers are not release evidence.
- Do not compare incompatible memory metrics across operating systems.

## 9. CI plan (not implemented)

```text
format -> lint -> license/advisory -> build -> unit -> integration
-> security -> platform tests -> packaging validation -> release approval
```

- Use a pinned toolchain and `Cargo.lock`.
- Cache dependencies without allowing stale or compromised artifacts to bypass verification.
- Run platform-specific jobs for Windows, macOS, and Linux where the change is platform-sensitive.
- Publish test reports and artifacts with retention appropriate to the release.

## 10. Current state

- No `Cargo.toml`, test directory, fixture, CI configuration, or benchmark harness exists.
- No test has been executed for product code.
- `python3 scripts/verify-skills.py` is the only executable project check today.
- The first workspace baseline must include a minimal but real test harness and a documented command set before product implementation is declared complete.
