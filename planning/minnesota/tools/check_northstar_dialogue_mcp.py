#!/usr/bin/env python3
"""Read the revised native scenes over stdio from same-pack chapter saves.

The saves come from production-input chapter tests with explicitly supplied
upstream access/parties. This is not a fresh or unboosted campaign claim.
"""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_north_shore_story_mcp import choose, finish
from check_winona_mcp import walk
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]
STORY = json.loads((ROOT / "planning/minnesota/story.json").read_text())


def lines(scene, label):
    if scene in STORY["supporting_dialogue"]:
        return STORY["supporting_dialogue"][scene][label]
    milestone = next(m for m in STORY["milestones"] if m["id"] == scene)
    return milestone["native_texts"][label]


def read(client, scene, label):
    raw = lines(scene, label)
    start = client.status()
    for offset in range(0, len(raw), 2):
        expected = raw[offset:offset + 2]
        view = client.observe()["structuredContent"]
        actual = [line["text"] for line in view["dialogue"]]
        assert actual == expected, (label, actual, expected)
        client.press("down")
        now = client.status()
        assert (now["map"], now["position"]) == (start["map"], start["position"])
        assert [line["text"] for line in client.observe()["structuredContent"]["dialogue"]] == expected
        client.press("a")


def rescue(client, pack):
    office = "MnMinneapolisDispatchOffice"
    client.at(office, 9, 9)
    walk(client, pack, (10, 11))
    client.press("left")
    client.press("a")
    read(client, "MS03", "MnMPDispatchLockedText")
    finish(client)
    walk(client, pack, (11, 6))
    for accepted in [False, True]:
        client.press("right")
        client.press("a")
        choose(client, accepted)
        read(client, "MS03_RESCUE", "MnMPCellReleaseText" if accepted else "MnMPCellDeclinedText")
        finish(client)
    client.move("right", 3)
    client.at(office, 14, 6)
    assert "MN_ICE_CAPTIVE_DISPATCHER" not in json.dumps(client.call("map_info"))
    # Cross the actually opened cell: the source grid deliberately retains
    # its closed block, while production movement uses the saved override.
    client.move("left", 3)
    client.at(office, 11, 6)
    walk(client, pack, (10, 11))
    client.press("left")
    client.press("a")
    read(client, "MS03", "MnMPDispatchReadText")
    finish(client)
    client.press("left")
    client.press("a")
    read(client, "MS03", "MnMPDispatchDoneText")
    finish(client)


def harbor(client, pack):
    del pack
    client.at("MnDuluthHarborWorkshop", 10, 5)
    client.press("right")
    client.press("a")
    read(client, "MS10", "MN_HARBOR_RECEIVERText")
    finish(client)
    client.press("right")
    client.press("a")
    read(client, "MS10", "MN_HARBOR_RECEIVERDoneText")
    finish(client)


def relay(client, pack):
    room = "MnBoundaryWatersSurveyShelter"
    walk(client, pack, (12, 4))
    for accepted in [False, True]:
        client.press("up")
        client.press("a")
        choose(client, accepted)
        if not accepted:
            read(client, "MS12", "MnBoundarySurveyReceiverDeclinedText")
            finish(client)
    for _ in range(64):
        if any(p["text"] == "Picture: KETTLURK" for p in client.observe()["structuredContent"]["prompt"]):
            break
        client.press("a")
    else:
        raise AssertionError("native picture must precede the shutdown")
    client.press("down")
    client.at(room, 12, 4)
    assert any(p["text"] == "Picture: KETTLURK" for p in client.observe()["structuredContent"]["prompt"])
    client.press("b")
    expected = lines("MS12", "MnBoundarySurveyReceiverDoneText")[:2]
    for _ in range(64):
        if [r["text"] for r in client.observe()["structuredContent"]["dialogue"]] == expected:
            break
        client.press("a")
    else:
        raise AssertionError("shutdown result did not follow the native picture")
    read(client, "MS12", "MnBoundarySurveyReceiverDoneText")
    finish(client)
    client.press("up")
    client.press("a")
    read(client, "MS12", "MnBoundarySurveyReceiverAlreadyText")
    finish(client)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-dialogue.crystalpack")
    parser.add_argument("--client", type=Path, default=ROOT / "target/northstar-story/wasm-dev/geothite")
    parser.add_argument("--save", type=Path, required=True)
    parser.add_argument("--mode", choices=["rescue", "harbor", "relay"], required=True)
    args = parser.parse_args()
    pack = load_pack(args.pack)
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen([str(args.client), "mcp", str(args.pack), "--load", str(args.save)],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                                   text=True, bufsize=1)
        client = Client(process, timeout=90)
        try:
            client.rpc("initialize", dict(protocolVersion="2025-06-18", capabilities={},
                                          clientInfo=dict(name="northstar-dialogue-review", version="1")))
            before = client.status()
            {"rescue": rescue, "harbor": harbor, "relay": relay}[args.mode](client, pack)
            client.press("start")
            assert "PACK" in client.observe()["content"][0]["text"]
            after = client.status()
            assert (after["money"], after["badges"], after["pokedex"]) == (before["money"], before["badges"], before["pokedex"])
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr must remain byte-empty"
    print(f"PASS: actual stdio {args.mode}; every revised page, input ownership, repeat/Start, no resource rewards and empty stderr. Explicit chapter fixture; not a new campaign.")


if __name__ == "__main__":
    main()
