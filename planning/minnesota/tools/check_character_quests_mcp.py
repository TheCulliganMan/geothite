#!/usr/bin/env python3
"""Replay witnessed character quests over the production stdio controller."""
import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_north_shore_story_mcp import choose, finish


def play(client, chapter):
    client.rpc("initialize", dict(
        protocolVersion="2025-06-18", capabilities={},
        clientInfo=dict(name="character-quest-check", version="1"),
    ))
    if chapter == "mina":
        client.at("MnStPaulUnionDepot", 19, 16)
        facing = "left"
        destination = "BRAINERD"
    else:
        client.at("MnIronDrillCore", 9, 9)
        client.move("up", 3)
        client.move("left")
        client.at("MnIronDrillCore", 8, 6)
        facing = "down"
        destination = "GRAND RAPIDS"
    for yes in [False, True]:
        client.press(facing)
        client.press("a")
        choose(client, yes)
        pages = finish(client)
        if yes:
            assert destination in pages, pages
    client.press(facing)
    client.press("a")
    pages = finish(client)
    assert pages, "acknowledged quest must retain a spoken follow-up"
    client.press("start")
    assert "PACK" in client.observe()["content"][0]["text"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--client", type=Path, required=True)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--save", type=Path, required=True)
    parser.add_argument("--chapter", choices=["mina", "finn"], required=True)
    args = parser.parse_args()
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen(
            [str(args.client.resolve()), "mcp", str(args.pack.resolve()),
             "--load", str(args.save.resolve())],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
            text=True, bufsize=1,
        )
        try:
            play(Client(process), args.chapter)
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0, "client exited unsuccessfully"
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr was not byte-empty"
    print(f"PASS: {args.chapter} native refusal/retry, spoken quest, repeat dialogue and START; byte-empty stderr.")


if __name__ == "__main__":
    main()
