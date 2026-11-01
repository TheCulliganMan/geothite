#!/usr/bin/env python3
"""Replay the native relay picture from a played chief victory or picture save."""
import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_north_shore_story_mcp import choose, finish
from check_winona_mcp import walk
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]
ROOM = "MnBoundaryWatersSurveyShelter"


def picture(client):
    return any(line["text"] == "Picture: KETTLURK" for line in client.observe()["structuredContent"]["prompt"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-relay-v2.crystalpack")
    parser.add_argument("--client", type=Path, default=ROOT / "target/northstar-story/wasm-dev/geothite")
    parser.add_argument("--save", type=Path, required=True)
    parser.add_argument("--mode", choices=["chief-win", "picture"], required=True)
    parser.add_argument("--close", choices=["a", "b"], default="a")
    parser.add_argument("--boosted-review", action="store_true", help="Label the user-authorized supplied-level continuation.")
    args = parser.parse_args()
    pack = load_pack(args.pack)
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen([str(args.client), "mcp", str(args.pack), "--load", str(args.save)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True, bufsize=1)
        client = Client(process)
        try:
            client.rpc("initialize", dict(protocolVersion="2025-06-18", capabilities={}, clientInfo=dict(name="northstar-relay-check", version="1")))
            before = client.status()
            if args.mode == "chief-win":
                walk(client, pack, (12, 4))
                client.press("up")
                client.press("a")
                choose(client, False)
                assert "another time" in finish(client)
                client.press("up")
                client.press("a")
                choose(client, True)
                for _ in range(64):
                    if picture(client):
                        break
                    client.press("a")
            assert picture(client), "actual picture must be observable through the shared renderer"
            client.at(ROOM, 12, 4)
            client.press("down")
            client.at(ROOM, 12, 4)
            assert picture(client), "directional input cannot dismiss the native picture"
            client.press(args.close)
            for _ in range(64):
                view = client.observe()["structuredContent"]
                if any("You cut the link" in line["text"] for line in view["dialogue"]):
                    break
                client.press("down")
                client.at(ROOM, 12, 4)
                client.press("a")
            else:
                raise AssertionError("native picture did not continue to the separate cut-link pages")
            text = finish(client)
            assert "The motors coast" in text and "KETTLURK escapes" in text, text
            assert not picture(client)
            client.press("up")
            client.press("a")
            assert "Drainage stopped" in finish(client)
            client.press("start")
            assert "PACK" in client.observe()["content"][0]["text"]
            after = client.status()
            assert (after["money"], after["badges"], after["pokedex"]) == (before["money"], before["badges"], before["pokedex"])
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr must remain byte-empty"
    scope = "boosted story review" if args.boosted_review else "played"
    print(f"PASS: actual stdio {scope} {args.mode}, visible KETTLURK picture/input ownership, separate shutdown, repeat/Start, no money/badge/dex reward and byte-empty stderr.")


if __name__ == "__main__":
    main()
