# Versioning

**Status:** approved policy. All products are at planning version `0.0.0`; no release exists.

## 1. Principles

- Each product has an independent version history. A shared-crate change does not automatically bump every product.
- Versions use Semantic Versioning: `MAJOR.MINOR.PATCH`.
- `0.0.0` means planning state, not a usable release.
- A version bump is a deliberate milestone or release action, not a side effect of a commit.
- One canonical version source exists per product. Cargo metadata, release manifests, update metadata, changelogs, and tags are synchronized by verified tooling.
- Never describe an alpha, prototype, or planning build as stable.

## 2. Product versions

| Product | Current | First planned public milestone |
|---|---:|---|
| EQwec Hub | `0.0.0` | `0.1.0-alpha.1` after the Hub MVP gate |
| EQwec Browser | `0.0.0` | `0.0.1-alpha.1` after the engine feasibility prototype |
| PhotoCraft | `0.0.0` | `0.1.0-alpha.1` after the first complete document workflow |
| VectorCraft | `0.0.0` | `0.1.0-alpha.1` after the first complete vector workflow |
| FilmCraft | `0.0.0` | `0.1.0-alpha.1` after the first complete edit/render workflow |
| EffectCraft | `0.0.0` | `0.1.0-alpha.1` after the first complete compositing workflow |
| CraftAnything | `0.0.0` | `0.1.0-alpha.1` after the scope ADR and first asset-exchange workflow |
| AudioCraft | `0.0.0` | `0.1.0-alpha.1` after the first record/edit/render workflow |

The exact milestone version is confirmed when the milestone scope is approved. The table records intent, not a promise of date or feature completeness.

## 3. Prerelease progression

```text
0.0.0  planning
0.0.1-alpha.1  first narrow prototype
0.1.0-alpha.1  first usable vertical slice
0.1.0-beta.1   feature-complete for the milestone, hardening
0.1.0-rc.1     release candidate, only blocking fixes
0.1.0          stable milestone
```

Rules:

- Prerelease identifiers are lowercase and dot-separated: `alpha.1`, `beta.2`, `rc.1`.
- Build metadata (`+sha`) may identify a build but never determines version ordering.
- A prerelease is not stable. Do not publish it on a stable channel.
- A release candidate accepts only blocking fixes; any feature change requires a new prerelease.

## 4. What triggers a bump

| Change | Version effect |
|---|---|
| Incompatible public API or document format change | `MAJOR` (or `MINOR` while `0.y.z`, with explicit compatibility notes) |
| Backward-compatible feature | `MINOR` |
| Backward-compatible bug or security fix | `PATCH` |
| Shared-crate change with no product behavior change | Assess first; usually no product bump |
| Documentation-only change | No product bump |
| Dependency security update with no API change | `PATCH`, with advisory note |
| New product milestone | Deliberate `MINOR`/`PATCH` prerelease per approved scope |

While the major version is `0`, public APIs are not stable, but the project still documents breaking changes and avoids silent breakage. Do not use `0.x` as an excuse for uncontrolled churn.

## 5. Shared crates

- Shared crates are versioned in the workspace and tested against every consumer.
- A shared crate may use its own SemVer cadence. A breaking shared-crate change requires an impact assessment and coordinated consumer updates.
- Do not publish a shared crate as a stable public API until it has an owner, documented compatibility expectations, and cross-product contract tests.
- Product crates must not depend on another product's internal crate.

## 6. Canonical metadata

Per product, maintain:

- `Cargo.toml` package version (when the workspace exists).
- Release manifest/update metadata version.
- Changelog entry.
- Git tag `<product>/vX.Y.Z` (for example `eqwec-hub/v0.1.0`).
- Build identifier and commit SHA.
- Supported platforms and architectures.
- Engine/dependency versions where applicable (Chromium, codecs, audio backend).
- Compatibility notes and known limitations.

Synchronization must be checked by a deterministic script as crates are added. Today only `eqwec-core` exists, so there is no cross-crate version set to synchronize.

## 7. Tags and signing

- Use namespaced tags: `eqwec-hub/v0.1.0`, `photocraft/v0.1.0`, `audiocraft/v0.1.0`.
- Prefer signed tags when the release infrastructure supports them.
- Never reuse or move a published tag.
- A tag without a verified artifact and release record is not a release.

## 8. Release record

A release record contains:

```text
product, version, channel, release date, commit SHA, tag,
build environment, dependency/engine versions, artifact hashes,
signature/key identity, supported platforms, changelog,
compatibility notes, rollback target, approval reference
```

No release record, no release.

## 9. Current state

- No product has a `Cargo.toml`, release manifest, changelog, or tag. Only the shared `eqwec-core` crate exists, at `0.0.0`.
- No version has been bumped beyond `0.0.0`.
- Version tooling is `BLOCKED` until the workspace exists; do not invent commands.
