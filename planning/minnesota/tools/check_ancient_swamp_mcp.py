#!/usr/bin/env python3
"""Verify optional Vergloom refusal, RUN and capture through production stdio."""

import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_north_shore_story_mcp import choose, finish


def screen(client):
    return client.observe()["content"][0]["text"]


def battle(client):
    client.press("up")
    client.press("a")
    choose(client, True)
    for _ in range(64):
        view = screen(client)
        if all(action in view for action in ["FIGHT", "PKMN", "PACK", "RUN"]):
            assert client.status()["mode"] == "staticwildbattle"
            assert "Foe VERGLOOM" in view
            return
        client.press("a")
    raise AssertionError("native optional battle commands never appeared")


def return_to_trail(client):
    # A press clocks one frame. Capture owns elapsed animation frames even
    # when the text renderer has no pixels to animate. Advance the shared
    # production clock with original joypad input, then decline nicknaming.
    pages = []
    for _ in range(16):
        pages.append(screen(client))
        state = client.status()
        if state["mode"] == "overworld" and not state["dialogue"]:
            return "\n".join(pages)
        client.press("a")
        client.call("hold_button", button="b", frames=120)
    raise AssertionError("native battle did not release the trail")


def play(client, capture):
    client.rpc("initialize", dict(
        protocolVersion="2025-06-18", capabilities={},
        clientInfo=dict(name="ancient-swamp-check", version="1"),
    ))
    client.at("MnR08C00R00", 59, 24)
    assert "MN_ANCIENT_MIRE_VERGLOOM" in str(client.call("map_info"))
    initial = client.status()
    if capture:
        client.press("up")
        client.press("a")
        choose(client, False)
        assert "VERGLOOM stays by the pool" in finish(client)
        assert "MN_ANCIENT_MIRE_VERGLOOM" in str(client.call("map_info"))
    battle(client)
    client.press("down")
    if not capture:
        client.press("right")
    assert ("> PACK" if capture else "> RUN") in screen(client)
    client.press("a")
    if capture:
        client.press("right")  # BALL pocket, after the empty ITEM pocket.
        assert "> MASTER BALL x1" in screen(client)
        client.press("a")
        client.press("a")
        assert "MASTER BALL" in screen(client)
    pages = return_to_trail(client)
    assert "MN_ANCIENT_MIRE_VERGLOOM" not in str(client.call("map_info"))
    final = client.status()
    assert final["badges"] == initial["badges"]
    assert final["money"] == initial["money"]
    if capture:
        assert final["pokedex"]["owned"] == initial["pokedex"]["owned"] + 1
        assert "VERGLOOM" in screen(client)
        assert "MASTER BALL" in pages
    else:
        assert final["pokedex"]["owned"] == initial["pokedex"]["owned"]
    client.press("start")
    assert "PACK" in screen(client)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--save", type=Path, required=True,
                        help="reached revealed fixture save bound to this exact pack")
    parser.add_argument("--client", type=Path, required=True)
    parser.add_argument("--captured-save", type=Path,
                        help="native reached captured checkpoint for fresh Continue verification")
    args = parser.parse_args()
    for capture in [False, True]:
        with tempfile.TemporaryFile() as stderr:
            process = subprocess.Popen(
                [str(args.client), "mcp", str(args.pack), "--load", str(args.save)],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                text=True, bufsize=1,
            )
            try:
                play(Client(process, timeout=90), capture)
            finally:
                process.stdin.close()
                assert process.wait(timeout=30) == 0
            stderr.seek(0)
            assert stderr.read() == b"", "gameplay stderr was not byte-empty"
    if args.captured_save:
        with tempfile.TemporaryFile() as stderr:
            process = subprocess.Popen(
                [str(args.client), "mcp", str(args.pack), "--load", str(args.captured_save)],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                text=True, bufsize=1,
            )
            try:
                client = Client(process, timeout=90)
                client.rpc("initialize", dict(
                    protocolVersion="2025-06-18", capabilities={},
                    clientInfo=dict(name="ancient-swamp-continue", version="1"),
                ))
                assert client.status()["pokedex"]["owned"] >= 1
                assert "VERGLOOM" in screen(client)
                assert "MN_ANCIENT_MIRE_VERGLOOM" not in str(client.call("map_info"))
                client.press("start")
                assert "PACK" in screen(client)
            finally:
                process.stdin.close()
                assert process.wait(timeout=30) == 0
            stderr.seek(0)
            assert stderr.read() == b"", "continued gameplay stderr was not byte-empty"
    print("PASS: fresh stdio RUN, refusal/reoffer, native Master Ball capture, "
          "party/Dex, actor removal, Start; clean exit and byte-empty stderr.")


if __name__ == "__main__":
    main()
