#!/usr/bin/env python3
"""Real stdio river-city round trip; explicit empty-party/two-badge setup."""
import argparse
from collections import deque
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_winona_mcp import DIRECTIONS, travel, walk
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]


def road(client, pack, target, prefix="MnR03", rows=2, columns=1, encounter=None):
    floor, blocked, trainers = set(), set(), []
    for row in range(rows):
        for column in range(columns):
            module = pack["data"]["maps"][f"{prefix}C{column:02}R{row:02}"]
            attr = module["attributes"]
            collision = pack["data"]["tilesets"][attr["tileset_name"]]["collision"]
            for y in range(attr["height"] * 2):
                for x in range(attr["width"] * 2):
                    block = module["blocks"][y // 2 * attr["width"] + x // 2]
                    if collision[f"{block:02x}"][y % 2 * 2 + x % 2] in {"FLOOR", "TALL_GRASS", "LADDER"}:
                        floor.add((x + column * 64, y + row * 48))
            for actor in module["objects"]:
                point = (actor["x"] + column * 64, actor["y"] + row * 48)
                blocked.add(point)
                if actor["object_type"] == "OBJECTTYPE_TRAINER":
                    trainers.append((point, actor))
    for (x, y), actor in trainers:
        dx, dy = {"SPRITEMOVEDATA_STANDING_UP": (0, -1),
                  "SPRITEMOVEDATA_STANDING_LEFT": (-1, 0),
                  "SPRITEMOVEDATA_STANDING_RIGHT": (1, 0)}.get(actor["spritemovedata"], (0, 1))
        for step in range(1, actor["radius"] + 1):
            at = (x + step * dx, y + step * dy)
            if at not in floor:
                break
            blocked.add(at)
    def position(state):
        assert state["map"].startswith(prefix + "C"), state
        column, row = state["map"][len(prefix) + 1:].split("R")
        return state["position"]["x"] + int(column) * 64, state["position"]["y"] + int(row) * 48
    start = position(client.status())
    queue, paths = deque([start]), {start: []}
    while queue and target not in paths:
        x, y = queue.popleft()
        for dx, dy, button in DIRECTIONS:
            at = (x + dx, y + dy)
            if at not in floor or at in blocked or at in paths:
                continue
            paths[at] = paths[(x, y)] + [(button, at)]
            queue.append(at)
    assert target in paths, (start, target)
    for button, expected in paths[target]:
        before = position(client.status())
        # A wild encounter can own input before the pending tile finishes.
        # Like the controller road course, issue the next native step only if
        # fleeing leaves the player on the exact previous tile.
        for _ in range(8):
            client.move(button)
            state = client.status()
            if state["mode"] != "overworld" and encounter is not None:
                encounter(client)
                state = client.status()
            assert state["mode"] == "overworld", (button, expected, state)
            current = position(state)
            if current == expected:
                break
            assert current == before, ("unexpected route displacement", button, before, expected, state)
        assert position(state) == expected, ("native step did not finish", button, before, expected, state)
    client.at(f"{prefix}C{target[0] // 64:02}R{target[1] // 48:02}", target[0] % 64, target[1] % 48)


def same_map_travel(client, button, name, target):
    for _ in range(12):
        state = client.status()
        if state["map"] == name and state["position"] == dict(x=target[0], y=target[1]):
            return
        client.press(button)
    raise AssertionError((name, target, client.status()))


def play(client, pack):
    client.rpc("initialize", dict(protocolVersion="2025-06-18", capabilities={},
                                  clientInfo=dict(name="river-local-check", version="1")))
    client.at("MnMankato", 24, 4)
    initial = client.status()
    assert initial["badges"] == 2
    travel(client, "up", "MnR03C00R01", (18, 15))
    road(client, pack, (20, 0))
    travel(client, "up", "MnMinneapolis", (32, 47))
    same_map_travel(client, "up", "MnMinneapolis", (32, 46))
    walk(client, pack, (27, 34))
    travel(client, "up", "MnMinneapolisDispatchOffice", (10, 14))
    # The existing city chapter is reachable without granting any ICE flag.
    walk(client, pack, (10, 11))
    client.press("left")
    client.press("a")
    pages = client.finish_dialogue()
    assert "You need the clerk to read this copy." in " ".join(pages.split()), pages
    walk(client, pack, (10, 14))
    travel(client, "down", "MnMinneapolis", (27, 34))
    walk(client, pack, (32, 46))
    travel(client, "down", "MnR03C00R00", (20, 0))
    road(client, pack, (18, 63))
    travel(client, "down", "MnMankato", (24, 0))
    same_map_travel(client, "down", "MnMankato", (24, 4))
    client.press("start")
    assert "PACK" in client.observe()["content"][0]["text"]
    final = client.status()
    assert final["money"] == initial["money"] and final["badges"] == initial["badges"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-river-story.crystalpack")
    parser.add_argument("--save", type=Path, required=True,
                        help="river-local-road-ready fixture at Mankato (24,4); not earned progression")
    parser.add_argument("--client", type=Path, default=ROOT / "target/northstar-story/wasm-dev/geothite")
    args = parser.parse_args()
    pack = load_pack(args.pack)
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen([str(args.client), "mcp", str(args.pack), "--load", str(args.save)],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                                   text=True, bufsize=1)
        try:
            play(Client(process), pack)
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr was not byte-empty"
    print("PASS: local setup; actual stdio Mankato/R03/Minneapolis/office and return, locked record and Start; unchanged money/badges and byte-empty stderr.")


if __name__ == "__main__":
    main()
