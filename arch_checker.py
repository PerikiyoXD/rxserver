#!/usr/bin/env python3
"""
Architecture Compliance Checker
Checks if src/ matches the architecture defined in a JSON specification file.
- Parses JSON specification for clean, reliable compliance checking.
- Generates a detailed report and actionable suggestions.
"""

import os
import sys
import json
from pathlib import Path
from typing import Dict, Optional, Any
from dataclasses import dataclass, field
from enum import Enum


class ComplianceLevel(Enum):
    COMPLIANT = "[OK] Compliant"
    MISSING = "[MISSING] Missing"
    EXTRA = "[EXTRA] Extra"
    TYPE_MISMATCH = "[TYPE] Type Mismatch"


@dataclass
class FileNode:
    name: str
    is_dir: bool
    children: Dict[str, "FileNode"] = field(default_factory=dict)
    description: str = ""
    parent: Optional["FileNode"] = None

    def add_child(self, child: "FileNode"):
        self.children[child.name] = child
        child.parent = self


def parse_json_structure(json_obj: Dict[str, Any], name: str = "") -> FileNode:
    """Parse a JSON object into a FileNode tree."""
    # Determine if this is a directory (has nested objects) or file (has string value)
    is_dir = isinstance(json_obj, dict) and any(
        isinstance(v, dict) for v in json_obj.values()
    )

    node = FileNode(name, is_dir)

    if isinstance(json_obj, dict):
        for _key, value in json_obj.items():
            if isinstance(value, dict):
                # Nested directory
                child = parse_json_structure(value, _key)
                node.add_child(child)
            elif isinstance(value, str):
                # File with description
                child = FileNode(_key, False, description=value)
                node.add_child(child)
            else:
                # Treat as file
                child = FileNode(_key, False)
                node.add_child(child)

    return node


def parse_specification(spec_path: Path, debug=False) -> FileNode:
    """Parse JSON specification file."""
    if not spec_path.exists():
        raise FileNotFoundError(f"Specification file not found: {spec_path}")

    try:
        with open(spec_path, "r", encoding="utf-8") as f:
            json_data = json.load(f)

        if debug:
            print(f"DEBUG: Parsed JSON specification from {spec_path}")

        # Handle the case where JSON has a root "src" _key
        if len(json_data) == 1 and "src" in json_data:
            return parse_json_structure(json_data["src"], "src")
        else:
            # Assume the whole JSON represents the src structure
            return parse_json_structure(json_data, "src")

    except json.JSONDecodeError as e:
        raise Exception(f"Invalid JSON in specification file {spec_path}: {e}")
    except Exception as e:
        raise Exception(f"Could not parse specification file {spec_path}: {e}")


def scan_filesystem(path: Path) -> Optional[FileNode]:
    """Scan the filesystem at path into a FileNode tree."""
    if not path.exists():
        return None

    node = FileNode(path.name, path.is_dir())
    if path.is_dir():
        try:
            entries = []
            for entry in path.iterdir():
                # Skip hidden files and common build artifacts
                if entry.name.startswith(".") or entry.name in [
                    "target",
                    "__pycache__",
                    "node_modules",
                    "build",
                ]:
                    continue
                entries.append(entry)

            # Sort entries for consistent ordering
            for entry in sorted(entries):
                child = scan_filesystem(entry)
                if child:
                    node.add_child(child)
        except PermissionError:
            # Skip directories we can't read
            pass
    return node


def compare(
    expected: FileNode, actual: Optional[FileNode], prefix=""
) -> Dict[str, ComplianceLevel]:
    """Compare two FileNode trees, collect compliance issues."""
    results = {}
    curr_path = str(Path(prefix) / expected.name) if prefix else expected.name

    if actual is None:
        results[curr_path] = ComplianceLevel.MISSING
        # Recursively mark all children as missing
        for child in expected.children.values():
            results.update(compare(child, None, curr_path))
        return results

    if expected.is_dir != actual.is_dir:
        results[curr_path] = ComplianceLevel.TYPE_MISMATCH
        return results

    results[curr_path] = ComplianceLevel.COMPLIANT

    if expected.is_dir:
        expected_names = set(expected.children.keys())
        actual_names = set(actual.children.keys())

        # Handle missing files/directories
        for name in expected_names - actual_names:
            results.update(compare(expected.children[name], None, curr_path))

        # Handle extra files/directories
        for name in actual_names - expected_names:
            extra_path = str(Path(curr_path) / name)
            results[extra_path] = ComplianceLevel.EXTRA

        # Handle common files/directories
        for name in expected_names & actual_names:
            results.update(
                compare(expected.children[name], actual.children[name], curr_path)
            )

    return results


def generate_report(results: Dict[str, ComplianceLevel]) -> str:
    from collections import Counter

    lines = []
    lines.append("=" * 80)
    lines.append("Architecture Compliance Report")
    lines.append("=" * 80)

    counts = Counter(results.values())
    lines.append("Summary:")
    for level in ComplianceLevel:
        if counts[level]:
            lines.append(f"  {level.value}: {counts[level]}")
    lines.append("")

    # Detail sections
    for level in [
        ComplianceLevel.MISSING,
        ComplianceLevel.EXTRA,
        ComplianceLevel.TYPE_MISMATCH,
    ]:
        items = sorted([path for path, val in results.items() if val == level])
        if items:
            lines.append(f"{level.value}:")
            for item in items:
                lines.append(f"  {item}")
            lines.append("")

    # Compliance percentage
    total_expected = sum(
        1
        for val in results.values()
        if val
        in [
            ComplianceLevel.COMPLIANT,
            ComplianceLevel.MISSING,
            ComplianceLevel.TYPE_MISMATCH,
        ]
    )
    compliant = counts[ComplianceLevel.COMPLIANT]
    if total_expected > 0:
        percent = 100 * compliant / total_expected
        lines.append(
            f"Overall Compliance: {percent:.1f}% ({compliant}/{total_expected})"
        )

    return "\n".join(lines)


def suggest_implementation(results: Dict[str, ComplianceLevel]) -> str:
    missing = [path for path, val in results.items() if val == ComplianceLevel.MISSING]
    if not missing:
        return ""

    lines = ["Implementation priorities:"]
    # Show up to 10 missing items, prioritizing directories and core files
    prioritized = []

    # First add top-level directories (they're foundational)
    top_level_dirs = [
        item
        for item in missing
        if "/" not in item and any(child.startswith(item + "/") for child in missing)
    ]
    prioritized.extend(sorted(top_level_dirs)[:3])

    # Then add mod.rs files (they're important in Rust)
    mod_files = [item for item in missing if item.endswith("mod.rs")]
    prioritized.extend(
        [item for item in sorted(mod_files) if item not in prioritized][:5]
    )

    # Fill remaining slots with other files
    remaining = [item for item in sorted(missing) if item not in prioritized]
    prioritized.extend(remaining[: 10 - len(prioritized)])

    for item in prioritized[:10]:
        lines.append(f"  - {item}")

    if len(missing) > 10:
        lines.append(f"  ... and {len(missing) - 10} more.")

    return "\n".join(lines)


def print_extras(results: Dict[str, ComplianceLevel]) -> int:
    extras = sorted(
        [path for path, val in results.items() if val == ComplianceLevel.EXTRA]
    )
    if extras:
        print("\n==== EXTRA FILES FOUND ====")
        for extra in extras:
            print(f"  {extra}")
        print(f"Total extra files: {len(extras)}")
    return len(extras)


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Check src/ against JSON architecture specification"
    )
    parser.add_argument("--src", default="src", help="Path to src directory")
    parser.add_argument(
        "--spec",
        default="docs/architecture.json",
        help="Path to JSON architecture specification",
    )
    parser.add_argument("--debug", action="store_true", help="Enable debug output")
    args = parser.parse_args()

    spec = Path(args.spec)
    src = Path(args.src)

    try:
        expected_tree = parse_specification(spec, debug=args.debug)
    except Exception as e:
        print(f"Error: Could not parse architecture spec at {spec}: {e}")
        sys.exit(1)

    actual_tree = scan_filesystem(src)

    if not expected_tree:
        print(f"Error: Could not parse architecture spec at {spec}")
        sys.exit(1)
    if not actual_tree:
        print(f"Error: src directory not found at {src}")
        sys.exit(1)

    if args.debug:

        def debug_print_tree(node, indent=0):
            prefix = "  " * indent
            print(f"{prefix}{node.name}{'/' if node.is_dir else ''}")
            for child in node.children.values():
                debug_print_tree(child, indent + 1)

        print(f"DEBUG: Found src structure:")
        debug_print_tree(actual_tree)
        print(f"\nDEBUG: Expected structure:")
        debug_print_tree(expected_tree)
        print()

    # Compare the trees
    # If both trees have 'src' as root, compare their children instead
    if expected_tree.name == "src" and actual_tree.name == "src":
        results = {}
        # Compare all children of both trees
        expected_children = set(expected_tree.children.keys())
        actual_children = set(actual_tree.children.keys())

        for name in expected_children - actual_children:
            results.update(compare(expected_tree.children[name], None, ""))
        for name in actual_children - expected_children:
            results[name] = ComplianceLevel.EXTRA
        for name in expected_children & actual_children:
            results.update(
                compare(expected_tree.children[name], actual_tree.children[name], "")
            )
    else:
        results = compare(expected_tree, actual_tree)

    report = generate_report(results)
    print(report)

    suggestions = suggest_implementation(results)
    if suggestions:
        print("\n" + suggestions)

    # Write to file
    out_path = Path("architecture_compliance_report.txt")
    try:
        with open(out_path, "w", encoding="utf-8") as f:
            f.write(report)
            if suggestions:
                f.write("\n" + suggestions)
        print(f"\nDetailed report saved to: {out_path}")
    except Exception as e:
        print(f"Warning: Could not save report to {out_path}: {e}")

    # Print and check for extra files
    extras = print_extras(results)
    missing = sum(1 for val in results.values() if val == ComplianceLevel.MISSING)

    if missing or extras:
        print(
            f"\nArchitecture compliance check failed: {missing} missing, {extras} extra"
        )
        sys.exit(1)

    print("\nArchitecture compliance check passed!")
    sys.exit(0)


if __name__ == "__main__":
    main()
