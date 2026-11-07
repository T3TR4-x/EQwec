#!/usr/bin/env python3
"""Deterministic preflight for the EQwec custom skill registry.

This script proves file-level facts only: package presence, frontmatter shape,
naming, required cross-references, description coverage of trigger cases, and
absence of placeholder text. It does NOT prove that a model will activate a
skill; that requires an agent-level trigger test recorded in
docs/development/SKILLS-AUDIT.md.

Exit codes: 0 = PASS, 1 = FAIL, 2 = usage error.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

EXPECTED = [
    "eqwec-engineering-core",
    "eqwec-quality-control",
    "eqwec-hub",
    "eqwec-browser",
    "eqwec-photocraft",
    "eqwec-vectorcraft",
    "eqwec-filmcraft",
    "eqwec-effectcraft",
    "eqwec-craftanything",
    "eqwec-audiocraft",
]

PRODUCT_SKILLS = [s for s in EXPECTED if s not in ("eqwec-engineering-core", "eqwec-quality-control")]

MIN_LINES = {"eqwec-engineering-core": 100, "eqwec-quality-control": 100}
DEFAULT_MIN_LINES = 150
MAX_DESCRIPTION = 1024


def parse_frontmatter(text: str) -> tuple[dict[str, str], str]:
    if not text.startswith("---\n"):
        raise ValueError("file does not start with a YAML frontmatter delimiter")
    end = text.find("\n---\n", 4)
    if end == -1:
        raise ValueError("frontmatter is not closed with ---")
    raw = text[4:end]
    body = text[end + 5 :]
    fields: dict[str, str] = {}
    for line in raw.splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        if ":" not in line:
            raise ValueError(f"frontmatter line is not a key/value pair: {line!r}")
        key, value = line.split(":", 1)
        fields[key.strip()] = value.strip().strip('"').strip("'")
    return fields, body


def load_cases(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def term_hits(description: str, terms: list[str]) -> int:
    lowered = description.lower()
    return sum(1 for term in terms if term.lower() in lowered)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--validator", default=None, help="path to skill-creator quick_validate.py")
    parser.add_argument(
        "--validator-python",
        default=sys.executable,
        help="interpreter with PyYAML installed (defaults to the current interpreter)",
    )
    parser.add_argument("--cases", default="docs/development/SKILL-TRIGGER-CASES.json")
    parser.add_argument("--json", action="store_true", help="emit machine-readable results")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    skills_dir = root / ".agents" / "skills"
    cases = load_cases(root / args.cases)

    failures: list[str] = []
    warnings: list[str] = []
    results: list[dict] = []

    if not skills_dir.is_dir():
        print(f"FAIL: skill directory missing: {skills_dir}", file=sys.stderr)
        return 1

    present = sorted(p.name for p in skills_dir.iterdir() if p.is_dir() and p.name.startswith("eqwec-"))
    if present != sorted(EXPECTED):
        missing = sorted(set(EXPECTED) - set(present))
        extra = sorted(set(present) - set(EXPECTED))
        if missing:
            failures.append(f"missing skill packages: {', '.join(missing)}")
        if extra:
            failures.append(f"unexpected eqwec-* packages: {', '.join(extra)}")

    for name in EXPECTED:
        skill_dir = skills_dir / name
        skill_file = skill_dir / "SKILL.md"
        record: dict = {"name": name, "status": "PASS", "checks": {}}

        if not skill_file.is_file():
            failures.append(f"{name}: SKILL.md missing")
            record["status"] = "FAIL"
            results.append(record)
            continue

        text = skill_file.read_text(encoding="utf-8")
        lines = text.count("\n") + (0 if text.endswith("\n") else 1)
        record["lines"] = lines

        try:
            fields, body = parse_frontmatter(text)
        except ValueError as exc:
            failures.append(f"{name}: {exc}")
            record["status"] = "FAIL"
            results.append(record)
            continue

        fm_name = fields.get("name", "")
        description = fields.get("description", "")

        if fm_name != name:
            failures.append(f"{name}: frontmatter name is {fm_name!r}, expected {name!r}")
        if not description:
            failures.append(f"{name}: description is empty")
        elif len(description) > MAX_DESCRIPTION:
            failures.append(f"{name}: description is {len(description)} chars (max {MAX_DESCRIPTION})")

        lowered = description.lower()
        if not re.search(r"\b(use when|use for|use before|use after|trigger)\b", lowered):
            failures.append(f"{name}: description has no trigger language")
        if name in PRODUCT_SKILLS and not re.search(r"\b(do not use|not for|don't use)\b", lowered):
            failures.append(f"{name}: product description has no explicit non-trigger clause")

        minimum = MIN_LINES.get(name, DEFAULT_MIN_LINES)
        if lines < minimum:
            failures.append(f"{name}: only {lines} lines, expected at least {minimum}")

        if name != "eqwec-engineering-core" and not re.search(r"eqwec-engineering-core", body):
            failures.append(f"{name}: body does not reference eqwec-engineering-core")
        if not re.search(r"eqwec-quality-control", body):
            failures.append(f"{name}: body does not reference eqwec-quality-control")

        headings = len(re.findall(r"^#{2,3} ", body, flags=re.MULTILINE))
        if headings < 4:
            failures.append(f"{name}: only {headings} section headings")

        for marker in ("TODO", "TBD", "FIXME", "Lorem ipsum"):
            if marker.lower() in text.lower():
                failures.append(f"{name}: contains placeholder marker {marker!r}")

        if "/Users/" in text or "C:\\Users\\" in text:
            failures.append(f"{name}: contains an absolute user path")

        skill_cases = cases.get("skills", {}).get(name, {})
        for case in skill_cases.get("positive", []):
            terms = case.get("terms", [])
            required = case.get("min_hits", 2)
            hits = term_hits(description, terms)
            record["checks"].setdefault("positive", []).append(
                {"prompt": case.get("prompt", ""), "hits": hits, "required": required}
            )
            if hits < required:
                failures.append(
                    f"{name}: positive case {case.get('id', '?')} matched {hits}/{required} description terms"
                )
        for case in skill_cases.get("negative", []):
            terms = case.get("terms", [])
            hits = term_hits(description, terms)
            record["checks"].setdefault("negative", []).append(
                {"prompt": case.get("prompt", ""), "exclusion_hits": hits}
            )
            if hits < 1:
                warnings.append(
                    f"{name}: negative case {case.get('id', '?')} has no explicit exclusion term in description"
                )

        if args.validator:
            proc = subprocess.run(
                [args.validator_python, args.validator, str(skill_dir)],
                capture_output=True,
                text=True,
            )
            record["validator"] = (proc.stdout + proc.stderr).strip()
            if proc.returncode != 0:
                if "No module named 'yaml'" in record["validator"]:
                    failures.append(
                        f"{name}: quick_validate cannot run: the validator interpreter lacks PyYAML. "
                        "See scripts/skill-validation-requirements.txt."
                    )
                else:
                    failures.append(f"{name}: quick_validate failed: {record['validator']}")

        if any(f.startswith(f"{name}:") for f in failures):
            record["status"] = "FAIL"
        results.append(record)

    if args.json:
        print(json.dumps({"results": results, "failures": failures, "warnings": warnings}, indent=2))
    else:
        for record in results:
            print(f"{record['status']:5} {record['name']:28} {record.get('lines', 0):4} lines")
        for warning in warnings:
            print(f"WARN  {warning}")
        for failure in failures:
            print(f"FAIL  {failure}")
        print(f"\nRESULT: {'FAIL' if failures else 'PASS'} ({len(results)} skills, {len(failures)} failures, {len(warnings)} warnings)")

    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
