#!/usr/bin/env python3
"""Actual stdio shore/road travel and locally earned Pine loss/victory Continue."""
import argparse
from copy import deepcopy
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_river_story_mcp import road, same_map_travel
from check_winona_mcp import finish, travel, walk
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]
SHORE = "MnLeechLake"


def cabins(client, pack):
    for room, door, arrival, carpets in [
        ("MnLeechLakeCareCabin", (11, 14), (10, 13), [(10, 15), (11, 15)]),
        ("MnLeechLakeFisherHouse", (9, 36), (5, 6), [(5, 7), (6, 7)]),
    ]:
        for carpet in carpets:
            walk(client, pack, door)
            travel(client, "up", room, arrival)
            if room.endswith("CareCabin"):
                walk(client, pack, (7, 5))
                client.press("up")
                client.press("a")
                assert "No forms needed." in finish(client)
            else:
                walk(client, pack, (6, 4))
                client.press("up")
                client.press("a")
                assert "ICE asked me" in finish(client)
            walk(client, pack, (carpet[0], carpet[1] - 1))
            travel(client, "down", SHORE, door)
            client.move("down")
            client.at(SHORE, door[0], door[1] + 1)


def roads(client, pack):
    client.at("MnBrainerd", 32, 4)
    assert client.status()["badges"] == 6
    travel(client, "up", "MnR07C00R01", (20, 47))
    road(client, pack, (20, 0), "MnR07", 2)
    travel(client, "up", SHORE, (32, 47))
    same_map_travel(client, "up", SHORE, (32, 44))
    cabins(client, pack)
    walk(client, pack, (35, 22))
    client.press("up")
    client.press("a")
    assert "empty this week." in finish(client)
    walk(client, pack, (40, 22))
    client.move("right", 3)
    client.at(SHORE, 40, 22)
    walk(client, pack, (32, 4))
    travel(client, "up", "MnR08C00R01", (26, 47))
    road(client, pack, (40, 0), "MnR08", 2, 2)
    travel(client, "up", "MnBemidji", (24, 47))
    same_map_travel(client, "up", "MnBemidji", (24, 46))
    walk(client, pack, (31, 10))
    travel(client, "up", "MnBemidjiPokecenter1F", (10, 13))
    walk(client, pack, (10, 14))
    travel(client, "down", "MnBemidji", (31, 10))
    walk(client, pack, (24, 46))
    travel(client, "down", "MnR08C00R00", (40, 0))
    road(client, pack, (26, 95), "MnR08", 2, 2)
    travel(client, "down", SHORE, (32, 0))
    same_map_travel(client, "down", SHORE, (32, 4))
    walk(client, pack, (32, 44))
    travel(client, "down", "MnR07C00R00", (20, 0))
    road(client, pack, (20, 95), "MnR07", 2)
    travel(client, "down", "MnBrainerd", (32, 0))
    same_map_travel(client, "down", "MnBrainerd", (32, 4))


def pine(client, pack, loss):
    state = client.status()
    assert state["badges"] == (6 if loss else 7)
    if loss:
        client.at("MnBemidjiPokecenter1F", 10, 13)
    else:
        assert state["map"] == "MnBemidjiGym"
        # This save earned both native Cut flags and Pine in the controller.
        # Adjust only the path planner's copy for those already pruned cells.
        # The live pack/save are never modified; real movement must reach the
        # exit or the check fails on the authoritative client position.
        planned = deepcopy(pack)
        module = planned["data"]["maps"]["MnBemidjiGym"]
        changes = [c for c in module["script_block_changes"] if c["source_script"].endswith("PrunedUnchanged")]
        assert len(changes) == 2
        for change in changes:
            # Preserve the replacement block's native wall quadrants too.
            x, y = change["x"], change["y"]
            module["blocks"][y // 2 * module["attributes"]["width"] + x // 2] = change["block_id"]
        walk(client, planned, (4, 18))
        travel(client, "down", "MnBemidji", (10, 10))
        walk(client, pack, (31, 10))
        travel(client, "up", "MnBemidjiPokecenter1F", (10, 13))
    walk(client, pack, (7, 5))
    client.press("up")
    client.press("a")
    finish(client)
    walk(client, pack, (10, 14))
    travel(client, "down", "MnBemidji", (31, 10))
    walk(client, pack, (21, 20))
    client.press("up")
    client.press("a")
    assert "Compare the logs." in finish(client)
    walk(client, pack, (10, 10))
    travel(client, "up", "MnBemidjiGym", (4, 17))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-leech-shore.crystalpack")
    parser.add_argument("--save", type=Path, required=True)
    parser.add_argument("--mode", choices=["roads", "pine", "loss"], required=True)
    parser.add_argument("--client", type=Path, default=ROOT / "target/northstar-story/wasm-dev/geothite")
    args = parser.parse_args()
    pack = load_pack(args.pack)
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen([str(args.client), "mcp", str(args.pack), "--load", str(args.save)],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                                   text=True, bufsize=1)
        client = Client(process)
        try:
            client.rpc("initialize", dict(protocolVersion="2025-06-18", capabilities={},
                                          clientInfo=dict(name="leech-shore-local-check", version="1")))
            initial = client.status()
            if args.mode == "roads":
                roads(client, pack)
            else:
                pine(client, pack, args.mode == "loss")
            client.press("start")
            assert "PACK" in client.observe()["content"][0]["text"]
            final = client.status()
            assert final["money"] == initial["money"] and final["badges"] == initial["badges"]
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr was not byte-empty"
    print(f"PASS: actual stdio {args.mode}/Start from local setup; unchanged money/badges and byte-empty stderr.")


if __name__ == "__main__":
    main()
