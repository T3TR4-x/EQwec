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
| 0002 | GUI and rendering stack | Pending research |
| 0003 | Browser engine foundation | Pending research |
| 0004 | Media and codec strategy | Pending research |
| 0005 | Audio backend and plugin hosting | Pending research |
| 0006 | Project serialization format | Not started |
| 0007 | Packaging and signing | Not started |
| 0008 | Original project license | Not started; owner approval required |
| 0009 | Crash reporting and diagnostics | Not started |
| 0010 | Plugin and extension trust model | Not started |

## Numbering

Use four digits: `0001-short-title.md`. Do not renumber. A superseded ADR keeps its number.
