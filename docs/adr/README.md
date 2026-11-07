# Architecture Decision Records

This directory records significant, hard-to-reverse technical decisions. An ADR is evidence and history, not a suggestion box.

## Lifecycle

```text
PROPOSED -> ACCEPTED -> SUPERSEDED
                    \-> REJECTED
```

- `PROPOSED` — under review; not normative.
- `ACCEPTED` — normative until superseded.
- `REJECTED` — considered and declined; keep for history.
- `SUPERSEDED` — replaced by a newer ADR; link both ways.

Never delete an ADR. If a decision changes, write a new one and mark the old one superseded.

## Required sections

1. Title, status, date, deciders.
2. Context and problem statement.
3. Requirements and constraints.
4. Alternatives considered.
5. Evidence and sources.
6. Decision.
7. Consequences: positive, negative, security, performance, licensing, maintenance.
8. Reconsideration criteria.
9. Links to related ADRs and documents.

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](0001-rust-first-modular-monorepo.md) | Rust-first modular monorepo | ACCEPTED |
| [0002](0002-gui-and-rendering-stack.md) | GUI and rendering stack | ACCEPTED with prototype gate |
| [0003](0003-browser-engine-foundation.md) | Browser engine foundation | ACCEPTED with prototype and legal gate |
| [0004](0004-media-and-codecs.md) | Media decode, encode, and codec policy | PROPOSED; legal gate |
| [0005](0005-audio-backend-and-plugins.md) | Audio backend and plugin hosting | ACCEPTED with real-time prototype gate |
| [0006](0006-project-serialization.md) | Project serialization and storage | ACCEPTED |
| [0007](0007-packaging-and-signing.md) | Packaging, updates, and signing | ACCEPTED with release gate |
| [0008](0008-original-project-license.md) | Original project license | PROPOSED; owner approval required |
| [0009](0009-crash-reporting-and-diagnostics.md) | Crash reporting and diagnostics | ACCEPTED with implementation gate |
| [0010](0010-plugin-and-extension-trust.md) | Plugin and extension trust model | ACCEPTED with prototype and isolation gates |
| [0011](0011-hub-manifest-registry-and-layout.md) | Hub manifest, registry, journal, and layout | ACCEPTED with implementation gates |
| [0012](0012-hub-package-archive.md) | Hub package archive | ACCEPTED with implementation gates |

An ADR marked "with a gate" is normative as a direction, but the gated work is `BLOCKED` until its stated evidence exists. A `PROPOSED` ADR is not normative.

## Numbering

Use four digits: `0001-short-title.md`. Do not renumber. A superseded ADR keeps its number.
