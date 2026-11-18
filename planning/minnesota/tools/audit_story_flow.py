#!/usr/bin/env python3
"""Check regional story changes and their native branches without running the game."""
import argparse
from collections import Counter
import json
from pathlib import Path
import re

from render_reference import load_pack


REGION = Path(__file__).resolve().parents[1]
BADGES = {
    "ENGINE_ZEPHYRBADGE", "ENGINE_HIVEBADGE", "ENGINE_PLAINBADGE", "ENGINE_FOGBADGE",
    "ENGINE_STORMBADGE", "ENGINE_MINERALBADGE", "ENGINE_GLACIERBADGE", "ENGINE_RISINGBADGE",
}
SCRIPT_REF = {
    "sjump": (0,), "scall": (0,), "iftrue": (0,), "iffalse": (0,), "ifequal": (1,),
    "ifnotequal": (1,), "writetext": (0,), "jumptext": (0,), "jumptextfaceplayer": (0,),
    "applymovement": (1,), "winlosstext": (0, 1),
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def instructions(room, label):
    return [(r["command"], *r["args"]) for r in room["scripts"][label]]


def native_pages(rows):
    pages = []
    for row in rows:
        op, args = row["command"], row["args"]
        if op in ("text", "para"):
            pages.append([])
        elif op == "done":
            continue
        else:
            require(op == "line", f"Unexpected authored text command {op}")
        pages[-1].append(json.loads(args[0]))
    return pages


def read_branch(room, start, flags=(), accept=None):
    """Read deterministic field dialogue and follow its flag tests and local jumps."""
    label = start
    active = set(flags)
    actions = []
    texts = []
    condition = None
    for _ in range(32):
        for op, *args in instructions(room, label):
            actions.append((op, *args))
            if op in ("checkevent", "checkflag"):
                condition = args[0] in active
            elif op == "yesorno":
                require(accept is not None, f"{label} needs an explicit decision")
                condition = accept
            elif op == "setevent":
                active.add(args[0])
            elif op == "writetext":
                texts.append(args[0])
            elif op in ("jumptext", "jumptextfaceplayer"):
                return actions, [*texts, args[0]], active
            elif op == "sjump" or (op == "iftrue" and condition) or (op == "iffalse" and not condition):
                label = args[0]
                break
            elif op == "end":
                return actions, texts, active
        else:
            raise ValueError(f"Unterminated story script {room['id']}/{label}")
    raise ValueError(f"Unexpected story loop {room['id']}/{start}")


def check_story(maps):
    ranger = "EVENT_MN_BRAINERD_REPORT"
    read = "EVENT_MN_HEADWATERS_READING"
    itasca = maps["MnLakeItasca"]
    for initial, first in [((), "MnFlowItascaOwnDateText"), ((ranger,), "MN_ITASCA_MARKERText")]:
        actions, texts, final = read_branch(itasca, "MN_ITASCA_MARKER", initial)
        require(texts[0] == first and read in final, f"Itasca first visit {initial}")
        require(actions.index(("promptbutton",)) < actions.index(("setevent", read)), "Itasca reading acknowledgment")
        actions, texts, _ = read_branch(itasca, "MN_ITASCA_MARKER", final)
        require(texts == ["MnItascaMarkerAfterText"] and ("setevent", read) not in actions, "Itasca repeat")

    depot = maps["MnStPaulUnionDepot"]
    defeated = "EVENT_MN_REED_STPAUL_DEFEATED"
    date = "EVENT_MN_DEEP_BORE_MANIFEST_READ"
    courier = "EVENT_MN_COURIER_DEFEATED"
    for accepted in (False, True):
        actions, texts, final = read_branch(depot, "MnArcSPBattle", (courier,), accepted)
        require(defeated in final and (date in final) == accepted and "MnNSTPMinaReadyText" in texts,
                "Reed's defeat must resume Mina's request and preserve her decision")
        require(actions.index(("setevent", defeated)) < actions.index(("yesorno",)), "Mina must speak after Reed")
        if accepted:
            require(actions.index(("writetext", "MnNSTPMinaQuestText")) < actions.index(("setevent", date)),
                    "Mina must actually give the date")
        else:
            _, retry, final = read_branch(depot, "MN_NSTP_MINA", final, True)
            require("MnNSTPMinaQuestText" in retry and date in final, "Mina must accept a later request")
        _, again, _ = read_branch(depot, "MN_NSTP_MINA", final)
        require(again == ["MnNSTPMinaAfterText"], "Mina repeats the destination")

    receiver = maps["MnDuluthHarborWorkshop"]
    signal = "EVENT_MN_LIGHTHOUSE_SIGNAL"
    _, no_meter, prior = read_branch(receiver, "MN_HARBOR_RECEIVER")
    require(signal not in prior and "MnArcHarborIntakeText" not in no_meter, "Ivy needs the meter times")
    _, call, final = read_branch(receiver, "MN_HARBOR_RECEIVER", ("EVENT_MN_DEPOT_CLEARED",))
    require(signal in final and "MnArcHarborIntakeText" in call, "Ivy must get through without the Harbor Badge")
    _, repeat, _ = read_branch(receiver, "MN_HARBOR_RECEIVER", final)
    require(repeat == ["MN_HARBOR_RECEIVERDoneText"], "engineer must keep the rescue moving")

    shelter = maps["MnBoundaryWatersSurveyShelter"]
    require(("startbattle",) not in read_branch(shelter, "MN_SURVEY_CHIEF")[0], "Hale needs the signal")
    actions, _, final = read_branch(shelter, "MN_SURVEY_CHIEF", (signal,))
    require(("startbattle",) in actions and "EVENT_MN_SURVEY_CHIEF_DEFEATED" in final,
            "Hale must yield the receiver with the signal and no badges")
    league = instructions(maps["MnLeagueLobby"], "MnElyRegister")
    require({a[1] for a in league if a[0] == "checkflag"} >= BADGES
            and ("checkevent", "EVENT_MN_SURVEY_RESTORED") in league, "Ely must retain eight badges and rescue")

    drill = "EVENT_MN_VERDIGRIM_DRILL_STOPPED"
    outlet = "EVENT_MN_VERDIGRIM_OUTLET_CLOSED"
    restored = "EVENT_MN_SURVEY_RESTORED"
    for flags, text in [
        ((), "MnArcIntakeFinnText"), ((drill,), "MnFlowFinnOutletText"),
        ((drill, outlet), "MnFlowFinnBypassText"), ((drill, outlet, restored), "MnFlowFinnRestoredText"),
    ]:
        _, texts, _ = read_branch(maps["MnBoundaryIntake"], "MN_ARC_INTAKE_FINN", flags)
        require(texts == [text], f"Finn must recognize {flags}")


def check_trainers(before, after):
    total = roads = revised = 0
    for obj in after["objects"]:
        if obj["object_type"] != "OBJECTTYPE_TRAINER":
            continue
        script = obj["script"]
        battle = after["trainer_scripts"][script]
        total += 1
        require(obj["radius"] > 0, f"{after['id']}/{script} sight encounter")
        callback = battle["callback"]
        new_callback = instructions(after, callback)
        win = battle["win_text"]
        new = f"{script}ReturnText"
        victory = native_pages(after["script_text_bodies"][win]["commands"])
        if new in after["scripts"]:
            revised += 1
            require(new_callback == [("endifjustbattled",), ("jumptextfaceplayer", new)], f"{callback} return trigger")
            require(new_callback != instructions(before, callback), f"{callback} already used new speech")
            later = native_pages(after["script_text_bodies"][new]["commands"])
            require(later != victory, f"{callback} repeats the victory speech")
        else:
            require(new_callback == instructions(before, callback), f"{callback} unchanged trainer return")
        if re.fullmatch(r"MN_R\d{2}_T\d{2}", script):
            roads += 1
            require(1 <= len(victory) <= 2 and new in after["scripts"], f"numbered-route trainer {script}")
    return total, roads, revised


def run(parent, child, dialogue):
    old, new = parent["data"], child["data"]
    migration = "data/northstar-save-compatibility.json"
    runtime = parent["runtime_files"]
    updated = child["runtime_files"]
    require(runtime.keys() == updated.keys(), "changed runtime file roster")
    require(all(runtime[k] == updated[k] for k in runtime.keys() - {migration}), "changed runtime art or assets")
    old_save = json.loads(bytes(runtime[migration]))
    new_save = json.loads(bytes(updated[migration]))
    require(old_save["schema_version"] == 3 and new_save["schema_version"] == 4 and
            len(new_save["sources"]) == len(old_save["sources"]) + 2 and
            new_save["sources"][2:] == old_save["sources"], "older save lineage")
    release = json.loads((REGION / "world.json").read_text())["regional_floorplans_revision"]
    for index, prefix in enumerate(("", "hosted_")):
        require(new_save["sources"][index] == {
            "modpack_id": old_save["target_modpack_ids"][index],
            "modpack_hash": release[f"{prefix}pack_sha256"],
            "content_hash": release["content_identity" if not prefix else "hosted_content_identity"],
        }, f"exact {prefix}floorplan predecessor")
    for key in old_save.keys() - {"schema_version", "target_manifest", "target_modpack_ids", "sources"}:
        require(old_save[key] == new_save[key], f"changed earlier save migration {key}")
    for key in parent["report"].keys() - {"manifests"}:
        require(parent["report"][key] == child["report"][key], f"changed build report {key}")
    # The raw revision is appended; hosted builds then reapply their original clock
    # and appearance publishers. Both retain the prior order around this insertion.
    added = new_save["target_manifest"]
    before = parent["report"]["manifests"]
    wrappers = ["realtime-clock", "player-customization"]
    expected = before[:-2] + [added] + wrappers if before[-2:] == wrappers else before + [added]
    require(added == "northstar-story-flow-v1" and added not in before and
            child["report"]["manifests"] == expected, "release manifest")
    outputs = {"identity", "report", "data", "runtime_files"}
    require(parent.keys() == child.keys() and old.keys() == new.keys(), "changed pack sections")
    for key in parent.keys() - outputs:
        require(parent[key] == child[key], f"non-story pack section {key}")
    for key in old.keys() - {"maps", "map_scripts"}:
        require(old[key] == new[key], f"non-story game data {key}")
    require(old["maps"].keys() == new["maps"].keys(), "map roster")
    modified = set()
    total = roads = revised = region = 0
    inventory_ops = {"giveitem", "takeitem", "checkitem", "hiddenitem", "verbosegiveitem", "itemball"}
    for name, before in old["maps"].items():
        after = new["maps"][name]
        if not name.startswith("Mn"):
            require(before == after, f"unrelated room {name}")
            continue
        region += 1
        for key in ("attributes", "blocks", "events", "objects", "scenes", "map_event_section_commands",
                    "map_script_section_commands", "trainer_scripts", "script_field_pickups"):
            require(before[key] == after[key], f"{name} changed {key}")
        old_scripts, scripts = before["scripts"], after["scripts"]
        require(old_scripts.keys() <= scripts.keys(), f"{name} lost scripts")
        fresh = scripts.keys() - old_scripts.keys()
        if fresh:
            reachable = set()
            # Map roots are unchanged. New subprograms must be reachable from
            # a pre-existing program, directly or through another new label.
            def links(label):
                return {arg for row in scripts[label] for arg in row["args"] if arg in fresh}

            frontier = set().union(*(links(label) for label in old_scripts))
            while frontier:
                label = frontier.pop()
                if label not in reachable:
                    reachable.add(label)
                    frontier.update(links(label) - reachable)
            require(reachable == fresh, f"{name} unreachable additions {sorted(fresh - reachable)}")
        previous_items = Counter()
        current_items = Counter()
        for label, rows in scripts.items():
            previous = old_scripts.get(label)
            if previous == rows:
                continue
            modified.add(label)
            where = f"{name}/{label}"
            require(new["map_scripts"].get(label) == rows, f"stale global script {where}")
            for commands, counts in ((previous or [], previous_items), (rows, current_items)):
                counts.update((r["command"], *r["args"]) for r in commands if r["command"] in inventory_ops)
            for row in rows:
                for index in SCRIPT_REF.get(row["command"], ()):
                    reference = row["args"][index]
                    require(reference in scripts or reference in new["map_scripts"], f"dangling {where} -> {reference}")
        require(previous_items == current_items, f"{name} changed item/HM receipts or pocket checks")
        for label, pages in dialogue.get(name, {}).items():
            require(all(1 <= len(p) <= 2 and all(0 < len(s) <= 18 and s.isascii() for s in p) for p in pages),
                    f"invalid source pages {name}/{label}")
            require(native_pages(after["script_text_bodies"][label]["commands"]) == pages, f"compiled {name}/{label}")
            require(native_pages(scripts[label]) == pages, f"source rows {name}/{label}")
        n, r, h = check_trainers(before, after)
        total += n
        roads += r
        revised += h
    require(dialogue.keys() <= new["maps"].keys(), "uninstalled authoring map")
    require((region, total, roads, revised) == (355, 301, 125, 276), f"unexpected release coverage {(region, total, roads, revised)}")
    require(old["map_scripts"].keys() <= new["map_scripts"].keys(), "global scripts removed")
    for k in new["map_scripts"].keys() - modified:
        require(old["map_scripts"].get(k) == new["map_scripts"][k], f"unrelated global program {k}")
    check_story(new["maps"])
    print(f"Verified {region} Minnesota rooms, {sum(map(len, dialogue.values()))} authored texts, "
          f"{total} sight trainers, {roads} numbered-route wins, {revised} new trainer returns, and the rescue branches.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    dialogue = json.loads((REGION / "story.json").read_text())["story_flow_revision"]["dialogue"]
    run(load_pack(args.parent), load_pack(args.child), dialogue)


if __name__ == "__main__":
    main()
