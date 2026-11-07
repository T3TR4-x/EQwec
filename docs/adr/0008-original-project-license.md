# ADR-0008: Original project license

- **Status:** PROPOSED (owner approval required; counsel review recommended)
- **Date:** 2026-11-07
- **Deciders:** EQwec project owner; engineering agent recommends; legal counsel must review before adoption
- **Supersedes:** none
- **Related:** ADR-0001, ADR-0004, ADR-0007, `docs/research/LEGAL-LICENSE-REVIEW.md`

## 1. Context

EQwec has no public license. Without one, the repository grants no rights to use, modify, or redistribute the source, and no binary can be distributed lawfully. The license also determines contributor terms, patent posture, compatibility with third-party dependencies, and whether the browser and media products can combine with platform SDKs, codecs, and commercial components.

The decision is expensive to reverse once contributions and distributions exist. It must be made deliberately, not by default.

## 2. Requirements

- Permits use, modification, and redistribution of EQwec-owned source and binaries.
- Compatible with the permissive and weak-copyleft dependencies already selected (MIT, Apache-2.0, BSD, ISC, MPL-2.0, LGPL-2.1/3.0).
- Does not force source disclosure of EQwec product code merely because a permissive or LGPL dependency is linked.
- Provides an express patent grant.
- Allows the browser and media products to combine with platform SDKs and, if legally approved, proprietary codec or DRM components.
- Is familiar to Rust and open-source contributors.
- Allows a sustainable business model without relicensing existing contributions.
- Keeps the EQwec name, logos, and brand assets under separate trademark terms.
- Defines contribution terms (DCO or CLA) and a provenance record.
- Does not grant trademark rights or imply endorsement.

## 3. Alternatives considered

### A. MIT OR Apache-2.0 (dual)

The Rust ecosystem default. MIT is short and permissive; Apache-2.0 adds an express patent grant, notice preservation, and a patent-retaliation clause. Users choose either. Maximizes compatibility with Rust crates and commercial adopters. Allows proprietary forks. Does not force disclosure of EQwec code.

### B. Apache-2.0 only

Similar to dual but less convenient for MIT-only consumers and some GPLv2 projects. Apache-2.0 is incompatible with GPLv2, which can complicate optional GPL components. Viable.

### C. MPL-2.0

File-level copyleft with a patent grant. Permits proprietary larger works but requires modifications to MPL-covered files to remain MPL and be published. More complex for contributors; less familiar in Rust. A credible middle ground if the owner wants a reciprocity boundary without GPL linking effects.

### D. LGPL-3.0

Library-boundary copyleft. Suitable for libraries, awkward as the license of an application and its plugins. Adds relinking and installation-information obligations. Not recommended for the product source as a whole.

### E. GPL-3.0

Strong copyleft. Prevents proprietary forks and aligns with a fully free-software strategy. Conflicts with shipping proprietary codecs, DRM, platform SDKs, or commercial components in the same work. Would force the codec/DRM strategy to change. Viable only if the entire product strategy is GPL-compatible.

### F. AGPL-3.0

GPL-3.0 plus a network-use source offer. Adds obligations that are not useful for local desktop applications and confuse users about whether local use triggers distribution. Rejected.

### G. Proprietary or source-available

Maximum commercial control, but limits community contribution, trust, audits, and adoption. Requires custom terms and a contribution policy. Possible, but inconsistent with the stated ecosystem vision.

### H. No license (all rights reserved)

The current de facto state. Blocks contribution, redistribution, packaging, and release. Not a viable option.

## 4. Evidence

- MIT and Apache-2.0 are OSI-approved, permissive licenses; Apache-2.0 includes an express patent grant and patent-retaliation clause.
- The Rust ecosystem predominantly uses `MIT OR Apache-2.0`; Cargo makes dual-license metadata and notice collection practical.
- MPL-2.0 is file-level copyleft with a patent grant and is compatible with larger proprietary works under its conditions.
- GPL-3.0 and AGPL-3.0 impose source-disclosure and, for AGPL, network-use obligations that conflict with proprietary codec/DRM and some commercial distribution models.
- Codec patents are independent of copyright licenses; no project license resolves patent licensing for H.264, AAC, HEVC, ProRes, or DNxHR.
- `docs/research/LEGAL-LICENSE-REVIEW.md` records the dependency license inventory and the unresolved legal blockers.
- No contributor policy, CLA, DCO, or trademark policy exists yet.

## 5. Decision

### 5.1 Recommended license

Release EQwec-owned source under **`MIT OR Apache-2.0`**, with Apache-2.0 as the default patent-grant option, and keep the EQwec name, logos, and brand assets under separate trademark terms.

### 5.2 Contribution terms

- Require a Developer Certificate of Origin (DCO) sign-off on every commit initially.
- Record author identity and provenance in the commit history.
- Do not adopt a CLA unless the owner needs deliberate dual licensing or relicensing flexibility; a CLA must be reviewed by counsel and communicated to contributors.
- Do not accept contributions that cannot be licensed under the project license.

### 5.3 Trademarks and brand

- The source license does not grant rights to the EQwec name, logos, or product marks.
- Publish a trademark policy before the first public release.
- Do not use third-party marks in a way that implies endorsement.
- Product names must be checked for conflicts before release.

### 5.4 Third-party compatibility

- The chosen license must remain compatible with the dependency inventory in `docs/research/LEGAL-LICENSE-REVIEW.md`.
- MPL-2.0 dependencies stay behind crate boundaries; modifications are published.
- LGPL dependencies use dynamic linking or a separate process with a documented compliance method.
- GPL components are excluded from the default proprietary build.
- Codec and DRM decisions remain governed by ADR-0004 and require separate legal approval.

### 5.5 Files and metadata

When accepted:

1. Add `LICENSE-MIT` and `LICENSE-APACHE` at the repository root.
2. Add the dual-license statement to `README.md` and package metadata.
3. Add SPDX identifiers to source files or package manifests.
4. Add `CONTRIBUTING.md` with the DCO requirement.
5. Add a `TRADEMARKS.md` policy.
6. Record the decision in `docs/development/VERSIONING.md` and the release checklist.

Until this ADR is accepted, no public license file exists and redistribution remains unauthorized.

## 6. Conditions and gates

1. **Owner approval:** the project owner must accept or amend this ADR in writing.
2. **Counsel review:** confirm compatibility with the dependency inventory and the codec/DRM strategy.
3. **Contribution policy:** DCO text and enforcement must exist before accepting contributions.
4. **Trademark policy:** must exist before the first public release.
5. **Notice and SBOM process:** must be defined before the first binary release (ADR-0007).

## 7. Consequences

**Positive**

- Maximum compatibility with the Rust ecosystem and commercial adopters.
- Express patent grant through Apache-2.0.
- Proprietary forks are allowed; the project does not depend on copyleft enforcement.
- Browser and media products can combine with approved platform and codec components.
- Contributors face familiar terms.

**Negative**

- Competitors may fork the code without contributing back.
- The project cannot rely on copyleft to keep improvements open.
- Dual licensing requires clear notice collection and SPDX discipline.
- A permissive license does not resolve codec patents or DRM agreements.
- Trademark protection must be handled separately.

**Security**

- The license does not affect the security model.
- Security researchers may audit and redistribute the code, which is desirable.
- Do not accept contributions that weaken security controls without review.

**Performance**

- No direct effect.

**Licensing**

- MIT OR Apache-2.0 is compatible with MIT, BSD, ISC, Apache-2.0, MPL-2.0, and LGPL dependencies under their obligations.
- GPL components remain excluded from the default build.
- Codec and DRM licensing remain `BLOCKED` until counsel review.
- Third-party notices and SBOM remain mandatory.

**Maintenance**

- SPDX identifiers and license headers must be kept consistent.
- Contribution provenance must be enforced.
- A future relicensing would require permission from every copyright holder; the DCO model makes that harder than a CLA, which is a deliberate tradeoff.

## 8. Reconsideration criteria

Revisit this ADR if:

- The owner chooses a copyleft or source-available business model.
- Counsel finds a dependency incompatibility that requires a different license.
- A codec, DRM, or platform agreement requires different terms.
- The project adopts a CLA or dual-licensing model.
- A trademark or patent dispute changes the risk posture.
- The contribution volume makes provenance or relicensing impractical.

## 9. Links

- `docs/research/LEGAL-LICENSE-REVIEW.md`
- `docs/architecture/TECHNOLOGY-SELECTION.md`
- `docs/development/VERSIONING.md`
- `docs/development/RELEASE-STRATEGY.md`
- ADR-0001
- ADR-0004
- ADR-0007
