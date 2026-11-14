#!/usr/bin/env python3
"""Validate release notes or prepare a local release; never commit or tag."""
import argparse
from datetime import date
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
VERSION = re.compile(r"0|[1-9][0-9]*")


def version_tuple(value):
    parts = value.split(".")
    if len(parts) != 3 or not all(VERSION.fullmatch(part) for part in parts):
        raise ValueError("Use a version such as 0.1.1 (three nonnegative integers).")
    return tuple(map(int, parts))


def run():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["check", "release"])
    parser.add_argument("version", nargs="?")
    args = parser.parse_args()
    changelog = ROOT / "CHANGELOG.md"
    version_file = ROOT / "VERSION"
    current = version_file.read_text().strip()
    version_tuple(current)
    content = changelog.read_text()
    marker = "## Unreleased\n"
    if content.count(marker) != 1:
        raise ValueError("CHANGELOG.md must contain exactly one Unreleased section.")
    before, rest = content.split(marker)
    notes, separator, history = rest.partition("\n## ")
    if not separator:
        raise ValueError("CHANGELOG.md must contain release history.")
    headings = re.findall(r"^## (\d+\.\d+\.\d+) — (\d{4}-\d{2}-\d{2})$", content, re.M)
    if not headings or headings[0][0] != current:
        raise ValueError("VERSION must match the newest dated changelog entry.")
    versions = [version_tuple(version) for version, stamp in headings]
    for _, stamp in headings:
        date.fromisoformat(stamp)
    if any(a <= b for a, b in zip(versions, versions[1:])):
        raise ValueError("Released versions must be unique and newest first.")
    if args.command == "check":
        if args.version:
            raise ValueError("check takes no version argument.")
        print(f"Changelog and VERSION agree: {current}")
        return
    if not args.version or version_tuple(args.version) <= version_tuple(current):
        raise ValueError("The release version must be greater than VERSION.")
    if not re.search(r"^- \S", notes, re.M):
        raise ValueError("Add meaningful bullet entries under Unreleased before releasing.")
    updated = (before + marker + f"\n## {args.version} — {date.today().isoformat()}\n\n"
               + notes.strip() + "\n" + separator + history)
    changelog.write_text(updated)
    version_file.write_text(args.version + "\n")
    print(f"Prepared {args.version}. Review the diff, run checks, then commit/tag explicitly.")


if __name__ == "__main__":
    try:
        run()
    except (ValueError, OSError) as error:
        raise SystemExit(str(error)) from error
