#!/usr/bin/env python3
"""Verify Research Choices against the exact Marais Supplies archive for each edition."""
import argparse
import json
from pathlib import Path
import runpy

HERE = Path(__file__).resolve().parent
PREVIOUS = HERE.parent / "marais-supplies-v01"
SHARED = runpy.run_path(str(PREVIOUS / "check-pack.py"))
read_pack, only, native_pages = (SHARED[k] for k in ("read_pack", "only", "native_pages"))
program = runpy.run_path(str(PREVIOUS / "check-aide.py"))["program"]
ROOT, REGION = SHARED["ROOT"], SHARED["REGION"]
WINONA, GALLERY = "MnWinonaFieldStation", "MnDriftlessLampGallery"
ENTRY, OLD_SCAN, QUESTION = "MnEarlyPartners", "MnEarlyPartnerPOUCHID", "MnResearchChoiceText"
CACHE, HOLD = "MN_LAMP_GALLERY_CACHE", "MnLampCacheBagFull"
STARTERS = {"POUCHID": "CHIKORITA", "KINDLIT": "CYNDAQUIL", "BIRCHIT": "TOTODILE"}
DELIVERED = "EVENT_MN_DELIVERED_ROCHESTER_PARCEL"
TABLES = {
    "script_control_commands": {"sjump", "ifequal", "iffalse", "iftrue", "end"},
    "script_text_commands": {"opentext", "closetext", "writetext", "waitbutton"},
    "script_flag_commands": {"checkevent"},
}


def claim(name):
    return f"EVENT_MN_{name}_RESEARCH_RECEIVED"


def picked(name):
    return f"EVENT_MN_WINONA_{STARTERS[name]}"


def chooser(name):
    return "MnResearchChoiceAfter" + name.title()


def indexed_body(rows, label):
    return dict(label=label, commands=[dict(r, command_index=i) for i, r in enumerate(rows)])


def check_room(raw_before, raw_after, name, labels, text, extra_tables=()):
    before, after = raw_before["maps"][name], raw_after["maps"][name]
    typed = {"script_control_commands", "script_text_commands"}
    if name == WINONA:
        typed.add("script_flag_commands")
    only(before, after, {"scripts", "script_text_bodies"} | typed | set(extra_tables), name)
    only(before["scripts"], after["scripts"], labels, f"{name} raw programs")
    assert labels - before["scripts"].keys() == labels - {CACHE if name == GALLERY else ENTRY}
    for label in labels:
        assert after["scripts"][label] == raw_after["map_scripts"][label], label
    only(before["script_text_bodies"], after["script_text_bodies"], {text}, name + " pages")
    canonical = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"][name]
    rows = after["scripts"][text]
    assert native_pages(rows) == canonical[text] and rows[-1]["command"] == "done", text
    assert after["script_text_bodies"][text] == indexed_body(rows, text)
    for table in typed:
        prior = [r for r in before[table] if r["source_script"] not in labels]
        assert prior == [r for r in after[table] if r["source_script"] not in labels], table
        expected = []
        for label in sorted(labels):
            for index, (op, *args) in enumerate(program(after["scripts"][label])):
                if op not in TABLES[table]:
                    continue
                row = dict(command=op, source_script=label, command_index=index)
                if table == "script_control_commands":
                    target = args[-1] if args else None
                    assert target is None or target in after["scripts"], (label, target)
                    row.update(target_label=target, resolved_target_script=target,
                               compare_value=args[0] if len(args) == 2 else None)
                elif table == "script_text_commands":
                    assert not args or args[0] in after["script_text_bodies"]
                    row.update(text_label=args[0] if args else None)
                else:
                    row.update(flag_id=args[0])
                expected.append(row)
        actual = [r for r in after[table] if r["source_script"] in labels]
        assert sorted(actual, key=lambda r: (r["source_script"], r["command_index"])) == expected, table
    return before, after


def gallery(old, new):
    labels = {CACHE, HOLD, HOLD + "Text"}
    before, after = check_room(old, new, GALLERY, labels, HOLD + "Text")
    expected = program(before["scripts"][CACHE])
    i = expected.index(("giveitem", "RARE_CANDY", "1")) + 1
    assert expected[i] == ("iffalse", "MnLampCacheClose")
    expected[i] = ("iffalse", HOLD)
    assert program(after["scripts"][CACHE]) == expected
    assert program(after["scripts"][HOLD]) == [
        ("writetext", HOLD + "Text"), ("waitbutton",), ("sjump", "MnLampCacheClose")]
    assert program(after["scripts"]["MnLampCacheClose"]) == [("closetext",), ("end",)]
    # These two guards protect the successful collection and its clue prerequisite.
    assert expected[:4] == [
        ("checkevent", "EVENT_MN_LAMP_GALLERY_CACHE"), ("iftrue", "MnLampCacheEnd"),
        ("checkevent", "EVENT_MN_LAMP_GALLERY_READ"), ("iffalse", "MnLampTooDark")]
    assert expected[i + 1:] == [
        ("setevent", "EVENT_MN_LAMP_GALLERY_CACHE"), ("playsound", "SFX_ITEM"), ("waitsfx",),
        ("writetext", CACHE + "ClaimReceipt5Text"), ("waitbutton",), ("disappear", CACHE),
        ("sjump", "MnLampCacheClose")]
    return labels


def next_offer(room, flags, result):
    """Follow the chooser until the player sees the old, unchanged partner question."""
    script, value, options = ENTRY, None, None
    for _ in range(20):
        for index, (op, *args) in enumerate(program(room["scripts"][script])):
            if op == "checkevent":
                value = args[0] in flags
            elif op == "verticalmenu":
                options = room["script_vertical_menus"][f"{script}:{index}"]["options"]
                value = result
            elif op == "yesorno":
                return options, script
            elif op in ("jumptext", "end"):
                return options, args[0] if args else None
            elif (op == "sjump" or (op == "iftrue" and value) or (op == "iffalse" and not value)
                  or (op == "ifequal" and value == int(args[0]))):
                script = args[-1]
                break
            else:
                assert op in ("iftrue", "iffalse", "ifequal", "opentext", "writetext", "loadmenu",
                              "closewindow", "closetext"), (script, op)
        else:
            raise AssertionError(("unterminated", script))
    raise AssertionError(("research chooser loop", script))


def winona(old, new):
    roots = {chooser(s) for s in STARTERS}
    definitions = {root + suffix for root in roots for suffix in ("Header", "MenuData")}
    labels = {ENTRY, QUESTION} | roots | definitions
    before, after = check_room(old, new, WINONA, labels, QUESTION,
                               ("script_menu_definitions", "script_vertical_menus", "script_runtime_commands"))
    raw = after["scripts"]
    initial = program(before["scripts"][ENTRY])
    assert initial == [("checkevent", DELIVERED), ("iffalse", "MnEarlyPartnerWait"), ("sjump", OLD_SCAN)]
    dispatch = [row for s in STARTERS for row in (("checkevent", picked(s)), ("iftrue", chooser(s)))]
    assert program(raw[ENTRY]) == initial[:2] + dispatch + initial[2:]
    menu_defs = after["script_menu_definitions"]
    only(before["script_menu_definitions"], menu_defs, definitions, "menu headers and data")
    vertical = set()
    commands = []
    for starter in STARTERS:
        root = chooser(starter)
        choices = [s for s in STARTERS if s != starter]
        offer = "MnEarlyPartner" + starter
        existing = program(raw[offer])
        gift = existing.index(("givepoke", starter, "5"))
        assert existing[gift - 3:gift + 3] == [
            ("readvar", "VAR_PARTYCOUNT"), ("ifequal", "6", "MnEarlyPartnerFull"), ("closetext",),
            ("givepoke", starter, "5"), ("ifequal", "2", offer + "Failed"), ("setevent", claim(starter))]
        assert existing[6:8] == [("yesorno",), ("iffalse", "MnEarlyPartnerClose")]
        assert [(r["command_index"], r["species_id"], r["level"]) for r in after["gift_pokemon_scripts"]
                if r["source_script"] == offer] == [(gift, starter, 5)]
        header, data = root + "Header", root + "MenuData"
        rows = program(raw[root])
        guards = [op for s in choices for op in (("checkevent", claim(s)), ("iftrue", OLD_SCAN))]
        assert rows == guards + [
            ("opentext",), ("writetext", QUESTION), ("loadmenu", header), ("verticalmenu",),
            ("closewindow",), ("closetext",),
            ("ifequal", "1", "MnEarlyPartner" + choices[0]),
            ("ifequal", "2", "MnEarlyPartner" + choices[1]), ("end",)]
        load, show = rows.index(("loadmenu", header)), rows.index(("verticalmenu",))
        commands.extend(dict(row, source_script=root, command_index=i)
                        for i, row in enumerate(raw[root]) if row["command"] in ("loadmenu", "verticalmenu", "closewindow"))
        expected_options = choices + ["CANCEL"]
        assert program(raw[data]) == [("db", "STATICMENU_CURSOR"), ("db", "3")] + [
            ("db", f'"{name}@"') for name in expected_options]
        h = program(raw[header])
        assert h[0] == ("db", "MENU_BACKUP_TILES") and h[2:] == [("dw", data), ("db", "1")]
        assert h[1][0] == "menu_coords"
        left, top, right, bottom = map(int, h[1][1:])
        assert 0 <= left < right <= 19 and 0 <= top < bottom <= 17
        assert right - left >= 2 + max(map(len, expected_options)) and bottom - top >= 6
        assert all(menu_defs[k] == indexed_body(raw[k], k) for k in (header, data))
        key = f"{root}:{show}"
        vertical.add(key)
        assert after["script_vertical_menus"][key] == dict(source_script=root, loadmenu_command_index=load,
            verticalmenu_command_index=show, header_label=header, data_label=data, options=expected_options,
            two_dimensional=False, rows=None, columns=None, spacing=None)
        flags = {DELIVERED, picked(starter)}
        assert next_offer(after, flags - {DELIVERED}, 1) == (None, "MnEarlyPartnerWaitText")
        for result in range(4):
            target = "MnEarlyPartner" + choices[result - 1] if result in (1, 2) else None
            assert next_offer(after, flags, result) == (expected_options, target), (starter, result)
        for received, remaining in (choices, choices[::-1]):
            assert next_offer(after, flags | {claim(received)}, 0) == (None, "MnEarlyPartner" + remaining)
        assert next_offer(after, flags | {claim(s) for s in choices}, 0) == (None, "MnEarlyPartnerDoneText")
    only(before["script_vertical_menus"], after["script_vertical_menus"], vertical, "vertical menus")
    previous = before["script_runtime_commands"]
    current = after["script_runtime_commands"]
    assert current[:len(previous)] == previous
    key = lambda r: (r["source_script"], r["command_index"])
    assert sorted(current[len(previous):], key=key) == sorted(commands, key=key)
    assert next_offer(after, {DELIVERED}, 0) == (None, OLD_SCAN)
    return labels


def verify(parent, child):
    only(parent, child, {"data", "identity", "report"}, "archive sections")
    only(parent["identity"], child["identity"], {"content_hash", "runtime_modpack_id"}, "identity")
    only(parent["report"], child["report"], {"manifests"}, "build report")
    manifests = parent["report"]["manifests"]
    assert manifests[-1] == "northstar-marais-supplies-v1"
    assert child["report"]["manifests"] == manifests + ["northstar-research-choices-v1"]
    assert child["identity"]["runtime_modpack_id"] == "+".join(child["report"]["manifests"])
    old, new = parent["data"], child["data"]
    only(old, new, {"maps", "map_scripts"}, "game data")
    only(old["maps"], new["maps"], {WINONA, GALLERY}, "maps")
    labels = gallery(old, new) | winona(old, new)
    only(old["map_scripts"], new["map_scripts"], labels, "global script index")
    print(f"{len(old['maps'])} maps and {len(parent['runtime_files'])} assets checked. "
          "All three starter choices, remaining offers, cancellation and Gallery retry verified.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("packs", nargs="*", type=Path)
    pairs = parser.parse_args().packs
    receipts = [json.loads((PREVIOUS / f"{edition}-build.json").read_text()) for edition in ("field", "intro")]
    if len(pairs) == 2:
        comparisons = [pairs]
    elif not pairs:
        comparisons = [(ROOT / r["output_pack"], ROOT / r["output_pack"].replace("marais-supplies", "research-choices"))
                       for r in receipts]
    else:
        parser.error("supply one parent and its child together")
    for p, c in comparisons:
        print(f"Comparing {c.name} to {p.name}", flush=True)
        verify(read_pack(p, {r["expected_pack_sha256"] for r in receipts}), read_pack(c))
