"""Reject publication that disagrees with Northstar's archive/restoration decisions."""
import argparse
import json
from pathlib import Path


def read(path):
    return json.loads(path.read_text())


def audit(root, release=None):
    roster = read(root / "roster.json")
    curation = read(root / "roster-curation.json")
    tasks = read(root / "revision-tasks-20261102.json")["tasks"]
    families = {f["id"]: f for f in roster["families"]}
    retired = {f["id"] for f in curation["retired_families"]}
    assert retired <= families.keys(), "Unknown archived family"
    active = {m["id"]: (f, m) for f in families.values() if f["id"] not in retired
              for m in f["members"]}
    assert len(active) == curation["active_species_count"], "Active count is stale"
    assert len(families) - len(retired) == curation["active_family_count"], "Family count is stale"
    assert "MN_107" in active, "Tussnip must stay active"
    for task in tasks:
        if task.get("kind") == "archive":
            for target in task.get("targets", []):
                assert target["species_id"] not in active, (
                    f"{task['id']}: {target['name']} was requested archived")
    # Restoration requests include a typing-and-sprite task as well as restore tasks.
    for restoration in curation["restored_families"]:
        family = families[restoration["id"]]
        for member in family["members"]:
            assert member["id"] in active, f"Requested restoration missing: {member['name']}"
    for removed in curation["removed_evolutions"]:
        assert removed["id"] not in active, f"Removed final stage active: {removed['name']}"
    for restored in curation.get("restored_evolutions", []):
        assert restored["id"] in active, f"Restored final stage missing: {restored['name']}"
        family = active[restored["id"]][0]
        assert family["id"] == restored["family_id"], "Restored final stage in wrong family"
        assert any(m.get("evolves_to") == restored["id"] for m in family["members"]), "Restored final stage disconnected"
    assert "MN_F127" in retired and "MN_F136" in retired, "Both old bear lines must remain archived"
    for family, member in active.values():
        target = member.get("evolves_to")
        if target is not None:
            assert target in active, f"Active evolution enters archive: {member['name']}"
            assert active[target][0]["id"] == family["id"], "Evolution leaves its family"
    if release is not None:
        catalog = read(release / "pokedex.json")
        published = {s["identity"]["id"]: s for s in catalog["species"]}
        assert set(published) == set(active), (
            f"Published active roster differs: extra={sorted(set(published)-set(active))}; "
            f"missing={sorted(set(active)-set(published))}")
        assert catalog["species_count"] == len(active), "Published count disagrees"
        assert catalog["family_count"] == curation["active_family_count"], "Published family count disagrees"
        numbers = sorted(s["identity"]["regional_number"] for s in published.values())
        assert numbers == list(range(1, len(active) + 1)), "Published numbering has gaps/duplicates"
        archived = {s["identity"]["id"] for s in catalog["cut_species"]}
        assert not (archived & published.keys()), "Species is both active and archived"
        for task in tasks:
            if task.get("kind") == "archive":
                for target in task.get("targets", []):
                    assert target["species_id"] in archived, f"Archive record lost: {target['name']}"
        for species_id, (_, member) in active.items():
            assert published[species_id]["identity"].get("evolves_to") == member.get("evolves_to"), (
                f"Stale published evolution: {member['name']}")
    return len(active), curation["active_family_count"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1] / "pokemon")
    parser.add_argument("--release", type=Path)
    args = parser.parse_args()
    species, families = audit(args.root, args.release)
    print(f"PASS: {species} active species / {families} families; archives, restorations and evolution closure agree.")


if __name__ == "__main__":
    main()
