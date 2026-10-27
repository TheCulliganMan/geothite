#!/usr/bin/env python3
"""Actual stdio lake-country journeys from explicitly local controller saves."""
import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_river_story_mcp import road, same_map_travel
from check_winona_mcp import finish, travel, walk
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]


def route_five(client, pack):
    client.at("MnMinneapolis", 32, 4)
    assert client.status()["badges"] == 4
    travel(client, "up", "MnR05C00R02", (22, 15))
    road(client, pack, (24, 0), "MnR05", 3)
    travel(client, "up", "MnStCloud", (24, 35))
    same_map_travel(client, "up", "MnStCloud", (24, 34))
    walk(client, pack, (24, 4))
    client.move("up")
    assert "WIND BADGE" in finish(client)
    client.at("MnStCloud", 24, 4)
    walk(client, pack, (24, 34))
    travel(client, "down", "MnR05C00R00", (24, 0))
    road(client, pack, (22, 111), "MnR05", 3)
    travel(client, "down", "MnMinneapolis", (32, 0))
    same_map_travel(client, "down", "MnMinneapolis", (32, 4))


def route_six(client, pack):
    client.at("MnStCloud", 24, 4)
    assert client.status()["badges"] == 5
    travel(client, "up", "MnR06C00R01", (16, 39))
    road(client, pack, (42, 0), "MnR06", 2)
    travel(client, "up", "MnBrainerd", (32, 47))
    same_map_travel(client, "up", "MnBrainerd", (32, 46))
    walk(client, pack, (25, 20))
    client.press("up")
    client.press("a")
    before = finish(client)
    assert "SIVREN" in before, before
    walk(client, pack, (32, 46))
    travel(client, "down", "MnR06C00R00", (42, 0))
    road(client, pack, (16, 87), "MnR06", 2)
    travel(client, "down", "MnStCloud", (24, 0))
    same_map_travel(client, "down", "MnStCloud", (24, 4))


def ranger(client, pack):
    client.at("MnBrainerd", 12, 36)
    assert client.status()["badges"] == 6
    walk(client, pack, (25, 20))
    client.press("up")
    client.press("a")
    for expected in ["The fish moved on.", "Ask the fishers."]:
        page = " ".join(line["text"] for line in client.observe()["structuredContent"]["dialogue"])
        assert expected in page, page
        client.press("down")
        client.at("MnBrainerd", 25, 20)
        client.press("a")
    assert "Explore the lakes." in finish(client)
    client.press("up")
    client.press("a")
    repeat = finish(client)
    assert "belongs on record." in repeat, repeat


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-lake-country.crystalpack")
    parser.add_argument("--save", type=Path, required=True)
    parser.add_argument("--mode", choices=["r05", "r06", "ranger"], required=True)
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
                                          clientInfo=dict(name="lake-country-local-check", version="1")))
            initial = client.status()
            {"r05": route_five, "r06": route_six, "ranger": ranger}[args.mode](client, pack)
            client.press("start")
            assert "PACK" in client.observe()["content"][0]["text"]
            final = client.status()
            assert final["badges"] == initial["badges"] and final["money"] == initial["money"]
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr was not byte-empty"
    print(f"PASS: actual stdio {args.mode}/Start from local setup; unchanged money/badges and byte-empty stderr.")


if __name__ == "__main__":
    main()
