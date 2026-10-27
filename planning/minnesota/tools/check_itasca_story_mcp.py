#!/usr/bin/env python3
"""Actual stdio headwaters outing, acknowledged comparison and Iron Range road."""
import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_river_story_mcp import road, same_map_travel
from check_winona_mcp import finish, travel, walk
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]
PARK = "MnLakeItasca"
GATE = "MnItascaGatehouse"


def dialogue(client):
    return " ".join(line["text"] for line in client.observe()["structuredContent"]["dialogue"])


def officer(client, pack):
    walk(client, pack, (21, 20))
    client.press("up")
    client.press("a")


def outing(client, pack):
    walk(client, pack, (4, 24))
    travel(client, "left", "MnR14C01R00", (31, 26))
    road(client, pack, (5, 42), "MnR14", 2, 2)
    travel(client, "left", GATE, (8, 4))
    walk(client, pack, (5, 4))
    client.press("up")
    client.press("a")
    assert "LAKE ITASCA" in finish(client)
    walk(client, pack, (1, 4))
    travel(client, "left", PARK, (42, 24))
    walk(client, pack, (19, 22))
    client.press("up")
    client.press("a")


def return_trip(client, pack):
    walk(client, pack, (42, 25))
    travel(client, "right", GATE, (1, 5))
    walk(client, pack, (8, 5))
    travel(client, "right", "MnR14C00R00", (5, 43))
    road(client, pack, (95, 27), "MnR14", 2, 2)
    travel(client, "right", "MnBemidji", (0, 25))
    same_map_travel(client, "right", "MnBemidji", (4, 25))


def iron_denial(client, pack, word):
    walk(client, pack, (43, 24))
    for _ in range(8):
        if client.status()["dialogue"]:
            break
        client.press("right")
    assert word in dialogue(client)
    client.press("right")
    client.at("MnBemidji", 44, 24)
    finish(client)
    client.at("MnBemidji", 43, 24)


def compare(client, pack):
    officer(client, pack)
    assert "One meter log." in dialogue(client)
    client.press("left")
    client.at("MnBemidji", 21, 20)
    client.press("a")
    assert "These were copied." in dialogue(client)
    client.press("left")
    client.at("MnBemidji", 21, 20)
    finish(client)
    officer(client, pack)
    assert "IRON RANGE" in finish(client)


def iron(client, pack, full):
    walk(client, pack, (43, 24))
    travel(client, "right", "MnR09C00R01", (0, 2))
    if full:
        road(client, pack, (127, 24), "MnR09", 2, 2)
        travel(client, "right", "MnGrandRapids", (0, 24))
        travel(client, "left", "MnR09C01R00", (63, 24))
        road(client, pack, (0, 50), "MnR09", 2, 2)
    travel(client, "left", "MnBemidji", (47, 24))
    same_map_travel(client, "left", "MnBemidji", (43, 24))


def play(client, pack, mode):
    if mode == "earned-report":
        client.at("MnBemidji", 21, 20)
        assert client.status()["badges"] == 7
        officer(client, pack)
        assert "IRON RANGE" in finish(client)
        iron(client, pack, False)
        return
    client.at("MnBemidji", *( (4, 24) if mode == "early" else (21, 20) ))
    assert client.status()["badges"] == (6 if mode == "early" else 7)
    outing(client, pack)
    assert ("PINE" if mode == "early" else "HEADWATERS MARKER") in dialogue(client)
    client.press("left")
    client.at(PARK, 19, 22)
    finish(client)
    return_trip(client, pack)
    if mode == "early":
        officer(client, pack)
        assert "PINE" in finish(client)
        iron_denial(client, pack, "PINE")
    else:
        iron_denial(client, pack, "marker")
        compare(client, pack)
        iron(client, pack, True)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-itasca-story.crystalpack")
    p.add_argument("--save", type=Path, required=True)
    p.add_argument("--mode", choices=["early", "chapter", "earned-report"], required=True)
    p.add_argument("--client", type=Path, default=ROOT / "target/northstar-story/wasm-dev/geothite")
    args = p.parse_args()
    pack = load_pack(args.pack)
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen([str(args.client), "mcp", str(args.pack), "--load", str(args.save)],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                                   text=True, bufsize=1)
        client = Client(process)
        try:
            client.rpc("initialize", dict(protocolVersion="2025-06-18", capabilities={},
                                          clientInfo=dict(name="itasca-local-check", version="1")))
            initial = client.status()
            play(client, pack, args.mode)
            client.press("start")
            assert "PACK" in client.observe()["content"][0]["text"]
            final = client.status()
            assert final["money"] == initial["money"] and final["badges"] == initial["badges"]
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr must remain byte-empty"
    print(f"PASS: actual stdio {args.mode}, Start, unchanged money/badges and byte-empty stderr.")


if __name__ == "__main__":
    main()
