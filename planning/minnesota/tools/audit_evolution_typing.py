#!/usr/bin/env python3
"""Check continuity of type combinations in the active Minnesota roster."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / "pokemon"


def main():
    roster = json.loads((ROOT / "roster.json").read_text())
    curation = json.loads((ROOT / "roster-curation.json").read_text())
    retired = {family["id"] for family in curation["retired_families"]}
    removed = {member["id"] for member in curation["removed_evolutions"]}
    errors = []
    for family in roster["families"]:
        if family["id"] in retired:
            continue
        members = [member for member in family["members"] if member["id"] not in removed]
        if len(members) < 2:
            continue
        types = [set(member["types"]) for member in members]
        changes = sum(left != right for left, right in zip(types, types[1:]))
        if changes > 1:
            errors.append(f"{family['id']}: {changes} type changes")
        if any(not left.issubset(right) for left, right in zip(types, types[1:])):
            errors.append(f"{family['id']}: loses a type during evolution")
    if errors:
        raise SystemExit("\n".join(errors))
    print("PASS: active evolution lines retain every existing type and change typing at most once")


if __name__ == "__main__":
    main()
