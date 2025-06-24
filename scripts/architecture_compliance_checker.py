#!/usr/bin/env python3
"""
Architecture Compliance Checker - Clean Rewrite
Checks if filesystem structure matches self-describing JSON specification.
YAGNI/KISS principles: simple, focused, reliable.
"""

import json
import sys
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List, Set
from enum import Enum


class Status(Enum):
    OK = "✓"
    MISSING = "✗ MISSING"
    EXTRA = "! EXTRA"
    TYPE_MISMATCH = "⚠ TYPE_MISMATCH"


@dataclass
class Node:
    name: str
    is_dir: bool
    description: str = ""
    children: Dict[str, "Node"] = None

    def __post_init__(self):
        if self.children is None:
            self.children = {}


def load_spec(spec_path: Path) -> Node:
    """Load and parse self-describing JSON specification."""
    with open(spec_path) as f:
        data = json.load(f)
    return parse_node(data)


def parse_node(data: dict) -> Node:
    """Parse a single node from self-describing JSON."""
    node = Node(
        name=data["name"],
        is_dir=data["type"] == "dir",
        description=data.get("description", ""),
    )

    if node.is_dir and "content" in data:
        for child_data in data["content"]:
            child = parse_node(child_data)
            node.children[child.name] = child

    return node


def scan_filesystem(path: Path) -> Node:
    """Scan filesystem into Node structure."""
    # Use absolute path to get proper name, or use 'project_root' for current dir
    name = path.resolve().name if path.name else "project_root"
    node = Node(name=name, is_dir=path.is_dir())

    if path.is_dir():
        try:
            for child_path in sorted(path.iterdir()):
                # Skip hidden files and build artifacts
                if child_path.name.startswith(".") or child_path.name in {
                    "target",
                    "__pycache__",
                    "node_modules",
                    "build",
                    ".git",
                }:
                    continue
                child = scan_filesystem(child_path)
                node.children[child.name] = child
        except PermissionError:
            pass  # Skip inaccessible directories

    return node


def check_compliance(expected: Node, actual: Node, path: str = "") -> Dict[str, Status]:
    """Compare expected vs actual structure."""
    results = {}
    current_path = f"{path}/{expected.name}" if path else expected.name

    # Type mismatch check
    if expected.is_dir != actual.is_dir:
        results[current_path] = Status.TYPE_MISMATCH
        return results

    results[current_path] = Status.OK

    if expected.is_dir:
        expected_names = set(expected.children.keys())
        actual_names = set(actual.children.keys())

        # Missing items
        for name in expected_names - actual_names:
            missing_path = f"{current_path}/{name}"
            results[missing_path] = Status.MISSING
            # Mark all children as missing too
            _mark_missing(expected.children[name], missing_path, results)

        # Extra items
        for name in actual_names - expected_names:
            extra_path = f"{current_path}/{name}"
            results[extra_path] = Status.EXTRA

        # Common items - recurse
        for name in expected_names & actual_names:
            child_results = check_compliance(
                expected.children[name], actual.children[name], current_path
            )
            results.update(child_results)

    return results


def _mark_missing(node: Node, path: str, results: Dict[str, Status]):
    """Recursively mark all children as missing."""
    results[path] = Status.MISSING
    for child_name, child in node.children.items():
        child_path = f"{path}/{child_name}"
        _mark_missing(child, child_path, results)


def print_report(results: Dict[str, Status]):
    """Print compliance report."""
    print("=" * 60)
    print("ARCHITECTURE COMPLIANCE REPORT")
    print("=" * 60)

    # Count by status
    counts = {status: 0 for status in Status}
    for status in results.values():
        counts[status] += 1

    print("\nSUMMARY:")
    for status in Status:
        if counts[status] > 0:
            print(f"  {status.value}: {counts[status]}")

    # Show issues
    for status in [Status.MISSING, Status.EXTRA, Status.TYPE_MISMATCH]:
        items = [path for path, s in results.items() if s == status]
        if items:
            print(f"\n{status.value}:")
            for item in sorted(items)[:20]:  # Limit output
                print(f"  {item}")
            if len(items) > 20:
                print(f"  ... and {len(items) - 20} more")

    # Compliance percentage
    total = counts[Status.OK] + counts[Status.MISSING] + counts[Status.TYPE_MISMATCH]
    if total > 0:
        compliance = 100 * counts[Status.OK] / total
        print(f"\nCOMPLIANCE: {compliance:.1f}% ({counts[Status.OK]}/{total})")

    return (
        counts[Status.MISSING] + counts[Status.EXTRA] + counts[Status.TYPE_MISMATCH]
        == 0
    )


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Check filesystem against JSON architecture spec"
    )
    parser.add_argument(
        "--spec", default="architecture.json", help="JSON specification file"
    )
    parser.add_argument("--root", default=".", help="Project root directory to check")
    parser.add_argument("--output", help="Save report to file")
    args = parser.parse_args()

    spec_path = Path(args.spec)
    root_path = Path(args.root)

    # Load specification
    try:
        expected = load_spec(spec_path)
    except Exception as e:
        print(f"ERROR: Cannot load spec from {spec_path}: {e}")
        sys.exit(1)

    # Scan filesystem
    if not root_path.exists():
        print(f"ERROR: Root path {root_path} does not exist")
        sys.exit(1)

    actual = scan_filesystem(root_path)

    # Find the root to compare (handle nested structure)
    if expected.name != actual.name:
        # Look for matching child
        if expected.name in actual.children:
            actual = actual.children[expected.name]
        elif actual.name in expected.children:
            expected = expected.children[actual.name]
        else:
            print(
                f"WARNING: Root name mismatch: expected '{expected.name}', got '{actual.name}'"
            )

    # Check compliance
    results = check_compliance(expected, actual)

    # Print report
    success = print_report(results)

    # Save to file if requested
    if args.output:
        try:
            with open(args.output, "w") as f:
                import io
                import contextlib

                output = io.StringIO()
                with contextlib.redirect_stdout(output):
                    print_report(results)
                f.write(output.getvalue())
            print(f"\nReport saved to: {args.output}")
        except Exception as e:
            print(f"WARNING: Could not save to {args.output}: {e}")

    if success:
        print("\n✓ COMPLIANCE CHECK PASSED")
        sys.exit(0)
    else:
        print("\n✗ COMPLIANCE CHECK FAILED")
        sys.exit(1)


if __name__ == "__main__":
    main()
