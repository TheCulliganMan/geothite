#!/usr/bin/env python3
"""Exercise both native shops through the production stdio controller.

Requires explicit local fixture saves at the two road approaches. Those saves
are setup, not evidence of earned regional access. All subsequent actions use
the client's production controller; no pack or save bytes are edited here.
"""

import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_winona_mcp import finish, travel, walk
from render_reference import load_pack


def open_client(args, save, output):
    stderr = tempfile.TemporaryFile()
    process = subprocess.Popen(
        [str(args.client), "mcp", str(args.pack), "--load", str(save), "--save", str(output)],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
        text=True, bufsize=1,
    )
    client = Client(process, timeout=60)
    client.rpc("initialize", dict(protocolVersion="2025-06-18", capabilities={},
                                  clientInfo=dict(name="north-shore-shops", version="1")))
    return client, stderr


def close_client(client, stderr):
    client.process.stdin.close()
    assert client.process.wait(timeout=30) == 0, "native client exited unsuccessfully"
    stderr.seek(0)
    assert stderr.read() == b"", "gameplay wrote to stderr"
    stderr.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--client", type=Path, required=True)
    parser.add_argument("--pie-fixture", type=Path, required=True)
    parser.add_argument("--smoke-fixture", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    pack = load_pack(args.pack)
    source = json.loads(args.source.read_text())
    for shop, fixture in zip(source["shops"], [args.pie_fixture, args.smoke_fixture]):
        room, exterior = shop["id"], shop["exterior_map"]
        active = args.output / f"{room}-active.crystalsave"
        client, stderr = open_client(args, fixture, active)
        initial = client.status()
        client.at(exterior, *shop["road_approach"])
        walk(client, pack, tuple(shop["porch"]))
        shutil.copy2(active, args.output / f"{room}-porch.crystalsave")
        travel(client, "up", room, (3, 6))
        shutil.copy2(active, args.output / f"{room}-arrival.crystalsave")
        staff = shop["room"]["residents"][0]
        at = (staff["tile"][0] - 1, staff["tile"][1])
        walk(client, pack, at)
        client.press("right")
        client.press("a")
        first = client.observe()["structuredContent"]["dialogue"]
        assert first and staff["lines"][0] in " ".join(line["text"] for line in first)
        client.press("down")
        client.at(room, *at)
        client.press("a")
        assert client.observe()["structuredContent"]["dialogue"] != first, "A did not change the visible page"
        finish(client)
        inside = args.output / f"{room}-inside.crystalsave"
        shutil.copy2(active, inside)
        close_client(client, stderr)
        client, stderr = open_client(args, inside, active)
        client.at(room, *at)
        for x in [2, 3]:
            walk(client, pack, (x, 6))
            travel(client, "down", exterior, tuple(shop["porch"]))
            if x == 2:
                travel(client, "up", room, (3, 6))
        x, y = shop["sign"]["tile"]
        walk(client, pack, (x, y + 1))
        client.press("up")
        client.press("a")
        assert "NORTH SHORE" in finish(client)
        walk(client, pack, tuple(shop["road_approach"]))
        client.press("start")
        assert "PACK" in client.observe()["content"][0]["text"]
        client.press("b")
        client.at(exterior, *shop["road_approach"])
        final = client.status()
        assert final["money"] == initial["money"] and final["badges"] == initial["badges"]
        outside = args.output / f"{room}-road.crystalsave"
        shutil.copy2(active, outside)
        close_client(client, stderr)
        client, stderr = open_client(args, outside, active)
        client.at(exterior, *shop["road_approach"])
        close_client(client, stderr)
        print(f"PASS: {room} road/door, staff pages/input ownership, both carpets, sign, Start, indoor/outdoor Continue and empty stderr", flush=True)


if __name__ == "__main__":
    main()
