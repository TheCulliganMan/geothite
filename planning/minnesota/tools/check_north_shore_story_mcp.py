#!/usr/bin/env python3
"""Replay the native coast gate, optional lamp or care over actual stdio MCP."""
import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_river_story_mcp import same_map_travel
from check_winona_mcp import travel, walk
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]
TOWN = "MnGrandMarais"
GATE = "MnGrandMaraisShoreGate"


def finish(client):
    pages = []
    for _ in range(256):
        status = client.status()
        if status["mode"] == "overworld" and not status["dialogue"]:
            return " ".join(" ".join(pages).split())
        pages.extend(line["text"] for line in client.observe()["structuredContent"]["dialogue"])
        client.press("a")
    raise AssertionError("native dialogue/healing did not release input")


def choose(client, yes):
    """Read every page before choosing; pending menus may precede visible input."""
    def selected():
        view = client.observe()["structuredContent"]
        lines = [line["text"].strip() for line in view["prompt"] + view["menu"]]
        return next((line[2:] for line in lines if line.startswith("> ")), None)
    for _ in range(64):
        previous = selected()
        if previous in {"YES", "NO"}:
            client.press("down" if previous == "YES" else "up")
            changed = selected()
            if changed in {"YES", "NO"} and changed != previous:
                target = "YES" if yes else "NO"
                if changed != target:
                    client.press("up" if yes else "down")
                assert selected() == target
                client.press("a")
                return
        client.press("a")
    raise AssertionError("offer never released its pages to the visible YES/NO menu")


def gate(client, pack):
    state = client.status()
    lane = state["position"]["y"] - 4
    client.at(GATE, 8, 4 + lane)
    walk(client, pack, (1, 4 + lane))
    travel(client, "left", "MnR11C00R00", (42, 4 + lane))
    travel(client, "right", GATE, (1, 4 + lane))
    walk(client, pack, (8, 4 + lane))
    travel(client, "right", TOWN, (8, 18 + lane))
    walk(client, pack, (4, 8 + lane))
    travel(client, "left", "MnR12C01R01", (63, 8 + lane))
    travel(client, "right", TOWN, (0, 8 + lane))
    same_map_travel(client, "right", TOWN, (4, 8 + lane))


def part(client, pack):
    assert client.status()["map"] == "MnGrandMaraisFieldStation"
    walk(client, pack, (5, 4))
    client.press("up")
    client.press("a")
    choose(client, False)
    assert "It can wait here" in finish(client)
    client.press("up")
    client.press("a")
    choose(client, True)
    assert "spare lamp part" in finish(client)
    client.press("up")
    client.press("a")
    assert "relay is separate" in finish(client)
    walk(client, pack, (2, 6))
    travel(client, "down", TOWN, (6, 7))
    same_map_travel(client, "down", TOWN, (6, 8))
    walk(client, pack, (15, 28))
    travel(client, "up", "MnGrandMaraisLighthouse", (4, 9))
    same_map_travel(client, "up", "MnGrandMaraisLighthouse", (4, 8))
    walk(client, pack, (5, 4))
    client.press("up")
    client.press("a")
    assert "lamp can turn" in finish(client)
    client.press("up")
    client.press("a")
    assert "beacon turns" in finish(client)


def care(client, pack):
    client.at("MnGrandMaraisPokecenter", 3, 3)
    client.press("up")
    client.press("a")
    choose(client, True)
    finish(client)
    walk(client, pack, (4, 6))
    travel(client, "down", TOWN, (15, 9))
    same_map_travel(client, "down", TOWN, (15, 10))
    walk(client, pack, (8, 18))
    # This fixture arrived unqualified. Return through the checkpoint must
    # remain refused; entering the clinic must not grant badges or the signal.
    for _ in range(8):
        if client.status()["dialogue"]:
            break
        client.press("left")
    assert "BADGES" in finish(client)
    client.at(TOWN, 8, 18)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-north-shore-story.crystalpack")
    parser.add_argument("--client", type=Path, default=ROOT / "target/northstar-story/wasm-dev/geothite")
    parser.add_argument("--save", type=Path, required=True)
    parser.add_argument("--mode", choices=["gate", "part", "care"], required=True)
    args = parser.parse_args()
    pack = load_pack(args.pack)
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen([str(args.client), "mcp", str(args.pack), "--load", str(args.save)],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                                   text=True, bufsize=1)
        client = Client(process)
        try:
            client.rpc("initialize", dict(protocolVersion="2025-06-18", capabilities={},
                                          clientInfo=dict(name="north-shore-check", version="1")))
            initial = client.status()
            {"gate": gate, "part": part, "care": care}[args.mode](client, pack)
            client.press("start")
            assert "PACK" in client.observe()["content"][0]["text"]
            final = client.status()
            assert final["money"] == initial["money"] and final["badges"] == initial["badges"]
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr must remain byte-empty"
    print(f"PASS: actual stdio {args.mode}, native inputs/Start, unchanged money/badges and byte-empty stderr.")


if __name__ == "__main__":
    main()
