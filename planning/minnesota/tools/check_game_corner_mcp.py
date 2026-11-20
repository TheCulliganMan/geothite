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
OTHER_PRIZES = [("RARE_CANDY", 500), ("PP_UP", 1000), ("TM_THUNDER", 5500),
                ("TM_BLIZZARD", 5500), ("TM_FIRE_BLAST", 5500)]


class Review:
    def __init__(self, args):
        self.args = args
        self.pack = load_pack(args.pack)
        current = "northstar-arcade-frontage-v1" in self.pack["report"]["manifests"]
        self.offers = [("QUICK_CLAW" if current else "MOON_STONE", 300), *OTHER_PRIZES]
        self.results = []
        args.output.mkdir(parents=True, exist_ok=False)

    def speech(self, label):
        lines = self.pack["data"]["maps"][ROOM]["scripts"][label]
        return " ".join(json.loads(row["args"][0]) for row in lines
                        if row["command"] in {"text", "para", "line", "cont"})

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
        if accept is not None:
            # YES/NO can be visible behind an unread page. Select NO first to
            # distinguish a live cursor from a prompt still waiting on the text.
            for _ in range(16):
                if client.status()["mode"] == "yesno":
                    client.press("down")
                    prompt = client.observe()["structuredContent"]["prompt"]
                    if any(line["text"].strip() == "> NO" for line in prompt):
                        if accept:
                            client.press("up")
                        client.press("a")
                        break
                client.press("a")
            else:
                raise AssertionError(f"prize {index} never produced a selectable confirmation")
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
                assert self.speech("MnCornerRulesText") in pages, pages
                if repeat == 0:
                    assert self.speech("MnCornerWelcomeText") in pages, pages
                state = self.inspect(active)
                assert state["coins"] == 20 and state["items"]["COIN_CASE"] == 1 and state["welcome"]
            pages = self.prize(c, 0, accept=None)
            assert self.speech("MnCornerPoor") in pages, pages
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
        for index, (item, price) in enumerate(self.offers):
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
                assert self.speech("MnCornerFull" if full else "MnCornerThanks") in pages, pages
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
                label = "MnCornerMoneyShortText" if kind == "poor" else "MnCornerCoinLimitText"
                assert self.speech(label) in pages, pages
            finally:
                self.close(c, errors)
            self.passed(f"coin exchange rejects {kind} wallet without charging", state=state)

    def games(self):
        fixture = self.fixture("fresh", "games")
        c, active, errors = self.open(fixture, "games")
        def wait(frames=120):
            c.call("hold_button", button="select", frames=frames)
        def display():
            return [row["text"] for row in c.observe()["structuredContent"]["dialogue"]]

        def play(button):
            c.press(button)
            wait()

        try:
            self.enter(c)
            walk(c, self.pack, (3, 9)); c.press("left"); c.press("a"); finish(c)
            walk(c, self.pack, (5, 6)); c.press("right"); play("a")
            assert display() == ["SLOT MACHINE", "COINS 0020", "BET 3 PAY 0000", "BET 3"]
            play("a")
            assert display() == ["SLOT MACHINE", "COINS 0017", "BET 3 PAY 0000", "PRESS A"]
            for _ in range(3):
                play("a")
            for _ in range(10):
                wait()
            slot_result = display()
            slot_coins = int(slot_result[1].removeprefix("COINS "))
            slot_payout = slot_coins - 17
            assert slot_payout in (0, 6, 8, 10, 15, 50, 300), slot_result
            assert slot_result[-1] == (f"WIN {slot_payout}" if slot_payout else "DARN"), slot_result
            play("a")
            assert display()[-1] == "PLAY AGAIN?"
            play("b"); finish(c)
            c.at(ROOM, 5, 6)
            walk(c, self.pack, (14, 6))
            # The native client writes its autosave after control returns to the map.
            state = self.inspect(active)
            assert state["coins"] == slot_coins, state
            c.press("left"); play("a")
            assert display() == ["CARD FLIP", f"COINS {slot_coins:04} PAY 00", "PLAY WITH THREE", "COINS?"]
            play("a")
            assert display() == ["CARD FLIP", f"COINS {slot_coins-3:04} PAY 00", "CHOOSE A CARD."]
            play("a")
            assert display()[-1] == "PLACE YOUR BET."
            # The native top-row group wager covers half the deck and pays six.
            play("up"); play("up"); play("a")
            for _ in range(10):
                wait()
            card_result = display()
            play("a")
            assert display()[-2:] == ["WANT TO PLAY", "AGAIN?"]
            play("b"); finish(c)
            c.at(ROOM, 14, 6)
            c.move("down")
            c.at(ROOM, 14, 7)
            state = self.inspect(active)
            cards = state["cards"]
            assert cards["phase"] == "quit" and len(cards["deck"]) == 24, cards
            assert sum(cards["discard_pile"]) == 1, cards
            payout = 6 if cards["face_up_card"] & 3 < 2 else 0
            assert state["coins"] == slot_coins - 3 + payout, state
            assert card_result[-1] == ("YEAH!" if payout else "DARN…"), card_result
        finally:
            self.close(c, errors)
        self.passed("slots stake, three reel stops and result; Card Flip stake, reveal and result; quit to movement",
                    slot_result=slot_result, card_result=card_result, state=state)


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
