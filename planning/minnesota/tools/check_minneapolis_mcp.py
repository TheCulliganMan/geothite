#!/usr/bin/env python3
"""Play the native ICE rescue over stdio from an earned enforcer-win save."""

import argparse
import json
from pathlib import Path
import select
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[3]


class Client:
    def __init__(self, process, timeout=30):
        self.process = process
        self.sequence = 0
        self.timeout = timeout

    def rpc(self, method, params):
        self.sequence += 1
        request = dict(jsonrpc="2.0", id=self.sequence, method=method, params=params)
        self.process.stdin.write(json.dumps(request) + "\n")
        self.process.stdin.flush()
        assert select.select([self.process.stdout], [], [], self.timeout)[0], "live MCP observation timed out"
        response = json.loads(self.process.stdout.readline())
        assert "error" not in response, response
        result = response["result"]
        assert not result.get("isError"), result
        return result

    def call(self, name, **arguments):
        return self.rpc("tools/call", dict(name=name, arguments=arguments))

    def status(self):
        return self.call("status")["structuredContent"]

    def observe(self):
        return self.call("observe", detail="full")

    def press(self, button):
        return self.call("press", button=button)

    def move(self, direction, steps=1):
        return self.call("move", direction=direction, steps=steps)

    def at(self, map_name, x, y):
        status = self.status()
        assert status["map"] == map_name and status["position"] == dict(x=x, y=y), status

    def finish_dialogue(self):
        pages = []
        for _ in range(64):
            if not self.status()["dialogue"]:
                return "\n".join(pages)
            pages.append(self.observe()["content"][0]["text"])
            self.press("a")
        raise AssertionError("dialogue did not return control")

    def choose(self, yes):
        target = "YES" if yes else "NO"
        for _ in range(64):
            prompt = self.observe()["structuredContent"]["prompt"]
            if any(line["text"].strip() == "> " + target for line in prompt):
                self.press("a")
                return
            self.press(("up" if yes else "down") if self.status()["mode"] == "yesno" else "a")
        raise AssertionError("displayed choice was not selectable")


def play(client):
    client.rpc("initialize", dict(
        protocolVersion="2025-06-18", capabilities={},
        clientInfo=dict(name="minneapolis-check", version="1"),
    ))
    office = "MnMinneapolisDispatchOffice"
    client.at(office, 9, 9)
    initial = client.status()
    assert initial["badges"] == 2
    client.move("right")
    client.move("down", 2)
    client.at(office, 10, 11)
    client.press("left")
    client.press("a")
    assert "You need the clerk" in client.finish_dialogue(), "record requires the separate rescue"
    client.move("up", 5)
    client.move("right")
    client.at(office, 11, 6)
    for yes, expected in [(False, "The latch is shut."), (True, "The clerk is free.")]:
        client.press("right")
        client.press("a")
        client.choose(yes)
        pages = client.finish_dialogue()
        assert expected in pages, pages
    client.move("right", 3)
    client.at(office, 14, 6)
    client.move("up")
    client.at(office, 14, 5)
    assert "MN_ICE_CAPTIVE_DISPATCHER" not in json.dumps(client.call("map_info"))
    client.move("down")
    client.move("left", 4)
    client.move("down", 5)
    client.at(office, 10, 11)
    client.press("left")
    client.press("a")
    record = client.finish_dialogue()
    assert all(name in record for name in ["TEAM ICE", "TEAM LODE", "ST PAUL"]), record
    client.press("left")
    client.press("a")
    assert "Follow the record." in client.finish_dialogue()
    client.move("down", 3)
    for _ in range(8):
        if client.status()["map"] == "MnMinneapolis":
            break
        client.press("down")
    client.at("MnMinneapolis", 27, 34)
    client.move("right", 5)
    client.move("up", 20)
    client.move("left", 5)
    client.at("MnMinneapolis", 27, 14)
    home = "MnMinneapolisDispatcherHouse"
    for _ in range(8):
        if client.status()["map"] == home:
            break
        client.press("up")
    client.at(home, 5, 6)
    assert "MN_MP_RESCUED_DISPATCHER" in json.dumps(client.call("map_info"))
    client.move("up", 2)
    client.move("right")
    client.at(home, 6, 4)
    client.press("up")
    client.press("a")
    assert "They made it home." in client.finish_dialogue()
    client.press("start")
    assert "PACK" in client.observe()["content"][0]["text"]
    final = client.status()
    assert final["badges"] == 2 and final["money"] == initial["money"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-minneapolis-canonical-story.crystalpack")
    parser.add_argument("--save", type=Path, required=True, help="earned enforcer-win save at office (9,9)")
    parser.add_argument("--client", type=Path, default=ROOT / "target/northstar-story/wasm-dev/geothite")
    args = parser.parse_args()
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen(
            [str(args.client), "mcp", str(args.pack), "--load", str(args.save)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
            text=True, bufsize=1,
        )
        try:
            play(Client(process))
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0, "client exited unsuccessfully"
        stderr.seek(0)
        assert stderr.read() == b"", "gameplay stderr was not byte-empty"
    print("PASS: actual stdio rescue refusal/retry, opened cell, ICE/Lode record, return home and START; clean exit and byte-empty stderr.")


if __name__ == "__main__":
    main()
