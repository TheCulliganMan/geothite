#!/usr/bin/env python3
"""Exercise R02 and the notebook over real stdio from explicit local setup."""
import argparse
from collections import deque
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_winona_mcp import DIRECTIONS, finish, travel, walk
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]


def road(client, pack, target, name="MnR02C00R00"):
    """Plan a collision-valid path outside trainer sight; public input executes it."""
    state = client.status()
    assert state["map"] == name
    start = (state["position"]["x"], state["position"]["y"])
    module = pack["data"]["maps"][name]
    width, height = module["attributes"]["width"], module["attributes"]["height"]
    collision = pack["data"]["tilesets"][module["attributes"]["tileset_name"]]["collision"]

    def ground(at):
        x, y = at
        if not (0 <= x < width * 2 and 0 <= y < height * 2):
            return False
        block = module["blocks"][y // 2 * width + x // 2]
        return collision[f"{block:02x}"][y % 2 * 2 + x % 2] in {"FLOOR", "TALL_GRASS", "LADDER"}

    blocked = {(o["x"], o["y"]) for o in module["objects"]}
    for actor in module["objects"]:
        if actor["object_type"] != "OBJECTTYPE_TRAINER":
            continue
        dx, dy = {"SPRITEMOVEDATA_STANDING_UP": (0, -1),
                  "SPRITEMOVEDATA_STANDING_LEFT": (-1, 0),
                  "SPRITEMOVEDATA_STANDING_RIGHT": (1, 0)}.get(actor["spritemovedata"], (0, 1))
        for step in range(1, actor["radius"] + 1):
            at = (actor["x"] + dx * step, actor["y"] + dy * step)
            if not ground(at):
                break
            blocked.add(at)
    queue, paths = deque([start]), {start: []}
    while queue and target not in paths:
        x, y = queue.popleft()
        for dx, dy, button in DIRECTIONS:
            at = (x + dx, y + dy)
            if at in paths or at in blocked or not ground(at):
                continue
            paths[at] = paths[(x, y)] + [button]
            queue.append(at)
    assert target in paths, (start, target)
    for button in paths[target]:
        client.move(button)
        assert client.status()["mode"] == "overworld", "local empty-party fixture must not enter battle"
    client.at(name, *target)


def play(client, pack):
    client.rpc("initialize", dict(protocolVersion="2025-06-18", capabilities={},
                                  clientInfo=dict(name="prairie-local-check", version="1")))
    client.at("MnRochester", 4, 18)
    initial = client.status()
    assert initial["badges"] == 1
    travel(client, "left", "MnR02C00R00", (59, 14))
    road(client, pack, (0, 30))
    travel(client, "left", "MnMankato", (47, 18))
    client.move("left")
    walk(client, pack, (26, 26))
    client.press("up")
    client.press("a")
    page = " ".join(line["text"] for line in client.observe()["structuredContent"]["dialogue"])
    assert "Different lakes" in page and "same numbers" in page, page
    client.press("down")
    client.at("MnMankato", 26, 26)
    client.press("a")
    page = " ".join(line["text"] for line in client.observe()["structuredContent"]["dialogue"])
    assert "Who copied" in page, page
    finish(client)
    client.press("up")
    client.press("a")
    assert "same numbers" in finish(client)
    walk(client, pack, (46, 18))
    travel(client, "right", "MnR02C00R00", (0, 30))
    road(client, pack, (59, 14))
    travel(client, "right", "MnRochester", (0, 18))
    # Same-map motion must execute input, rather than a warp-only assertion.
    for _ in range(12):
        if client.status()["position"] == dict(x=4, y=18):
            break
        client.press("right")
    client.at("MnRochester", 4, 18)
    client.press("start")
    assert "PACK" in client.observe()["content"][0]["text"]
    final = client.status()
    assert final["badges"] == initial["badges"] and final["money"] == initial["money"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-prairie-story.crystalpack")
    parser.add_argument("--save", type=Path, required=True,
                        help="prairie-local-road-ready controller save: empty party and granted Care; not earned progression")
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
            assert process.wait(timeout=30) == 0, "client exited unsuccessfully"
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr was not byte-empty"
    print("PASS: local setup only; actual stdio Care crossing, R02 both ways, two notebook pages/input ownership/repeat and Start; unchanged badges/money and byte-empty stderr.")


if __name__ == "__main__":
    main()
