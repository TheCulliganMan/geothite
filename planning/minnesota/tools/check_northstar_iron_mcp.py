#!/usr/bin/env python3
"""Replay earned mine, depot and harbor saves through production stdio inputs."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_river_story_mcp import same_map_travel
from check_winona_mcp import choose, finish, travel, walk
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]


def dialogue(client):
    return " ".join(" ".join(line["text"] for line in client.observe()["structuredContent"]["dialogue"]).split())


def cache(client, pack):
    assert client.status()["map"] == "MnR09Cave1F"
    walk(client, pack, (8, 5))
    client.press("left")
    client.press("a")
    choose(client, False)
    assert "Leave the meter" in finish(client)
    client.press("left")
    client.press("a")
    choose(client, True)
    assert "You take the meter" in finish(client)
    client.press("left")
    client.press("a")
    assert "crate is empty" in finish(client)


def depot(client, pack):
    assert client.status()["map"] == "MnGrandRapidsSurveyDepot"
    walk(client, pack, (11, 10))
    client.press("up")
    client.press("a")
    assert "Use the east road." in finish(client)
    walk(client, pack, (12, 18))
    travel(client, "down", "MnGrandRapids", (27, 14))
    walk(client, pack, (43, 24))
    travel(client, "right", "MnR10C00R00", (0, 10))
    travel(client, "left", "MnGrandRapids", (47, 24))
    same_map_travel(client, "left", "MnGrandRapids", (44, 24))


def custody_loss(client, pack):
    client.at("MnBemidjiPokecenter1F", 10, 13)
    walk(client, pack, (10, 14))
    travel(client, "down", "MnBemidji", (31, 10))
    walk(client, pack, (43, 25))
    travel(client, "right", "MnR09C00R01", (0, 3))


def reveal(client, pack):
    client.at("MnDuluthHarborWorkshop", 10, 5)
    assert client.status()["badges"] == 8
    client.press("right")
    client.press("a")
    story = json.loads((ROOT / "planning/minnesota/story.json").read_text())
    milestone = next(m for m in story["milestones"] if m["id"] == "MS10")
    lines = milestone["native_texts"]["MN_HARBOR_RECEIVERText"]
    assert len(lines) == 20
    for i in range(0, len(lines), 2):
        expected = " ".join(lines[i:i + 2])
        assert dialogue(client) == expected, (i // 2, dialogue(client), expected)
        client.press("down")
        client.at("MnDuluthHarborWorkshop", 10, 5)
        assert dialogue(client) == expected
        client.press("a")
    finish(client)
    client.press("right")
    client.press("a")
    assert "The harbor has the drainage plan." in finish(client)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-itasca-story.crystalpack")
    p.add_argument("--save", type=Path, required=True)
    p.add_argument("--mode", choices=["cache", "depot", "reveal", "custody-loss"], required=True)
    p.add_argument("--client", type=Path, default=ROOT / "target/northstar-story/wasm-dev/geothite")
    p.add_argument("--boosted-review", action="store_true", help="Label evidence from the user-authorized level review.")
    args = p.parse_args()
    pack = load_pack(args.pack)
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen([str(args.client), "mcp", str(args.pack), "--load", str(args.save)],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                                   text=True, bufsize=1)
        client = Client(process)
        try:
            client.rpc("initialize", dict(protocolVersion="2025-06-18", capabilities={},
                                          clientInfo=dict(name="northstar-earned-check", version="1")))
            initial = client.status()
            {"cache": cache, "depot": depot, "reveal": reveal, "custody-loss": custody_loss}[args.mode](client, pack)
            client.press("start")
            assert "PACK" in client.observe()["content"][0]["text"]
            final = client.status()
            assert final["badges"] == initial["badges"] and final["money"] == initial["money"]
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr must remain byte-empty"
    scope = "boosted story review" if args.boosted_review else "earned"
    print(f"PASS: actual stdio {scope} {args.mode}, native inputs/Start, unchanged money/badges and byte-empty stderr.")


if __name__ == "__main__":
    main()
