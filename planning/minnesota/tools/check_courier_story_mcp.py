#!/usr/bin/env python3
"""Real stdio Current-only road trip or a controller-earned courier-win Continue."""
import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_prairie_story_mcp import road
from check_river_story_mcp import same_map_travel
from check_winona_mcp import finish, travel, walk
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]
DEPOT = "MnStPaulUnionDepot"


def road_trip(client, pack):
    client.at("MnMinneapolis", 59, 24)
    initial = client.status()
    assert initial["badges"] == 3
    travel(client, "right", "MnR04C00R00", (0, 18))
    road(client, pack, (55, 18), "MnR04C00R00")
    travel(client, "right", "MnStPaul", (0, 24))
    same_map_travel(client, "right", "MnStPaul", (1, 24))
    walk(client, pack, (44, 38))
    travel(client, "up", DEPOT, (15, 21))
    walk(client, pack, (19, 16))
    client.press("right")
    client.press("a")
    assert "No dispatch copy?" in finish(client)
    client.press("left")
    client.press("a")
    assert "courier holds" in finish(client)
    walk(client, pack, (15, 21))
    travel(client, "down", "MnStPaul", (44, 38))
    walk(client, pack, (1, 24))
    travel(client, "left", "MnR04C00R00", (55, 18))
    road(client, pack, (0, 18), "MnR04C00R00")
    travel(client, "left", "MnMinneapolis", (63, 24))
    same_map_travel(client, "left", "MnMinneapolis", (59, 24))
    assert client.status()["money"] == initial["money"]


def manifest(client, pack):
    client.at(DEPOT, 19, 16)
    initial = client.status()
    assert initial["badges"] == 3
    client.press("right")
    client.press("a")
    assert "LODE will go on" in finish(client)
    client.press("left")
    client.press("a")
    for expected in ["DEEP BORE", "Diversion logged", "Record copied"]:
        page = " ".join(line["text"] for line in client.observe()["structuredContent"]["dialogue"])
        assert expected in page, page
        client.press("down")
        client.at(DEPOT, 19, 16)
        client.press("a")
    finish(client)
    client.press("left")
    client.press("a")
    assert "instrument record" in finish(client)
    walk(client, pack, (15, 21))
    travel(client, "down", "MnStPaul", (44, 38))
    assert client.status()["money"] == initial["money"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-courier-story.crystalpack")
    parser.add_argument("--save", type=Path, required=True)
    source = parser.add_mutually_exclusive_group()
    source.add_argument("--earned-local-win", action="store_true",
                        help="controller-earned courier win from a local prior-badge/strong-party setup; not a fresh-start journey")
    source.add_argument("--earned-campaign-win", action="store_true",
                        help="actual courier victory checkpoint from the earned Winona campaign; replay its separate manifest")
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
                                          clientInfo=dict(name="courier-local-check", version="1")))
            (manifest if args.earned_local_win or args.earned_campaign_win else road_trip)(client, pack)
            client.press("start")
            assert "PACK" in client.observe()["content"][0]["text"]
            assert client.status()["badges"] == 3
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr was not byte-empty"
    if args.earned_campaign_win:
        scope = "earned campaign victory Continue/three manifest pages/repeat/city return"
    elif args.earned_local_win:
        scope = "local courier-win Continue/three manifest pages/repeat/city return"
    else:
        scope = "empty-party local road round trip/depot prerequisites"
    print(f"PASS: actual stdio {scope}/Start; unchanged money/badges and byte-empty stderr.")


if __name__ == "__main__":
    main()
