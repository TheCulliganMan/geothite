#!/usr/bin/env python3
"""Replay the separately earned chief/receiver finale through production stdio."""
import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_north_shore_story_mcp import choose, finish
from check_winona_mcp import travel, walk
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]
ROOM = "MnBoundaryWatersSurveyShelter"


def play(client, pack, mode):
    if mode == "chief-win":
        client.at(ROOM, 8, 5)
        client.press("right")
        client.press("a")
        assert "have not changed my mind" in finish(client)
        walk(client, pack, (12, 4))
        client.press("up")
        client.press("a")
        choose(client, False)
        assert "another time" in finish(client)
        client.press("up")
        client.press("a")
        choose(client, True)
        client.press("down")
        client.at(ROOM, 12, 4)
        text = finish(client)
        assert all(phrase in text for phrase in ["align the dial", "Crews stop the drainage pumps", "KETTLURK escapes", "ELY trail"]), text
    else:
        client.at(ROOM, 12, 4)
    client.press("up")
    client.press("a")
    assert "Drainage stopped" in finish(client)
    walk(client, pack, (8, 10))
    travel(client, "down", "MnBoundaryWaters", (43, 20))
    walk(client, pack, (32, 41))
    travel(client, "down", "MnR13C00R00", (30, 0))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-north-shore-story.crystalpack")
    parser.add_argument("--client", type=Path, default=ROOT / "target/northstar-story/wasm-dev/geothite")
    parser.add_argument("--mode", choices=["chief-win", "restored"], required=True)
    parser.add_argument("--save", type=Path, required=True)
    parser.add_argument("--boosted-review", action="store_true", help="Label the user-authorized supplied-level continuation.")
    args = parser.parse_args()
    pack = load_pack(args.pack)
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen([str(args.client), "mcp", str(args.pack), "--load", str(args.save)],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                                   text=True, bufsize=1)
        client = Client(process)
        try:
            client.rpc("initialize", dict(protocolVersion="2025-06-18", capabilities={},
                                          clientInfo=dict(name="northstar-finale-check", version="1")))
            before = client.status()
            play(client, pack, args.mode)
            client.press("start")
            assert "PACK" in client.observe()["content"][0]["text"]
            after = client.status()
            assert (after["money"], after["badges"]) == (before["money"], before["badges"])
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr must remain byte-empty"
    scope = "boosted story review" if args.boosted_review else "earned"
    print(f"PASS: {scope} {args.mode}, separate receiver, League release, Start, unchanged money/badges and byte-empty stderr.")


if __name__ == "__main__":
    main()
