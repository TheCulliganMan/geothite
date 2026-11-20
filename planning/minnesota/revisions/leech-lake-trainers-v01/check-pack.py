#!/usr/bin/env python3
"""Check the Leech Lake teams, their legal sources, and every other archived section."""
import argparse
from collections import Counter
from copy import deepcopy
import json
from pathlib import Path
import runpy

HERE = Path(__file__).resolve().parent
PRIOR = HERE.parent / "trail-signs-v01"
REGION = HERE.parents[1]
ROOT = REGION.parents[1]
ARCHIVE = runpy.run_path(str(HERE.parent / "ely-return-v01" / "check-pack.py"))
FIELD = runpy.run_path(str(HERE.parent / "lake-observations-v01" / "check-hidden.py"))
from audit_region_encounters import active_roster

read_pack, only = ARCHIVE["read_pack"], ARCHIVE["only"]
NORTH, SOUTH = "MnR07C00R00", "MnR07C00R01"
SLOTS = {
    "MN_R07_T05": [(1, "CHORIP", "VELVAWN")],
    "MN_R07_T07": [(0, "CHORIP", "JIGLET")],
    "MN_R07_T08": [(0, "FLOSSAIL", "DARTDUCK"), (1, "CHORIP", "FLOSSAIL")],
}


def parties(old, new, recipe):
    only(old["trainers"], new["trainers"], {"trainers"}, "trainer catalog")
    prior, revised = old["trainers"]["trainers"], new["trainers"]["trainers"]
    only(prior, revised, SLOTS.keys(), "trainer records")
    assert Counter(row["trainer"] for row in recipe) == Counter(SLOTS.keys())
    active = active_roster()
    for row in recipe:
        trainer = row["trainer"]
        expected = deepcopy(prior[trainer])
        assert len(expected["party"]) == 2
        assert [(s["index"], s["original"], s["new_member"]["species"]) for s in row["slots"]] == SLOTS[trainer]
        for s in row["slots"]:
            member = expected["party"][s["index"]]
            target = s["new_member"]
            name = target["species"]
            assert set(target) == {"species", "level", "moves", "item"}
            assert name in active and name in new["pokemon"] and member["species"] == s["original"]
            assert member["level"] == target["level"] == 30 and target["item"] is member["item"] is None
            natural = {move for level, move in new["learnsets"][name] if level <= 30}
            moves = target["moves"]
            extra = set(moves) - natural
            assert extra == ({"GIGA_DRAIN"} if name == "FLOSSAIL" else set()), (name, extra)
            assert len(moves) == len(set(moves)) == 4
            assert all(m in new["moves"] for m in moves)
            member.update(species=name, moves=[
                dict(name=m, current_pp=new["moves"][m]["pp"], pp_ups=0) for m in moves])
        assert expected == revised[trainer], ("team fields", trainer)
    # A formerly identical matchup (ignoring order) should diverge both in
    # species and in the tactical choices available to the two trainers.
    def team_fingerprint(catalog, trainer):
        return sorted((m["species"], m["level"], tuple(x["name"] for x in m["moves"])) for m in catalog[trainer]["party"])
    dean, wren = "MN_R07_T02", "MN_R07_T08"
    assert team_fingerprint(prior, dean) == team_fingerprint(prior, wren)
    teams = [team_fingerprint(revised, f"MN_R07_T{n:02}") for n in range(1, 9)]
    assert len({tuple(team) for team in teams}) == 8
    counts = [sum(m["species"] == "CHORIP" for key, tr in table.items() if key.startswith("MN_R07_T")
                  for m in tr["party"]) for table in (prior, revised)]
    assert counts == [5, 2]


def local_access(data, entries):
    """Check catchable pre-League partners and the physical shop that sells Wren's TM."""
    for room, species, level in [("MnR06C00R01", "VELVAWN", 27), (SOUTH, "TAPDUCK", 29)]:
        encounter = data["wild_encounters"][room]
        assert encounter["grass_rates"]["day"] > 0
        assert dict(species=species, level=level) in encounter["grass"]["day"]
        area = FIELD["approaches"](data, room, entries[room], "Walk")
        assert any(FIELD["quadrant"](data, room, *p) == "TALL_GRASS" for p in area)
    assert any(e["method"] == "LEVEL" and e["level"] == 16 and e["species"] == "DARTDUCK"
               for e in data["evolutions"]["TAPDUCK"])
    angler = data["maps"]["MnRiverValleyFishingGuruHouse"]
    gift = [r for r in angler["script_item_grants"] if r["item_id"] == "GOOD_ROD"]
    assert len(gift) == 1 and any(o["script"] == gift[0]["source_script"] for o in angler["objects"])
    assert any(w["target_map_constant"] == angler["attributes"]["map_constant"] for m in ("MnR03C00R00", "MnR03C00R01")
               for w in data["maps"][m]["events"]["warps"])
    group = data["fishing"]["groups"][data["maps"][SOUTH]["attributes"]["fishing_group"]]
    slots = group["rod_tables"]["GOOD_ROD"]["slots"]
    [index] = [i for i, s in enumerate(slots) if s["species"] == "JIGLET" and s["level"] == 20]
    assert group["bite_threshold"] > 0 and slots[index]["threshold"] > (slots[index - 1]["threshold"] if index else 0)
    spots = FIELD["approaches"](data, SOUTH, entries[SOUTH], "Walk")
    assert any(FIELD["quadrant"](data, SOUTH, *n) == "WATER" for p in spots for n in FIELD["adjacent"](p))
    clerk, market = "MN_MP_DEPT_5_CLERK", "MART_MN_MP_DEPT_5_CLERK"
    shop = data["maps"]["MnMinneapolisDepartmentStore5F"]
    assert any(o["script"] == clerk for o in shop["objects"])
    assert any(r["command"] == "pokemart" and r["mart_id"] == market and r["source_script"] == clerk
               for r in shop["script_shop_commands"])
    machine = data["items"]["TM_GIGA_DRAIN"]
    assert "TM_GIGA_DRAIN" in data["marts"][market] and machine["tmhm_move"] == "GIGA_DRAIN"
    assert machine["price"] == 3000 and machine["consumable"]
    assert "GIGA_DRAIN" in data["pokemon"]["FLOSSAIL"]["tmhm_learnset"] and data["moves"]["GIGA_DRAIN"]["pp"] == 5


def road(data, entries):
    floor, sight = set(), set()
    beats = set()
    for row, name in enumerate([NORTH, SOUTH]):
        room = data["maps"][name]
        a = room["attributes"]
        width, height = 2 * a["width"], 2 * a["height"]
        assert (width, height) == (64, 48)
        neighbor = SOUTH if name == NORTH else NORTH
        assert dict(direction="south" if name == NORTH else "north", target_map=neighbor, offset=0) in a["connections"]
        avoid = {(v["x"], v["y"]) for v in room["objects"] + room["events"]["warps"]}
        ground = {(x, y) for x in range(width) for y in range(height)
                  if FIELD["quadrant"](data, name, x, y) in {"FLOOR", "TALL_GRASS", "LONG_GRASS"}} - avoid
        floor.update((x, y + row * height) for x, y in ground)
        reached = FIELD["approaches"](data, name, entries[name], "Walk")
        for o in room["objects"]:
            key = o["script"]
            if o["object_type"] != "OBJECTTYPE_TRAINER":
                continue
            t = room["trainer_scripts"][key]
            assert t["trainer_id"] == key and t["trainer_class"] == data["trainers"]["trainers"][key]["trainer_class"]
            direction = o["sightline_direction_override"]
            assert direction in FIELD["DIRECTIONS"] and o["spritemovedata"] == "SPRITEMOVEDATA_STANDING_" + direction
            assert all(t[field] in room["script_text_bodies"] for field in ("seen_text", "win_text", "loss_text"))
            assert t["callback"] in room["scripts"] and t["event_flag"] not in beats
            beats.add(t["event_flag"])
            dx, dy = FIELD["DIRECTIONS"][direction]
            ray = {(o["x"] + dx * n, o["y"] + dy * n) for n in range(1, o["radius"] + 1)}
            assert ray and ray <= ground & reached, (key, "blocked or inaccessible sightline")
            sight.update((x, y + row * height) for x, y in ray)
    assert len(beats) == 8 and len(sight) == 16
    entry = {(x, y + 48) for x, y in entries[SOUTH] if y == 47}
    exit_ = {p for p in entries[NORTH] if p[1] == 0}
    assert entry == {(20, 95), (21, 95)} and exit_ == {(20, 0), (21, 0)}
    for permitted, steps in ((floor, 159), (floor - sight, 169)):
        edges = {p: FIELD["adjacent"](p) & permitted for p in permitted}
        distances = FIELD["walk"](edges, entry)
        assert min(distances[p] for p in exit_) == steps


def verify(parent, child):
    only(parent, child, {"data", "identity", "report"}, "archive sections")
    only(parent["identity"], child["identity"], {"content_hash", "runtime_modpack_id"}, "archive identity")
    only(parent["report"], child["report"], {"manifests"}, "build report")
    chain = parent["report"]["manifests"]
    assert chain[-1] == "northstar-trail-signs-v1"
    assert child["report"]["manifests"] == chain + ["northstar-leech-lake-trainers-v1"]
    assert child["identity"]["runtime_modpack_id"] == "+".join(child["report"]["manifests"])
    old, new = parent["data"], child["data"]
    only(old, new, {"trainers"}, "game data")
    parties(old, new, json.loads((HERE / "parties.json").read_text()))
    entries = FIELD["anchors"](new)
    local_access(new, entries)
    road(new, entries)
    print(f"{len(new['maps'])} maps, {len(child['runtime_files'])} embedded files and all encounter tables preserved. "
          "Three trainers changed. Eight distinct teams, sixteen reachable sight squares; optional detour adds ten steps.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("packs", nargs="*", type=Path)
    args = parser.parse_args()
    receipts = [json.loads((PRIOR / f"{edition}-build.json").read_text()) for edition in ("field", "intro")]
    if len(args.packs) == 2:
        pairs = [args.packs]
    elif not args.packs:
        pairs = [(ROOT / r["output_pack"], ROOT / r["output_pack"].replace("trail-signs", "leech-lake-trainers"))
                 for r in receipts]
    else:
        parser.error("supply a parent and its child together")
    for before, after in pairs:
        print(f"Comparing {after.name} to {before.name}", flush=True)
        verify(read_pack(before, {r["expected_pack_sha256"] for r in receipts}), read_pack(after))
