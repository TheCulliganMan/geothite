#!/usr/bin/env python3
"""Check the physical control faces and both first-visit routes in an installed gym archive."""
import argparse
from collections import deque
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import instructions, native_pages
from render_reference import load_pack

gym = runpy.run_path(str(HERE.parent / "trail-battles-v01/check-trainers.py"))
coords, adjacent, run = (gym[k] for k in ("coords", "adjacent", "run"))
MPLS, CLOUD = "MnMinneapolisGym", "MnStCloudGym"
# Each original read is followed by the additional, exposed faces of the same device.
CABINETS = {
    MPLS: {"WEST_FEEDER": ((5, 14), (4, 14), (4, 15), (5, 15)),
           "EAST_FEEDER": ((8, 7), (9, 6), (9, 7))},
    CLOUD: {"LOWER_WEST_BANK": ((6, 13), (7, 13)), "LOWER_EAST_BANK": ((13, 13), (12, 13)),
            "UPPER_WEST_BANK": ((6, 5), (7, 5)), "UPPER_EAST_BANK": ((14, 3), (15, 3))},
}
NEW_TEXT = {"MnGymWalkWestFeederLockedText": [["Beat LENVI to use", "the west breaker."]],
            "MnGymWalkEastFeederLockedText": [["Beat JOSSA to use", "the east breaker."]],
            "MnGymWalkLenviBreakerText": [["My breaker powers", "the west bridge."],
                                          ["Give it a flip.", "JOSSA's across."]]}


class Visit:
    def __init__(self, data, name):
        self.room = data["maps"][name]
        self.name = name
        self.tileset = data["tilesets"][self.room["attributes"]["tileset_name"]]
        self.actors = {o["script"]: coords(o) for o in self.room["objects"]}
        self.flags = set()
        (self.callback,) = [r["args"][1] for r in self.room["map_script_section_commands"]
                            if r["command"] == "callback" and r["args"][0] == "MAPCALLBACK_TILES"]
        self.arrival = (6, 19) if name == MPLS else (4, 15)
        self.at = self.arrival
        self.lengths = []
        self.refresh()

    def refresh(self):
        self.blocks = self.room["blocks"].copy()
        run(self.room, self.callback, self.flags, self.blocks)

    def kind(self, at):
        x, y = at
        attr = self.room["attributes"]
        if not (0 <= x < attr["width"] * 2 and 0 <= y < attr["height"] * 2):
            return "WALL"
        block = self.blocks[y // 2 * attr["width"] + x // 2]
        return self.tileset["collision"][f"{block:02x}"][y % 2 * 2 + x % 2]

    def walk(self, goals, avoid=()):
        blocked = set(self.actors.values()) | set(avoid)
        dry = {"FLOOR", "WALK_LEFT", "WALK_RIGHT", "HOP_DOWN", "WARP_CARPET_DOWN"}
        queue, distances = deque([self.at]), {self.at: 0}
        while queue:
            pos = queue.popleft()
            if pos in goals:
                self.at = pos
                self.lengths.append(distances[pos])
                return
            for dx, dy in gym["D"].values():
                scale = 2 if self.kind(pos) == "HOP_DOWN" and dy == 1 else 1
                to = (pos[0] + dx * scale, pos[1] + dy * scale)
                if to in blocked or self.kind(to) not in dry:
                    continue
                while self.kind(to) in {"WALK_LEFT", "WALK_RIGHT"}:
                    side = -1 if self.kind(to) == "WALK_LEFT" else 1
                    nxt = (to[0] + side, to[1])
                    if self.kind(nxt) not in dry or nxt in blocked:
                        break
                    to = nxt
                if (to not in distances and
                        (self.kind(to) != "WARP_CARPET_DOWN" or to in goals)):
                    distances[to] = distances[pos] + 1
                    queue.append(to)
        raise AssertionError((self.name, self.at, goals, self.flags))

    def fight(self, actor):
        obj = next(o for o in self.room["objects"] if o["script"] == actor)
        ray = gym["rays"](obj, self.actors[actor], set(self.actors.values()), self.kind)
        self.walk(ray)
        dx, dy = gym["D"][gym["bearing"](obj)]
        self.actors[actor] = (self.at[0] - dx, self.at[1] - dy)
        self.flags.add(self.room["trainer_scripts"][actor]["event_flag"])
        self.refresh()

    def press(self, face, choice=1):
        assert face in adjacent(self.at) and self.kind(face) == "WALL", (self.at, face)
        (event,) = [e for e in self.room["events"]["bg_events"] if coords(e) == face]
        assert event["event_type"] == "BGEVENT_READ"
        run(self.room, event["script"], self.flags, self.blocks, choice)


def faces(data, name, baseline):
    visit = Visit(data, name)
    room = visit.room
    bg = room["events"]["bg_events"]
    expected = {}
    for suffix, targets in CABINETS[name].items():
        for at in (targets[:1] if baseline else targets):
            expected[at] = name + suffix + "Routing"
            assert visit.kind(at) == "WALL"
    assert {coords(e):e["script"] for e in bg} == expected and len(bg) == len(expected)
    native = room["map_event_section_commands"]
    assert {tuple(r["args"]) for r in native if r["command"] == "bg_event"} == {
        (str(x), str(y), "BGEVENT_READ", label) for (x,y), label in expected.items()}
    assert data["map_scripts"][room["attributes"]["map_events_label"]] == [
        {"command":r["command"], "args":r["args"]} for r in native]
    assert data["npcs"][name] == room["objects"]
    assert visit.actors[name+"Guide"] == ((9,19) if name == MPLS else ((7,14) if baseline else (5,14)))
    if name == MPLS:
        for label, pages in NEW_TEXT.items():
            if baseline:
                assert label not in room["scripts"]
            else:
                raw = room["scripts"][label]
                assert native_pages(raw) == pages and raw == data["map_scripts"][label]
                assert room["script_text_bodies"][label]["commands"] == [
                    dict(row, command_index=i) for i, row in enumerate(raw)]
        def bound(script, index, opcode, label):
            assert instructions(room, script)[index] == (opcode, label)
            assert {"command":opcode, "text_label":label, "source_script":script, "command_index":index} in room["script_text_commands"]
        for bank, dest in [("WEST", "MnGymWalkWestFeederLockedText"), ("EAST", "MnGymWalkEastFeederLockedText")]:
            for mode in ("ON", "OFF"):
                gate = f"{MPLS}{bank}_FEEDERRouting{mode}Denied"
                bound(gate, 0, "writetext", gate+"Text" if baseline else dest)
        after = "MN_GYM_3_TRAINER_1After"
        bound(after, 1, "jumptext", after+"Text" if baseline else "MnGymWalkLenviBreakerText")
    # Use the open bridges/catwalk to reach every exposed face, even when a
    # shutter would normally hide that approach on the first visit.
    opened = Visit(data, name)
    slot = 3 if name == MPLS else 5
    opened.flags.update(f"EVENT_MN_GYM_{slot}_TRAINER_{i}_DEFEATED" for i in (1, 2))
    if name == MPLS:
        opened.flags.update(f"EVENT_MN_GYM_3_V2_{bank}_FEED_ON" for bank in ("WEST", "EAST"))
    opened.refresh()
    for face in expected:
        opened.walk(adjacent(face))
    print(f"{name}: all {len(expected)} control faces have a reachable A-button stance.")
    return visit


def trace(data, baseline):
    m = faces(data, MPLS, baseline)
    m.walk({(4,16)})
    if not baseline:
        m.press((4,15))
        assert not any(f.endswith("_FEED_ON") for f in m.flags)
    m.fight("MN_GYM_3_TRAINER_1")
    m.walk({(5,13)} if baseline else {(4,16)})
    m.press((5,14) if baseline else (4,15))
    assert "EVENT_MN_GYM_3_V2_WEST_BRIDGE_OPEN" in m.flags and "EVENT_MN_GYM_3_COURT_OPEN" not in m.flags
    m.fight("MN_GYM_3_TRAINER_2")
    # JOSSA steps to (9,8) on sight; the open east aisle reaches the same controls.
    m.walk({(8,8)} if baseline else {(10,7)})
    m.press((8,7) if baseline else (9,7))
    assert "EVENT_MN_GYM_3_COURT_OPEN" in m.flags
    m.walk(adjacent(m.actors[MPLS+"Leader"]))
    m.walk({(6,21)})
    print("Minneapolis: arrival, west breaker, LENVI, west power, JOSSA, east power, court, exit;",
          "walking inputs by leg:", m.lengths)

    for hallen in [False, True]:
        c = faces(data, CLOUD, baseline)
        guide = c.actors[CLOUD+"Guide"]
        c.walk(adjacent(guide))
        c.fight("MN_GYM_5_TRAINER_1")
        c.walk({(5,13)} if baseline else {(7,14)})
        c.press((6,13) if baseline else (7,13), 1)
        assert "EVENT_MN_GYM_5_V2_RELAY_EAST" in c.flags and "EVENT_MN_GYM_5_COURT_OPEN" not in c.flags
        if hallen:
            c.fight("MN_GYM_5_TRAINER_2")
            assert "EVENT_MN_GYM_5_COURT_OPEN" in c.flags
        else:
            c.walk({(14,4)} if baseline else {(15,4)}, avoid={(12,6),(13,6)})
            c.press((14,3) if baseline else (15,3), 2)
            assert "EVENT_MN_GYM_5_V2_RELAY_WEST" in c.flags and "EVENT_MN_GYM_5_COURT_OPEN" in c.flags
        c.walk(adjacent(c.actors[CLOUD+"Leader"]), avoid=() if hallen else {(12,6),(13,6)})
        if not hallen:
            # Return from the leader's bank via the west console; the menu really reverses both runs.
            c.walk({(5,5)})
            c.press((6,5), 1)
            c.walk({(13,14)}, avoid={(12,6),(13,6)})
            c.press((13,13), 2)
        c.walk({(4,17)}, avoid=() if hallen else {(12,6),(13,6)})
        print("St Cloud:", "HALLEN's catwalk" if hallen else "far-bank reversal without HALLEN",
              "from public door via guide, WYREL, controls, court and back; walking inputs by leg:", c.lengths)


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("pack", type=Path)
    p.add_argument("--baseline", action="store_true")
    args = p.parse_args()
    trace(load_pack(args.pack)["data"], args.baseline)
