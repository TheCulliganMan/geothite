"""Reject publication that disagrees with Northstar's roster and regional order."""
import argparse
import json
from pathlib import Path


def read(path):
    return json.loads(path.read_text())


def regional_order(roster, order):
    """Validate the authored family sequence and return its stable species IDs."""
    families = {family["id"]: family for family in roster["families"]}
    assert len(families) == len(roster["families"]), "Duplicate roster family"
    sections = order["sections"]
    assert len({section["id"] for section in sections}) == len(sections), "Duplicate order section"
    family_ids = [family_id for section in sections for family_id in section["families"]]
    assert len(set(family_ids)) == len(family_ids), "Family appears in multiple order sections"
    assert set(family_ids) == families.keys(), "Order sections must cover every source family"
    assert sections[0]["id"] == "starters", "Starter section must come first"
    assert [families[family_id].get("starter_role") for family_id in sections[0]["families"]] == [
        "grass", "fire", "water"
    ], "Starter families must be ordered Grass, Fire, Water"
    assert {family["id"] for family in families.values() if family.get("starter_role")} == set(
        sections[0]["families"]
    ), "Starter family appears outside the opening section"
    ranks = []
    species_ids = []
    for family_id in family_ids:
        family = families[family_id]
        special = family.get("rarity", family.get("classification", "ordinary"))
        ranks.append({"pseudo_legendary": 1, "legendary": 2, "mythical": 3}.get(special, 0))
        members = family["members"]
        assert members, f"Empty family: {family_id}"
        assert [member["stage"] for member in members] == list(range(1, len(members) + 1)), (
            f"Evolution stages must be consecutive within {family_id}")
        species_ids.extend(member["id"] for member in members)
    assert ranks == sorted(ranks), "Ordinary, pseudo-legendary, legendary and mythical families must stay in order"
    assert len(set(species_ids)) == len(species_ids), "Duplicate source species"
    assert len(species_ids) == roster["species_count"], "Source species count is stale"
    assert order["species"] == [
        {"id": species_id, "number": number} for number, species_id in enumerate(species_ids, 1)
    ], "Species numbers must follow the authored sections and evolution stages"
    return species_ids


def audit_published_order(catalog, species_ids):
    """Require both the published array and every displayed number to agree."""
    assert [species["identity"]["id"] for species in catalog["species"]] == species_ids, (
        "Published species sequence disagrees with the active regional order")
    assert [species["identity"]["regional_number"] for species in catalog["species"]] == list(
        range(1, len(species_ids) + 1)
    ), "Published species numbers disagree with the active regional order"


def audit(root, release=None):
    roster = read(root / "roster.json")
    curation = read(root / "roster-curation.json")
    tasks = read(root / "revision-tasks-20261102.json")["tasks"]
    families = {f["id"]: f for f in roster["families"]}
    retired = {f["id"] for f in curation["retired_families"]}
    assert retired <= families.keys(), "Unknown archived family"
    active = {m["id"]: (f, m) for f in families.values() if f["id"] not in retired
              for m in f["members"]}
    active_order = [species_id for species_id in regional_order(roster, read(root / "pokedex-order.json"))
                    if species_id in active]
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
        audit_published_order(catalog, active_order)
        published = {s["identity"]["id"]: s for s in catalog["species"]}
        assert set(published) == set(active), (
            f"Published active roster differs: extra={sorted(set(published)-set(active))}; "
            f"missing={sorted(set(active)-set(published))}")
        assert catalog["species_count"] == len(active), "Published count disagrees"
        assert catalog["family_count"] == curation["active_family_count"], "Published family count disagrees"
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
    print(f"PASS: {species} active species / {families} families; regional order, archives, restorations and evolution closure agree.")


if __name__ == "__main__":
    main()
