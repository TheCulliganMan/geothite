#!/usr/bin/env python3
"""Play delivery and city doors over stdio from an earned Rochester arrival."""
import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_winona_mcp import finish, travel, walk
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]


def play_badge(client, pack):
    """Continue an actually earned badge; visit the leader again and recover."""
    client.rpc("initialize", dict(protocolVersion="2025-06-18", capabilities={},
                                  clientInfo=dict(name="rochester-badge-check", version="1")))
    client.at("MnRochester", 30, 24)
    initial = client.status()
    assert initial["badges"] == 1 and initial["pokedex"]["owned"] >= 2
    travel(client, "up", "MnRochesterGym", (6, 17))
    walk(client, pack, (6, 4))
    client.press("up")
    client.press("a")
    pages = finish(client)
    assert "MANKATO" in pages, pages
    assert client.status()["mode"] == "overworld"
    walk(client, pack, (6, 18))
    travel(client, "down", "MnRochester", (30, 24))
    walk(client, pack, (15, 8))
    travel(client, "up", "MnRochesterPokecenter1F", (10, 13))
    walk(client, pack, (7, 5))
    client.press("up")
    client.press("a")
    assert finish(client), "ordinary care has visible dialogue"
    walk(client, pack, (10, 14))
    travel(client, "down", "MnRochester", (15, 8))
    client.press("start")
    assert "PACK" in client.observe()["content"][0]["text"]
    final = client.status()
    assert final["badges"] == 1 and final["money"] == initial["money"]


def play(client, pack):
    client.rpc("initialize", dict(protocolVersion="2025-06-18", capabilities={},
                                  clientInfo=dict(name="rochester-check", version="1")))
    client.at("MnRochester", 39, 18)
    initial = client.status()
    assert initial["badges"] == 0
    walk(client, pack, (30, 24))
    travel(client, "up", "MnRochesterGym", (6, 17))
    walk(client, pack, (6, 4))
    client.press("up")
    client.press("a")
    refusal = finish(client)
    assert "parcel" in refusal.lower() or "clinic" in refusal.lower(), refusal
    walk(client, pack, (6, 18))
    travel(client, "down", "MnRochester", (30, 24))
    walk(client, pack, (15, 8))
    travel(client, "up", "MnRochesterPokecenter1F", (10, 13))
    walk(client, pack, (11, 9))
    client.press("up")
    client.press("a")
    first = client.observe()["structuredContent"]["dialogue"]
    assert "meters" in " ".join(line["text"] for line in first)
    client.press("down")
    client.at("MnRochesterPokecenter1F", 11, 9)
    client.press("a")
    second = client.observe()["structuredContent"]["dialogue"]
    assert "LODE" in " ".join(line["text"] for line in second)
    finish(client)
    client.press("up")
    client.press("a")
    assert "We kept your copy" in finish(client)
    # Both real clinic carpets and both real gym carpets return to their city.
    for x in [10, 11]:
        walk(client, pack, (x, 14))
        travel(client, "down", "MnRochester", (15, 8))
        if x == 10:
            travel(client, "up", "MnRochesterPokecenter1F", (10, 13))
    for x in [6, 7]:
        walk(client, pack, (30, 24))
        travel(client, "up", "MnRochesterGym", (6, 17))
        walk(client, pack, (x, 18))
        travel(client, "down", "MnRochester", (30, 24))
    for outside, name in [((7, 8), "MnRochesterClerkHouse"),
                          ((7, 24), "MnRochesterPatientHouse")]:
        walk(client, pack, outside)
        travel(client, "up", name, (5, 6))
        walk(client, pack, (5, 3))
        client.press("right")
        client.press("a")
        assert finish(client), "resident must have native dialogue"
        walk(client, pack, (5, 6))
        travel(client, "down", "MnRochester", outside)
    walk(client, pack, (29, 10))
    travel(client, "up", "MnRochesterMart", (6, 5))
    walk(client, pack, (6, 6))
    travel(client, "down", "MnRochester", (29, 10))
    client.press("start")
    assert "PACK" in client.observe()["content"][0]["text"]
    final = client.status()
    assert final["badges"] == 0 and final["money"] == initial["money"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-rochester.crystalpack")
    parser.add_argument("--save", type=Path, required=True, help="earned arrival at Rochester (39,18), before delivery")
    parser.add_argument("--earned-badge", action="store_true", help="use the earned first-badge save at (30,24)")
    parser.add_argument("--client", type=Path, default=ROOT / "target/northstar-story/wasm-dev/geothite")
    args = parser.parse_args()
    pack = load_pack(args.pack)
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen([str(args.client), "mcp", str(args.pack), "--load", str(args.save)],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                                   text=True, bufsize=1)
        try:
            (play_badge if args.earned_badge else play)(Client(process), pack)
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0, "client exited unsuccessfully"
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr was not byte-empty"
    if args.earned_badge:
        print("PASS: earned Care Badge Continue, leader repeat, free clinic return and Start; unchanged badge/money and byte-empty stderr.")
    else:
        print("PASS: actual stdio delivery prerequisite, two pages/input ownership, repeat, all five facades, paired clinic/gym exits and Start; byte-empty stderr.")


if __name__ == "__main__":
    main()
