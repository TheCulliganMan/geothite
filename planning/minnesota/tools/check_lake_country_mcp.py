#!/usr/bin/env python3
"""Actual stdio lake-country journeys from played controller checkpoints."""
import argparse
from pathlib import Path
import re
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
    road(client, pack, (24, 0), "MnR05", 3, encounter=escape_wild)
    travel(client, "up", "MnStCloud", (24, 35))
    same_map_travel(client, "up", "MnStCloud", (24, 34))
    walk(client, pack, (24, 4))
    client.move("up")
    assert "WIND BADGE" in finish(client)
    client.at("MnStCloud", 24, 4)
    walk(client, pack, (24, 34))
    travel(client, "down", "MnR05C00R00", (24, 0))
    road(client, pack, (22, 111), "MnR05", 3, encounter=escape_wild)
    travel(client, "down", "MnMinneapolis", (32, 0))
    same_map_travel(client, "down", "MnMinneapolis", (32, 4))


def route_six(client, pack):
    client.at("MnStCloud", 24, 4)
    assert client.status()["badges"] == 5
    travel(client, "up", "MnR06C00R01", (16, 39))
    road(client, pack, (42, 0), "MnR06", 2, encounter=escape_wild)
    travel(client, "up", "MnBrainerd", (32, 47))
    same_map_travel(client, "up", "MnBrainerd", (32, 46))
    walk(client, pack, (25, 20))
    client.press("up")
    client.press("a")
    before = finish(client)
    assert "SIVREN" in before, before
    walk(client, pack, (32, 46))
    travel(client, "down", "MnR06C00R00", (42, 0))
    road(client, pack, (16, 87), "MnR06", 2, encounter=escape_wild)
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


def earned_brainerd_return(client, pack):
    client.at("MnBrainerd", 32, 46)
    assert client.status()["badges"] == 5
    walk(client, pack, (25, 20))
    client.press("up")
    client.press("a")
    pages = finish(client)
    assert "SIVREN" in pages, pages
    walk(client, pack, (9, 22))
    travel(client, "up", "MnBrainerdPokecenter1F", (10, 13))
    walk(client, pack, (7, 5))
    client.press("up")
    client.press("a")
    finish(client)
    health = [match.groups() for line in client.observe()["structuredContent"]["info"]
              if (match := re.search(r"L\d+\s+(\d+)/(\d+)", line["text"]))]
    assert health and all(hp == maximum for hp, maximum in health), health
    walk(client, pack, (10, 14))
    travel(client, "down", "MnBrainerd", (9, 22))
    walk(client, pack, (32, 46))
    travel(client, "down", "MnR06C00R00", (42, 0))
    road(client, pack, (16, 87), "MnR06", 2, encounter=escape_wild)
    travel(client, "down", "MnStCloud", (24, 0))
    same_map_travel(client, "down", "MnStCloud", (24, 4))


def escape_wild(client):
    """Select the displayed production RUN command; never resolve an engine turn."""
    for _ in range(256):
        state = client.status()
        view = client.observe()["structuredContent"]
        assert not view["dialogue"] or state["dialogue"], (
            "compact status omitted visible native dialogue", state, view["dialogue"])
        if (state["mode"] == "overworld" and not state["dialogue"]
                and not client.call("flow_state")["structuredContent"]["animating"]):
            return
        choices = [line["text"].strip() for line in view["menu"]]
        if any(line.lstrip("> ") == "RUN" for line in choices):
            selected = next(line.lstrip("> ") for line in choices if line.startswith(">"))
            client.press({"FIGHT": "down", "PKMN": "down", "PACK": "right", "RUN": "a"}[selected])
        else:
            assert not choices, ("unexpected menu during native RUN", choices)
            client.press("a")
    raise AssertionError("native RUN did not release the encounter")


def earned_wind_mart(client, pack):
    """Observe/cancel the real earned TM sale without spending that checkpoint."""
    client.at("MnStCloudPokecenter1F", 10, 13)
    assert client.status()["badges"] == 4
    walk(client, pack, (10, 14))
    travel(client, "down", "MnStCloud", (6, 10))
    walk(client, pack, (14, 10))
    travel(client, "up", "MnStCloudMart", (6, 5))
    walk(client, pack, (3, 3))
    client.press("left")
    client.press("a")
    for _ in range(32):
        choices = [line["text"].strip() for line in client.observe()["structuredContent"]["menu"]]
        if any(line.lstrip("> ") == "BUY" for line in choices):
            break
        client.press("a")
    else:
        raise AssertionError("native mart top did not open")
    client.press("down")
    client.press("a")
    name = pack["data"]["items"]["TM_THUNDERPUNCH"]["name"].replace("_", " ")
    for _ in range(64):
        choices = [line["text"].strip() for line in client.observe()["structuredContent"]["menu"]]
        selected = next((line for line in choices if line.startswith(">")), "")
        if name in selected:
            assert "INVALID" not in selected and "01" in selected, selected
            break
        client.press("down")
    else:
        raise AssertionError(("owned TM not selectable", name, choices))
    client.press("a")
    page = client.observe()["content"][0]["text"]
    assert "How many?" in page, page
    client.press("b")
    choices = client.observe()["structuredContent"]["menu"]
    assert any(name in line["text"] and "01" in line["text"] for line in choices), choices
    for _ in range(64):
        state = client.status()
        view = client.observe()["structuredContent"]
        if state["mode"] == "overworld" and not state["dialogue"] and not view["menu"]:
            break
        client.press("b" if view["menu"] else "a")
    walk(client, pack, (6, 6))
    travel(client, "down", "MnStCloud", (14, 10))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-lake-country.crystalpack")
    parser.add_argument("--save", type=Path, required=True)
    parser.add_argument("--mode", choices=["r05", "r06", "ranger", "brainerd-return", "wind-mart"], required=True)
    scope = parser.add_mutually_exclusive_group()
    scope.add_argument("--earned-campaign", action="store_true",
                        help="checkpoint was exported by the earned Winona campaign; no prior-badge/party fixture")
    scope.add_argument("--boosted-review", action="store_true",
                       help="actual campaign checkpoint with user-authorized supplied levels")
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
            {"r05": route_five, "r06": route_six, "ranger": ranger,
             "brainerd-return": earned_brainerd_return, "wind-mart": earned_wind_mart}[args.mode](client, pack)
            client.press("start")
            assert "PACK" in client.observe()["content"][0]["text"]
            final = client.status()
            assert final["badges"] == initial["badges"] and final["money"] == initial["money"]
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr was not byte-empty"
    source = ("boosted story review" if args.boosted_review else
              "earned Winona campaign" if args.earned_campaign else "local setup")
    print(f"PASS: actual stdio {args.mode}/Start from {source}; unchanged money/badges and byte-empty stderr.")


if __name__ == "__main__":
    main()
