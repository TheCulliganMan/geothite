#!/usr/bin/env python3
"""Verify native Game Corner inputs from explicitly supplied local test fixtures.

The Rust game_corner_fixture example creates test-only placement/wallet saves.
All subsequent travel, dialogue, games and purchases use production MCP input.
Inspection reads saved results and never edits a running game or save.
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

ROOM = "MnMinneapolisCornerHome"
PRIZES = [("MOON_STONE", 300), ("RARE_CANDY", 500), ("PP_UP", 1000),
          ("TM_THUNDER", 5500), ("TM_BLIZZARD", 5500), ("TM_FIRE_BLAST", 5500)]


class Review:
    def __init__(self, args):
        self.args = args
        self.pack = load_pack(args.pack)
        self.results = []
        args.output.mkdir(parents=True, exist_ok=False)

    def fixture(self, kind, label):
        path = self.args.output / f"{label}-fixture.crystalsave"
        subprocess.run([str(self.args.fixture_tool), str(self.args.pack), kind, str(path)], check=True)
        return path

    def inspect(self, path):
        return json.loads(subprocess.check_output(
            [str(self.args.fixture_tool), str(self.args.pack), "inspect", str(path)], text=True))

    def open(self, fixture, label):
        active = self.args.output / f"{label}-active.crystalsave"
        errors = tempfile.TemporaryFile()
        process = subprocess.Popen([str(self.args.client), "mcp", str(self.args.pack),
                                    "--load", str(fixture), "--save", str(active)],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=errors,
                                   text=True, bufsize=1)
        client = Client(process, timeout=60)
        client.rpc("initialize", dict(protocolVersion="2025-06-18", capabilities={},
                                      clientInfo=dict(name="game-corner-review", version="1")))
        return client, active, errors

    def close(self, client, errors):
        client.process.stdin.close()
        assert client.process.wait(timeout=30) == 0
        errors.seek(0)
        assert errors.read() == b"", "production gameplay wrote to stderr"
        errors.close()

    def enter(self, client):
        client.at("MnMinneapolis", 56, 52)
        client.move("up")
        client.at(ROOM, 3, 12)

    def menu(self, client):
        for _ in range(40):
            if client.status()["menu"] is not None:
                return client.observe()["structuredContent"]["menu"]
            client.press("a")
        raise AssertionError("native prize menu never opened")

    def prize(self, client, index, accept=True):
        walk(client, self.pack, (16 if index < 3 else 18, 4))
        client.press("up")
        client.press("a")
        menu = self.menu(client)
        assert len(menu) >= 4, menu
        for _ in range(index % 3):
            client.press("down")
        client.press("a")
        if client.status()["mode"] == "yesno":
            client.choose(accept)
        return finish(client)

    def passed(self, label, **evidence):
        self.results.append(dict(case=label, **evidence))
        (self.args.output / "receipt.json").write_text(json.dumps(self.results, indent=2) + "\n")
        print("PASS:", label, flush=True)

    def fresh(self):
        fixture = self.fixture("fresh", "fresh")
        c, active, errors = self.open(fixture, "fresh")
        try:
            self.enter(c)
            walk(c, self.pack, (3, 9))
            for repeat in range(2):
                c.press("left")
                c.press("a")
                first = c.observe()["structuredContent"]["dialogue"]
                assert first
                c.press("down")
                c.at(ROOM, 3, 9)
                pages = finish(c)
                assert "COIN CASE" in pages if repeat == 0 else "prizes" in pages
                state = self.inspect(active)
                assert state["coins"] == 20 and state["items"]["COIN_CASE"] == 1 and state["welcome"]
            pages = self.prize(c, 0)
            assert "more coins" in pages, pages
            assert self.inspect(active)["coins"] == 20
            walk(c, self.pack, (5, 6))
            c.press("right")
            shutil.copy2(active, self.args.output / "slots-approach.crystalsave")
            walk(c, self.pack, (14, 6))
            c.press("left")
            shutil.copy2(active, self.args.output / "cards-approach.crystalsave")
            walk(c, self.pack, (3, 12))
            continued = self.args.output / "continue.crystalsave"
            shutil.copy2(active, continued)
        finally:
            self.close(c, errors)
        c, active, errors = self.open(continued, "continued")
        try:
            c.at(ROOM, 3, 12)
            for x in [2, 3]:
                walk(c, self.pack, (x, 12))
                travel(c, "down", "MnMinneapolis", (56, 52))
                c.move("up")
                c.at(ROOM, 3, 12)
            state = self.inspect(active)
            assert state["coins"] == 20 and state["welcome"] and state["items"]["COIN_CASE"] == 1
        finally:
            self.close(c, errors)
        self.passed("free case and once-only 20 coins; poor refusal; both exits; Continue", state=state)

    def prizes(self, full=False):
        fixture = self.fixture("full" if full else "rich", "full" if full else "rich")
        baseline = self.inspect(fixture)
        for index, (item, price) in enumerate(PRIZES):
            c, active, errors = self.open(fixture, f"{'full' if full else 'buy'}-{item}")
            try:
                self.enter(c)
                if not full:
                    pages = self.prize(c, index, accept=False)
                    before = self.inspect(active)
                    assert before["coins"] == 9999 and before["items"][item] == 0, pages
                pages = self.prize(c, index)
                state = self.inspect(active)
                assert state["coins"] == (9999 if full else 9999-price), (item, state, pages)
                assert state["items"][item] == (baseline["items"][item] if full else 1), (item, state, pages)
                assert ("full" if full else "Enjoy") in pages, pages
                assert state["money"] == 100000
            finally:
                self.close(c, errors)
            self.passed(f"{item}: {'full-stack refusal keeps coins' if full else 'cancel and confirmed redemption'}", state=state)

    def coins(self):
        fixture = self.fixture("fresh", "coins")
        c, active, errors = self.open(fixture, "coins")
        try:
            self.enter(c)
            walk(c, self.pack, (3, 4))
            c.press("up"); c.press("a")
            assert "COIN CASE" in finish(c)
            assert self.inspect(active)["coins"] == 0
            walk(c, self.pack, (3, 9))
            c.press("left"); c.press("a"); finish(c)
            walk(c, self.pack, (3, 4))
            c.press("up"); c.press("a")
            menu = self.menu(c)
            print("coin menu", menu, flush=True)
            c.press("a")
            pages = finish(c)
            state = self.inspect(active)
            assert state["coins"] == 70 and state["money"] == 99000, (state, pages)
            c.press("up"); c.press("a"); self.menu(c); c.press("down"); c.press("a")
            finish(c)
            state = self.inspect(active)
            assert state["coins"] == 570 and state["money"] == 89000, state
        finally:
            self.close(c, errors)
        self.passed("Coin Case required; buy 50/500 coins for 1000/10000 money", state=state)
        for kind in ["poor", "rich"]:
            fixture = self.fixture(kind, f"coins-{kind}")
            before = self.inspect(fixture)
            c, active, errors = self.open(fixture, f"coins-{kind}")
            try:
                self.enter(c); walk(c, self.pack, (3, 4))
                c.press("up"); c.press("a"); self.menu(c); c.press("a")
                pages = finish(c)
                state = self.inspect(active)
                assert state["coins"] == before["coins"] and state["money"] == before["money"], pages
                assert ("money" if kind == "poor" else "9999") in pages, pages
            finally:
                self.close(c, errors)
            self.passed(f"coin exchange rejects {kind} wallet without charging", state=state)

    def games(self):
        fixture = self.fixture("fresh", "games")
        c, active, errors = self.open(fixture, "games")
        def wait(frames=120):
            c.call("hold_button", button="select", frames=frames)
        try:
            self.enter(c)
            walk(c, self.pack, (3, 9)); c.press("left"); c.press("a"); finish(c)
            walk(c, self.pack, (5, 6)); c.press("right"); c.press("a")
            wait()
            state = self.inspect(active)
            print("slots entered", state, flush=True)
            assert state["slots"]["phase"] == "betting"
            c.press("a"); wait()
            state = self.inspect(active)
            assert state["slots"]["phase"] == "spinning" and state["coins"] == 17, state
            assert state["slots"]["bet"] == 3
            for _ in range(3):
                c.press("a"); wait()
            for _ in range(10):
                wait()
            state = self.inspect(active)
            print("slots resolved", state, flush=True)
            assert state["slots"]["phase"] == "result" and state["slots"]["next_reel"] == 4, state
            c.press("a"); c.press("b"); wait(); finish(c)
            c.at(ROOM, 5, 6)
            slot_coins = self.inspect(active)["coins"]
            walk(c, self.pack, (14, 6)); c.press("left"); c.press("a"); wait()
            c.press("a"); wait()
            state = self.inspect(active)
            print("cards staked", state, flush=True)
            assert state["cards"]["phase"] == "choose_card" and len(state["cards"]["deck"]) == 24, state
            assert state["coins"] == slot_coins-3
            c.press("a"); wait()
            # The native top-row group wager covers half the deck and pays six.
            c.press("up"); c.press("up"); c.press("a")
            for _ in range(10):
                wait()
            state = self.inspect(active)
            print("cards resolved", state, flush=True)
            assert state["cards"]["phase"] == "result" and sum(state["cards"]["discard_pile"]) == 1, state
            payout = 6 if state["cards"]["face_up_card"] & 3 < 2 else 0
            assert state["coins"] == slot_coins - 3 + payout, state
            c.press("a")
            assert self.inspect(active)["cards"]["phase"] == "play_again"
            c.press("b"); wait(); finish(c)
            c.at(ROOM, 14, 6)
            c.move("down")
            c.at(ROOM, 14, 7)
        finally:
            self.close(c, errors)
        self.passed("slots stake, three reel stops and result; Card Flip stake, reveal and result; quit to movement", state=state)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--client", type=Path, required=True)
    parser.add_argument("--fixture-tool", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--case", choices=["all", "fresh", "prizes", "full", "coins", "games"], default="all")
    args = parser.parse_args()
    review = Review(args)
    for case in ["fresh", "prizes", "full", "coins", "games"]:
        if args.case in ["all", case]:
            review.prizes(full=True) if case == "full" else getattr(review, case)()


if __name__ == "__main__":
    main()
