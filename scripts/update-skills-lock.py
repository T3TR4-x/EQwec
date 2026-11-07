#!/usr/bin/env python3
"""Regenerate docs/development/SKILLS-LOCK.json.

External provenance comes from the global installer lock at
~/.agents/.skill-lock.json plus the manual-install records documented in
docs/development/SKILLS-AUDIT.md. Project skill hashes are computed from the
working tree. The script does not contact the network and does not validate
skills; run scripts/verify-skills.py for that.
"""

from __future__ import annotations

import hashlib
import json
import sys
from datetime import datetime, timezone
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

# requirement, installed ids, source, url, upstream commit verified in the audit,
# declared license, primary skill file relative to the installed skill directory,
# and the verified capability/limitation note.
EXTERNAL = [
    {
        "requirement": "Caveman",
        "installed_ids": ["caveman"],
        "source": "juliusbrussee/caveman",
        "source_url": "https://github.com/juliusbrussee/caveman",
        "skill_path": "skills/caveman/SKILL.md",
        "upstream_commit": None,
        "license": "not declared in the installed package",
        "capability": "Terse output voice. Not a deep-investigation skill; the original responsibility mapping was incorrect.",
    },
    {
        "requirement": "Superpowers",
        "installed_ids": [
            "using-superpowers",
            "brainstorming",
            "systematic-debugging",
            "test-driven-development",
            "writing-plans",
            "executing-plans",
            "verification-before-completion",
            "requesting-code-review",
            "receiving-code-review",
            "subagent-driven-development",
            "dispatching-parallel-agents",
            "using-git-worktrees",
            "finishing-a-development-branch",
            "writing-skills",
            "diagnosing-superpowers",
        ],
        "source": "obra/superpowers",
        "source_url": "https://github.com/obra/superpowers",
        "skill_path": "skills/brainstorming/SKILL.md",
        "upstream_commit": "22f0b46",
        "license": "not present in the installed packages; upstream license not verified in this audit",
        "capability": "Structured planning, debugging, TDD, execution, and verification discipline.",
    },
    {
        "requirement": "Code-review",
        "installed_ids": ["code-review"],
        "source": "mattpocock/skills",
        "source_url": "https://github.com/mattpocock/skills",
        "skill_path": "skills/engineering/code-review/SKILL.md",
        "upstream_commit": "4ae2317",
        "license": "not present in the installed package; upstream license not verified in this audit",
        "capability": "Two-axis review of changes since a fixed point, using parallel sub-agents.",
    },
    {
        "requirement": "Frontend-design",
        "installed_ids": ["frontend-design"],
        "source": "anthropics/skills",
        "source_url": "https://github.com/anthropics/skills",
        "skill_path": "skills/frontend-design/SKILL.md",
        "upstream_commit": "2c5958e",
        "license": "see LICENSE.txt in the installed package",
        "capability": "Distinctive visual design direction and typography guidance.",
    },
    {
        "requirement": "UI/UX Pro Max",
        "installed_ids": ["ui-ux-pro-max"],
        "source": "nextlevelbuilder/ui-ux-pro-max-skill",
        "source_url": "https://github.com/nextlevelbuilder/ui-ux-pro-max-skill",
        "skill_path": "SKILL.md",
        "upstream_commit": None,
        "license": "MIT",
        "capability": "Searchable design-system, palette, typography, UX, icon, chart, and stack guidance.",
    },
    {
        "requirement": "Playwright",
        "installed_ids": ["playwright-cli", "playwright-best-practices"],
        "source": "microsoft/playwright-cli and currents-dev/playwright-best-practices-skill",
        "source_url": "https://github.com/microsoft/playwright-cli",
        "skill_path": "SKILL.md",
        "upstream_commit": None,
        "license": "playwright-best-practices declares MIT; playwright-cli has no license file in the installed package",
        "capability": "Browser automation via playwright-cli and Playwright test guidance. The playwright-cli executable is not verified as installed; execution is BLOCKED until checked.",
    },
    {
        "requirement": "Ponytail",
        "installed_ids": ["ponytail", "ponytail-audit", "ponytail-review"],
        "source": "dietrichgebert/ponytail",
        "source_url": "https://github.com/dietrichgebert/ponytail",
        "skill_path": "skills/ponytail/SKILL.md",
        "upstream_commit": None,
        "license": "not declared in the installed package",
        "capability": "Minimal-change engineering, repository audit, and change review.",
    },
    {
        "requirement": "Matt Pocock Skills",
        "installed_ids": [
            "code-review",
            "codebase-design",
            "diagnosing-bugs",
            "domain-modeling",
            "implement-spec",
            "improve-codebase-architecture",
            "prototype",
            "research",
            "tdd",
            "to-spec",
            "grill-me",
        ],
        "source": "mattpocock/skills",
        "source_url": "https://github.com/mattpocock/skills",
        "skill_path": "skills/engineering/code-review/SKILL.md",
        "upstream_commit": "4ae2317",
        "license": "not present in the installed packages; upstream license not verified in this audit",
        "capability": "Engineering subset: review, design, diagnosis, domain modeling, spec, prototype, research, TDD.",
    },
    {
        "requirement": "Andrej Karpathy Skills",
        "installed_ids": ["karpathy-guidelines"],
        "source": "szkocot/andrej-karpathy-skills",
        "source_url": "https://github.com/szkocot/andrej-karpathy-skills",
        "skill_path": "skills/karpathy-guidelines/SKILL.md",
        "upstream_commit": None,
        "license": "MIT (declared in frontmatter)",
        "capability": "Behavioral guidance against overcomplication and unsupported assumptions. Community packaging, not an official Karpathy repository.",
    },
    {
        "requirement": "Unslop",
        "installed_ids": ["unslop"],
        "source": "heydamir/unslop",
        "source_url": "https://github.com/heydamir/unslop",
        "skill_path": "skills/unslop/SKILL.md",
        "upstream_commit": "a6f121e",
        "license": "CC-BY-4.0",
        "capability": "Remove generic AI-looking UI patterns and establish a product-specific visual direction.",
    },
    {
        "requirement": "Security Audit",
        "installed_ids": ["security-audit"],
        "source": "cloudflare/security-audit-skill",
        "source_url": "https://github.com/cloudflare/security-audit-skill",
        "skill_path": "skills/security-audit/SKILL.md",
        "upstream_commit": "c1c8a8c",
        "license": "MIT",
        "capability": "Structured security review with coverage ledger and findings validation.",
    },
    {
        "requirement": "Skill Creator",
        "installed_ids": ["skill-creator"],
        "source": "anthropics/skills",
        "source_url": "https://github.com/anthropics/skills/tree/main/skills/skill-creator",
        "skill_path": "skills/skill-creator/SKILL.md",
        "upstream_commit": None,
        "license": "Apache-2.0",
        "capability": "Skill authoring and validation. quick_validate.py works; eval scripts call the Claude CLI and are BLOCKED under OpenCode.",
    },
]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def installed_file(home: Path, skill_id: str) -> Path:
    base = home / ".agents" / "skills" / skill_id
    if skill_id == "unslop":
        return base / "skills" / "unslop" / "SKILL.md"
    return base / "SKILL.md"


def main() -> int:
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--validated",
        action="store_true",
        help="mark project skills VALIDATED_STATIC; use only after scripts/verify-skills.py passes",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    home = Path.home()
    generated = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    project_status = "VALIDATED_STATIC" if args.validated else "PRESENT_UNVALIDATED"

    external_records = []
    missing = []
    for item in EXTERNAL:
        files = []
        for skill_id in item["installed_ids"]:
            path = installed_file(home, skill_id)
            if not path.is_file():
                missing.append(str(path))
                continue
            files.append(
                {
                    "id": skill_id,
                    "path": str(path).replace(str(home), "~"),
                    "sha256": sha256(path),
                }
            )
        external_records.append(
            {
                "requirement": item["requirement"],
                "source": item["source"],
                "source_url": item["source_url"],
                "skill_path": item["skill_path"],
                "upstream_commit_verified": item["upstream_commit"],
                "license": item["license"],
                "capability": item["capability"],
                "installed_files": files,
                "status": "AVAILABLE" if files else "MISSING",
            }
        )

    project_records = []
    for name in EXPECTED:
        skill_file = root / ".agents" / "skills" / name / "SKILL.md"
        if not skill_file.is_file():
            project_records.append({"name": name, "status": "MISSING"})
            missing.append(str(skill_file))
            continue
        text = skill_file.read_text(encoding="utf-8")
        project_records.append(
            {
                "name": name,
                "path": f".agents/skills/{name}/SKILL.md",
                "sha256": sha256(skill_file),
                "lines": text.count("\n") + (0 if text.endswith("\n") else 1),
                "status": project_status,
            }
        )

    document = {
        "schema_version": 1,
        "generated_at": generated,
        "repository": "EQwec",
        "branch": "integration",
        "policy": {
            "canonical_project_registry": ".agents/skills/",
            "main_branch_skills": "forbidden; main is README-only",
            "external_registry": "~/.agents/skills/ (OpenCode discovery)",
            "validation": "python3 scripts/verify-skills.py --validator ~/.agents/skills/skill-creator/scripts/quick_validate.py",
            "hash_algorithm": "sha256",
            "note": "PRESENT_UNVALIDATED becomes PASS only after the verifier and audit record succeed.",
        },
        "external_skills": external_records,
        "project_skills": project_records,
        "missing": missing,
    }

    out = root / "docs" / "development" / "SKILLS-LOCK.json"
    out.write_text(json.dumps(document, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"wrote {out}")
    if missing:
        print("MISSING:")
        for path in missing:
            print(f"  {path}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
