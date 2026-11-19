#!/usr/bin/env python3
"""Read eight optional native encounters, their confirmations and their return callbacks."""

import argparse
from dataclasses import dataclass, field
import json
from pathlib import Path
import re
import sys

REGION = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, native_pages, require
from render_reference import load_pack

BATTLE = "BATTLETYPE_OPTIONAL_LEGENDARY"
PREFIX = "EVENT_MN_"
# A legacy script label is part of the save format. Species come from the current catalog.
SITES = {
    "MnBoundaryIntake": ("MN_VERDIGRIM_INTAKE", "MnVerdigrimEncounterText", "VERDIGRIM", 60,
                         ["VERDIGRIM_EXPOSED", "VERDIGRIM_CONTAINED"], "VERDIGRIM_CAUGHT"),
    "MnDevilsKettle": ("MN_DEVILS_KETTLURK", "MnKettleAskText", "KETTLURK", 60,
                       ["KETTLURK_RELEASED", "SURVEY_RESTORED"], "DEVILS_KETTLE_CAUGHT"),
    "MnFieldMargin": ("MN_MARGIN_KENSIGARD", "MnKensigardText_ask", "KENSIGARD", 50,
                      ["KENSIGARD_REVEALED"], "KENSIGARD_CAUGHT"),
    "MnInnerBasin": ("MN_BASIN_STRANDRAK", "MnBasinText_ask", "KENSIGARD", 60,
                     ["BASIN_INNER_OUTLET_ACKNOWLEDGED"], "BASIN_CAUGHT"),
    "MnLayeredFen": ("MN_FEN_FLARKINDLE", "MnFenText_ask", "MITWIST", 55,
                     ["FEN_FOLDED_PATH_REVEALED"], "FEN_CAUGHT"),
    "MnMississippiLowerWoodsC01R00": ("MN_PEPIN_PEPIVERN", "MnPepinText_ask", "BARBELISK", 55,
                                      ["PEPIN_SAFE_INLET_IDENTIFIED"], "PEPIN_CAUGHT"),
    "MnR03Cave1F": ("MN_R03_JAW_CASTOROCK", "MnJawOfferText", "CASTOROCK", 24,
                    ["CASTOROCK_TRACKS_READ"], "CASTOROCK_CAUGHT"),
    "MnR08C00R00": ("MN_ANCIENT_MIRE_VERGLOOM", "MnAncientOfferText", "VERGLOOM", 40,
                    ["ANCIENT_MIRE_TRACKS_READ"], "ANCIENT_MIRE_VERGLOOM_CAUGHT"),
}
RETIRED = {"ICEMYTH", "STRANDRAK", "FLARKINDLE", "PEPIVERN"}


@dataclass
class Visit:
    flags: set
    visible: set = field(default_factory=set)


def follow(room, start, visit, *, yes=False, captured=False):
    """Read only the instructions on this path, following local branches and native calls."""
    frames = [(start, 0)]
    trace = []
    condition = False
    for _ in range(240):
        if not frames:
            return trace
        label, index = frames[-1]
        op, *args = instructions(room, label)[index]
        frames[-1] = (label, index + 1)
        trace.append((op, *args))
        if op in ("end", "endcallback", "return", "jumptext"):
            frames.pop()
        elif op == "checkevent":
            condition = args[0] in visit.flags
        elif op == "setevent":
            visit.flags.add(args[0])
        elif op == "special":
            require(args == ["CheckCaughtCelebi"], f"unexpected battle result {args}")
            condition = captured  # Native routine reads the battle_result caught bit, 0x40.
        elif op == "yesorno":
            condition = yes
        elif op == "scall":
            frames.append((args[0], 0))
        elif op in ("sjump", "iftrue", "iffalse"):
            if op == "sjump" or condition == (op == "iftrue"):
                frames[-1] = (args[0], 0)
        elif op == "appear":
            visit.visible.add(args[0])
        elif op == "disappear":
            visit.visible.discard(args[0])
        elif op not in {
            "faceplayer", "opentext", "writetext", "promptbutton", "waitbutton", "closetext",
            "cry", "loadvar", "loadwildmon", "startbattle", "reloadmapafterbattle", "moveobject",
            "changeblock",
        }:
            raise ValueError(f"untraced {room['id']}/{label} {op}")
    raise ValueError(f"unterminated script {room['id']}/{start}")


def entry(room, visit, *, new_map):
    # The shared map_setup_callback_kinds uses NEWMAP, TILES and OBJECTS on a connection
    # or door entry. Reload after battle and Continue omit NEWMAP and OBJECTS.
    kinds = ["MAPCALLBACK_TILES"]
    if new_map:
        kinds = ["MAPCALLBACK_NEWMAP", "MAPCALLBACK_TILES", "MAPCALLBACK_OBJECTS"]
    for kind in kinds:
        for row in room["map_script_section_commands"]:
            if row["command"] == "callback" and row["args"][0] == kind:
                follow(room, row["args"][1], visit)


def text(data, room, label):
    rows = room["script_text_bodies"][label]["commands"]
    require(instructions(room, label) == [(r["command"], *r["args"]) for r in rows]
            and data["map_scripts"][label] == room["scripts"][label], f"divergent print {label}")
    return " ".join(line for page in native_pages(rows) for line in page)


def audit(pack, baseline):
    data, files = pack["data"], pack["runtime_files"]
    rooms = {k: v for k, v in data["maps"].items() if k.startswith("Mn")}
    all_actors = {(n, row["source_script"]) for n, room in rooms.items()
                  for row in room["scripted_wild_battles"]}
    tall_tale = {("MnTallTaleTrail", actor) for actor in ("MN_TALE_BABEHEMOTH", "MN_TALE_BUNJANT")}
    require(all_actors == {(m, s[0]) for m, s in SITES.items()} | tall_tale, "fixed encounter inventory")
    retired = {f["id"] for f in json.loads((REGION / "pokemon/roster-curation.json").read_text())["retired_families"]}
    allowed = {member["name"].upper() for f in json.loads((REGION / "pokemon/roster.json").read_text())["families"]
               if f["id"] not in retired for member in f["members"]}
    require(allowed.isdisjoint(RETIRED), "curated roster changed")
    obsolete = re.compile(r"\b(?:" + "|".join(RETIRED) + r")\b", re.IGNORECASE)
    for name, room in rooms.items():
        for label, body in room["script_text_bodies"].items():
            words = " ".join(arg.strip('"') for row in body["commands"] for arg in row["args"])
            require(not obsolete.search(words), f"retired species in visible {name}/{label}")
        if name in {"MnIcemythServicePassage", "MnStPaulFairExhibitionHall"}:
            require(not any("ICEMYTH" in key.upper() for key in room["scripts"]), f"live exhibit {name}")
    story = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"]
    report = {}
    for name, (actor, offer, species, level, gates, caught) in SITES.items():
        room = rooms[name]
        creature, = [a for a in room["objects"] if a["script"] == actor]
        require(species in allowed and species in data["pokemon"]
                and creature["sprite"] == "SPRITE_" + species
                and f"gfx/pokemon/{species.lower()}/overworld.2bpp" in files,
                f"incorrect native creature {name}/{actor}")
        program = instructions(room, actor)
        request, = room["scripted_wild_battles"]
        raw = request["request"]
        require((raw["battle_type"], raw["species"], raw["level"]) == (BATTLE, species, level)
                and program[request["loadwildmon_command_index"]] == ("loadwildmon", species, str(level))
                and program[request["startbattle_command_index"]] == ("startbattle",),
                f"incorrect native battle request {name}")
        prompt = text(data, room, offer)
        question = "Go closer?" if species == "BARBELISK" else "Step closer?"
        if not baseline:
            question = f"Battle {species}?"
            require(native_pages(room["script_text_bodies"][offer]["commands"]) == story[name][offer],
                    f"missing canonical pages {offer}")
        require(prompt.endswith(question) and species in prompt.removesuffix(question), f"misnamed offer {offer}")

        flags = {PREFIX + flag for flag in gates}
        caught = PREFIX + caught
        locked = Visit(set(), {actor})
        entry(room, locked, new_map=True)
        require(actor not in locked.visible, f"visible before its discovery {name}")
        ready = Visit(flags.copy())
        entry(room, ready, new_map=True)
        require(actor in ready.visible, f"hidden after its discovery {name}")
        # At the intake exposure is visible early. Its actual fight is gated by containment.
        if name == "MnBoundaryIntake":
            early = Visit({PREFIX + "VERDIGRIM_EXPOSED"})
            entry(room, early, new_map=True)
            require(actor in early.visible and ("yesorno",) not in follow(room, actor, early, yes=True),
                    "Verdigrim can battle before containment")
        refusal = follow(room, actor, ready)
        require(("writetext", offer) in refusal and ("yesorno",) in refusal and ("startbattle",) not in refusal
                and flags == ready.flags and actor in ready.visible, f"decline {name}")
        outcomes = {}
        for capture in (False, True):
            visit = Visit(flags.copy(), {actor})
            path = follow(room, actor, visit, yes=True, captured=capture)
            required = [("writetext", offer), ("yesorno",), ("cry", species), ("loadvar", "VAR_BATTLETYPE", BATTLE),
                        ("loadwildmon", species, str(level)), ("startbattle",), ("reloadmapafterbattle",),
                        ("special", "CheckCaughtCelebi")]
            indices = [path.index(op) for op in required]
            require(indices == sorted(indices) and path.count(("startbattle",)) == 1, f"battle order {name}")
            require(visit.flags == (flags | {caught} if capture else flags) and actor not in visit.visible,
                    f"wrong caught/withdrawal state {name}")
            outcomes[capture] = text(data, room, path[-1][1])
            entry(room, visit, new_map=False)
            require(actor not in visit.visible, f"immediate retry {name}")
            entry(room, visit, new_map=True)
            require((actor in visit.visible) != capture, f"later return {name}")
        require(outcomes[True] != outcomes[False], f"identical outcomes {name}")
        report[name] = (prompt.removesuffix(question), room["scripted_wild_battles"], program,
                        refusal, outcomes, creature, room["map_script_section_commands"])
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    parser.add_argument("--baseline", action="store_true", help="expect the earlier approach questions")
    parser.add_argument("--parent", type=Path, help="also compare native stages with this earlier edition")
    args = parser.parse_args()
    result = audit(load_pack(args.pack), args.baseline)
    if args.parent:
        require(result == audit(load_pack(args.parent), True),
                "original observations, actors, battle requests, refusal/aftermath or callback bindings changed")
    print("Eight regional encounters: installed roster, prints, battles, refusal, capture, withdrawal, "
          "same-visit and later return passed. Three active substitutions; retired exhibit absent.")


if __name__ == "__main__":
    main()
