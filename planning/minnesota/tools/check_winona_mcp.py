#!/usr/bin/env python3
"""Play Winona from New Game through the production stdio MCP client."""

import argparse
from collections import deque
import json
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]
DIRECTIONS = [(0, -1, "up"), (1, 0, "right"), (0, 1, "down"), (-1, 0, "left")]


def walk(client, pack, target):
    """Plan over packed collision; execute every tile through the public tool."""
    status = client.status()
    name = status["map"]
    start = (status["position"]["x"], status["position"]["y"])
    module = pack["data"]["maps"][name]
    width, height = module["attributes"]["width"], module["attributes"]["height"]
    collision = pack["data"]["tilesets"][module["attributes"]["tileset_name"]]["collision"]
    bodies = {(a["x"], a["y"]) for a in pack["data"]["npcs"].get(name, [])}
    queue, paths = deque([start]), {start: []}
    while queue and target not in paths:
        x, y = queue.popleft()
        for dx, dy, direction in DIRECTIONS:
            at = (x + dx, y + dy)
            if at in paths or at in bodies or not (0 <= at[0] < width * 2 and 0 <= at[1] < height * 2):
                continue
            block = module["blocks"][at[1] // 2 * width + at[0] // 2]
            if collision[f"{block:02x}"][at[1] % 2 * 2 + at[0] % 2] != "FLOOR":
                continue
            paths[at] = paths[(x, y)] + [direction]
            queue.append(at)
    assert target in paths, (name, start, target)
    for direction in paths[target]:
        client.move(direction)
    client.at(name, *target)


def travel(client, direction, name, tile):
    for _ in range(12):
        status = client.status()
        if status["map"] == name:
            client.at(name, *tile)
            return
        client.press(direction)
    raise AssertionError(("warp did not fire", name))


def choose(client, yes):
    target = "> " + ("YES" if yes else "NO")
    for _ in range(16):
        snapshot = client.observe()["structuredContent"]
        lines = [line["text"].strip() for line in snapshot["prompt"] + snapshot["menu"]]
        if target in lines:
            client.press("a")
            return
        client.press("down" if any(line.startswith("> ") for line in lines) else "a")
    raise AssertionError(("choice is not usable", target, lines))


def finish(client):
    pages = []
    for _ in range(64):
        if not client.status()["dialogue"]:
            return " ".join(" ".join(pages).split())
        pages.extend(line["text"] for line in client.observe()["structuredContent"]["dialogue"])
        client.press("a")
    raise AssertionError("dialogue did not release input")


def play(client, pack):
    client.rpc("initialize", dict(protocolVersion="2025-06-18", capabilities={},
                                  clientInfo=dict(name="winona-check", version="1")))
    client.at("MnWinonaHome2F", 3, 3)
    initial = client.status()
    walk(client, pack, (7, 1))
    travel(client, "up", "MnWinonaHome1F", (9, 0))
    client.press("down")
    client.at("MnWinonaHome1F", 9, 0)
    guardian = finish(client)
    assert "TAMARACK" in guardian and "partner" in guardian, guardian
    client.move("down")
    walk(client, pack, (6, 6))
    travel(client, "down", "MnWinona", (7, 8))
    walk(client, pack, (18, 20))
    travel(client, "up", "MnWinonaFieldStation", (4, 10))
    walk(client, pack, (7, 4))
    client.press("up")
    client.press("a")
    choose(client, False)
    declined = finish(client)
    assert "All three can wait" in declined, declined
    client.press("up")
    client.press("a")
    choose(client, True)
    assert "nickname" in client.observe()["content"][0]["text"].lower()
    choose(client, False)
    client.press("down")
    client.at("MnWinonaFieldStation", 7, 4)
    handover = finish(client)
    assert "ROCHESTER" in handover and "Five POKE BALLS" in handover, handover
    for x in [6, 8]:
        walk(client, pack, (x, 4))
        client.press("up")
        client.press("a")
        reminder = finish(client)
        assert "ROCHESTER needs" in reminder, reminder
    walk(client, pack, (4, 10))
    travel(client, "down", "MnWinona", (18, 20))
    client.press("start")
    assert "PACK" in client.observe()["content"][0]["text"]
    final = client.status()
    assert final["badges"] == 0 and final["money"] == initial["money"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-winona.crystalpack")
    parser.add_argument("--client", type=Path, default=ROOT / "target/northstar-story/wasm-dev/geothite")
    args = parser.parse_args()
    pack = load_pack(args.pack)
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen([str(args.client), "mcp", str(args.pack)], stdin=subprocess.PIPE,
                                   stdout=subprocess.PIPE, stderr=stderr, text=True, bufsize=1)
        try:
            play(Client(process), pack)
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0, "client exited unsuccessfully"
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr was not byte-empty"
    print("PASS: New Game, guardian, decline/retry, real starter/nickname choice, survey/balls handover, repeat, return and Start over actual stdio; byte-empty stderr.")


if __name__ == "__main__":
    main()
