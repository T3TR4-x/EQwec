#!/usr/bin/env python3
"""Check documentation integrity and the planning-document gate.

This script is intentionally dependency-free. It verifies:
- required planning documents exist and are substantive;
- relative Markdown links resolve;
- code fences are balanced;
- no absolute user paths or obvious secret patterns appear;
- AGENTS.md and CLAUDE.md both exist.

It does not prove that a document is correct or that a gate should pass; it
only detects missing or structurally broken documentation.

Exit codes: 0 = all requested checks pass, 1 = failures, 2 = usage error.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REQUIRED = [
    "docs/research/FEASIBILITY.md",
    "docs/research/COMPETITOR-ANALYSIS.md",
    "docs/research/LEGAL-LICENSE-REVIEW.md",
    "docs/research/BROWSER-ENGINE-FEASIBILITY.md",
    "docs/architecture/SYSTEM-ARCHITECTURE.md",
    "docs/architecture/TECHNOLOGY-SELECTION.md",
    "docs/architecture/DEPENDENCY-MAP.md",
    "docs/security/THREAT-MODEL.md",
    "docs/security/SECURITY-BASELINE.md",
    "docs/performance/PERFORMANCE-STRATEGY.md",
    "docs/development/ROADMAP.md",
    "docs/development/BRANCHING-STRATEGY.md",
    "docs/development/VERSIONING.md",
    "docs/development/TEST-STRATEGY.md",
    "docs/development/RELEASE-STRATEGY.md",
    "docs/development/SKILLS-AUDIT.md",
    "docs/development/SKILLS-LOCK.json",
    "docs/development/AGENT-INSTRUCTIONS-POLICY.md",
    "docs/development/DOCUMENTATION-LIFECYCLE.md",
    "docs/development/QUALITY-GATES.md",
    "docs/development/SKILL-USAGE.md",
    "docs/development/SKILL-TRIGGER-CASES.json",
]

SECRET_PATTERNS = [
    re.compile(r"sk-[A-Za-z0-9]{20,}"),
    re.compile(r"ghp_[A-Za-z0-9]{20,}"),
    re.compile(r"BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY"),
    re.compile(r"AKIA[0-9A-Z]{16}"),
]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--gate", action="store_true", help="treat missing required documents as failures")
    parser.add_argument("--min-lines", type=int, default=20)
    args = parser.parse_args()

    root = Path(args.root).resolve()
    failures: list[str] = []
    warnings: list[str] = []

    if not (root / "AGENTS.md").is_file():
        failures.append("AGENTS.md is missing")
    if not (root / "CLAUDE.md").is_file():
        failures.append("CLAUDE.md is missing")

    if args.gate:
        for rel in REQUIRED:
            path = root / rel
            if not path.is_file():
                failures.append(f"required document missing: {rel}")
                continue
            lines = path.read_text(encoding="utf-8").count("\n") + 1
            if lines < args.min_lines:
                failures.append(f"required document is too thin: {rel} ({lines} lines)")

    markdown = [p for p in root.rglob("*.md") if ".git" not in p.parts]
    for path in markdown:
        text = path.read_text(encoding="utf-8")
        if text.count("```") % 2:
            failures.append(f"{path.relative_to(root)}: unbalanced code fences")
        for match in re.finditer(r"\[[^\]]+\]\(([^)]+)\)", text):
            target = match.group(1).split("#")[0]
            if not target or target.startswith(("http://", "https://", "mailto:")):
                continue
            if not (path.parent / target).resolve().exists():
                failures.append(f"{path.relative_to(root)}: broken relative link: {target}")
        for pattern in SECRET_PATTERNS:
            if pattern.search(text):
                failures.append(f"{path.relative_to(root)}: possible secret pattern: {pattern.pattern}")
        if "/Users/" in text or "C:\\Users\\" in text:
            failures.append(f"{path.relative_to(root)}: absolute user path")

    print(f"Markdown files: {len(markdown)}")
    print(f"Required documents: {len(REQUIRED)}")
    for warning in warnings:
        print(f"WARN  {warning}")
    for failure in failures:
        print(f"FAIL  {failure}")
    print(f"RESULT: {'FAIL' if failures else 'PASS'} ({len(failures)} failures)")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
