#!/usr/bin/env python3
"""Read all five new League introductions over real production stdio MCP."""
import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client


def play(client, index):
    client.rpc("initialize", dict(
        protocolVersion="2025-06-18", capabilities={},
        clientInfo=dict(name="league-dialogue-check", version="1"),
    ))
    names = ["FERN", "FLINT", "VEIL", "HALDEN", "CASEY"]
    map_name = "MnLeagueChampion" if index == 5 else f"MnLeagueRoom{index}"
    client.at(map_name, 5, 8)
    client.press("up")
    client.press("a")
    pages = []
    for _ in range(96):
        view = client.observe()
        pages.extend(line["text"] for line in view["structuredContent"]["dialogue"])
        screen = view["content"][0]["text"]
        if "FIGHT" in screen and "RUN" in screen:
            assert client.status()["mode"] == "trainerbattle"
            break
        client.press("a")
    else:
        raise AssertionError("acknowledged introduction did not enter native battle")
    text = " ".join(" ".join(pages).split())
    assert names[index - 1] in text, text
    marker = ["roadside grass", "mine supports", "late shift", "canoe trips", "I'm the CHAMPION"]
    assert marker[index - 1] in text, text
    assert "REED" not in text
    print(f"PASS {names[index - 1]}: {text}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--saves", type=Path, required=True)
    parser.add_argument("--client", type=Path, required=True)
    args = parser.parse_args()
    for index in range(1, 6):
        reached = sorted(args.saves.glob(f"winona-league-written-before-{index}-*.crystalsave"))
        assert len(reached) == 1, f"one reached same-pack checkpoint required: {index}"
        with tempfile.TemporaryFile() as stderr:
            process = subprocess.Popen(
                [str(args.client), "mcp", str(args.pack), "--load", str(reached[0])],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                text=True, bufsize=1,
            )
            try:
                play(Client(process, timeout=90), index)
            finally:
                process.stdin.close()
                assert process.wait(timeout=30) == 0
            stderr.seek(0)
            assert stderr.read() == b"", "native gameplay stderr was not byte-empty"
    print("PASS: all five distinct native introductions, acknowledged battle entry; clean stdio exits.")


if __name__ == "__main__":
    main()
