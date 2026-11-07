# ADR-0001: Rust-first modular monorepo

- **Status:** ACCEPTED
- **Date:** 2026-11-07
- **Deciders:** EQwec project owner and engineering agent (bootstrap mandate)

## Context

EQwec is a planned ecosystem of eight desktop products (Hub, Browser, PhotoCraft, VectorCraft, FilmCraft, EffectCraft, CraftAnything, AudioCraft) plus shared infrastructure. The products share security, storage, IPC, graphics, media, logging, configuration, and design-system concerns, but must be independently buildable, testable, versioned, packaged, and released.

The project needs a repository structure that:

- Keeps product boundaries explicit.
- Allows shared crates to be tested against every consumer.
- Avoids duplicating infrastructure across eight repositories.
- Preserves independent release cadence.
- Supports a single reviewed integration baseline and scoped release branches.

## Alternatives considered

### A. Polyrepo (one repository per product)

Advantages: strong isolation, independent access control, small checkouts.

Disadvantages: shared-crate changes require coordinated multi-repo releases; cross-product contract tests are harder; version skew is likely; a single security fix may need eight pull requests; onboarding and CI duplication increase.

### B. Single crate / single binary with feature flags

Advantages: simplest initial build.

Disadvantages: weak boundaries, long compile times, feature-flag combinatorics, accidental coupling, one version for unrelated products, and a high risk that product-specific policy leaks into shared code. Not suitable for independently released professional applications.

### C. Microservices

Advantages: strong network boundaries.

Disadvantages: inappropriate for local desktop applications; adds deployment, latency, failure, and operational complexity with no user benefit. Rejected.

### D. Rust-first modular monorepo with a Cargo workspace

Advantages: shared crates with one dependency graph and lockfile; cross-product tests in one CI run; explicit crate boundaries; independent binaries and release metadata; one reviewed integration baseline; atomic shared-contract changes.

Disadvantages: larger checkout and CI graph; requires discipline to prevent coupling; a workspace-wide build can be slower unless scoped; access control is coarser than polyrepo.

## Decision

Adopt a Rust-first modular monorepo with a Cargo workspace.

- Application entry points live under `apps/`.
- Reusable libraries live under `crates/`.
- Shared crates have owners, public APIs, compatibility expectations, and tests that exercise every consumer.
- Products build, test, version, package, and release independently.
- `main` remains README-only; `integration` holds the shared baseline; `dev-*` branches hold product work; `production` holds verified releases.
- Do not create empty crates merely to match a diagram. A crate exists when it has a real owner, API, tests, and a second consumer or a documented architectural reason.

## Consequences

Positive:

- Shared security and infrastructure fixes land once and are tested across consumers.
- Cross-product contract tests are practical.
- One toolchain, lockfile, and CI baseline.
- Product boundaries are visible in the directory and dependency graph.

Negative:

- CI must scope builds and tests to affected crates to avoid unnecessary cost.
- Repository access control is coarser than polyrepo.
- A workspace-wide breaking change requires coordinated updates.
- Discipline is required to prevent product-to-product dependencies.

Security: shared crates become a high-value target; they require review, ownership, and supply-chain controls. Product isolation is preserved at the crate and process level, not by repository separation.

Performance: shared code can be optimized once, but a monolithic build graph can slow feedback. Use targeted builds, caching, and affected-crate CI.

Licensing: one repository simplifies license inventory and SBOM generation, but every dependency and asset still requires individual review.

Maintenance: lower coordination cost, higher need for ownership and contract discipline.

## Reconsideration criteria

Revisit this decision if:

- Build times or CI cost become unmanageable despite scoping and caching.
- Access control or legal isolation requires separate repositories.
- A product needs an independent release cadence that the workspace cannot support.
- A shared crate becomes a coupling bottleneck that cannot be fixed with a versioned contract.

## Links

- `docs/architecture/SYSTEM-ARCHITECTURE.md`
- `docs/development/BRANCHING-STRATEGY.md`
- `docs/development/VERSIONING.md`
- `AGENTS.md`
