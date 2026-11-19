#!/usr/bin/env python3
"""Read the Fen's directions, change-of-state pages and return from one compiled successor."""

import argparse
from itertools import product
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import native_pages, require
from render_reference import load_pack

FEN = "MnLayeredFen"
SPEECH = runpy.run_path(str(HERE.parent / "fen-trail-v01/check-fen.py"))
FIELDS = {
    "observer_intro": ("saplings", "between the pools"),
    "observer_partial": ("between the pools", "northwest corner"),
    "early": ("walk south", "cove", "northwest corner"),
    "wrong_order": ("east of the cove", "north bank", "little fir by the tall pines", "low branch"),
    "rib": ("fir branch", "moss"),
    "pool_selected": ("little fir east of the cove",),
    "layers_settle": ("root dips", "pool ripples", "fir shakes its moss"),
    "folded_path": ("pines part", "north end", "tracks", "hollow"),
    "observer_after": ("tracks go north", "east trail", "back to the woods"),
}
UPDATED = {"observer_after", "observer_intro", "observer_partial", "early", "wrong_order",
           "observer_compare", "root_selected", "pool_selected", "rib", "rib_selected", "folded_path", "layers_settle", "caught"}


def guide_pages(room):
    visit, follow, printed = (SPEECH[name] for name in ("Visit", "follow", "printed"))
    reads, met, opened = (SPEECH[name] for name in ("READS", "SEEN", "OPEN"))
    hints = ("observer_partial", "early", "wrong_order")
    for seen in product((False, True), repeat=3):
        observations = {flag for flag, read in zip(reads, seen) if read}
        for returning in (False, True):
            flags = observations | ({met} if returning else set())
            state = visit(flags.copy())
            actual = printed(follow(room, "MnFenObserver", state))
            if all(seen):
                require(actual == ["layers_settle", "folded_path"] and opened in state.flags,
                        "the guide must open the way for an interrupted three-observation save")
            else:
                next_page = hints[seen.index(False)] if returning or observations else "observer_intro"
                require(actual == [next_page] and observations <= state.flags and opened not in state.flags,
                        f"guide sent a returning visitor back to a completed clue: {seen}")
                for index, found in enumerate(seen):
                    if found:
                        for site in SPEECH["CONTACTS"][index]:
                            reread = visit(flags.copy())
                            require(printed(follow(room, site, reread)) == [next_page]
                                    and reread.flags == flags, f"returning to {site} lost the next clue")
    for caught in (False, True):
        state = visit({opened} | ({SPEECH["CAUGHT"]} if caught else set()))
        require(printed(follow(room, "MnFenObserver", state)) == ["observer_after"],
                "the guide should acknowledge the opening after a battle or capture")
    SPEECH["encounters"](room)


def check(parent, candidate):
    old = load_pack(parent)["data"]["maps"][FEN]
    data = load_pack(candidate)["data"]
    room = data["maps"][FEN]
    SPEECH["check_pages"](data, room)
    guide_pages(room)
    old_pages = {k: native_pages(v["commands"]) for k, v in old["script_text_bodies"].items()}
    actual = {k: native_pages(v["commands"]) for k, v in room["script_text_bodies"].items()}
    require(old_pages.keys() == actual.keys(), "unexpected extra/lost Fen pages")
    require({k for k in actual if actual[k] != old_pages[k]} == {"MnFenText_" + k for k in UPDATED},
            "changed text outside the route's directions, reveal and catch")
    words = {k.removeprefix("MnFenText_"): " ".join(sum(v, [])).lower() for k, v in actual.items()}
    for k, clues in FIELDS.items():
        require(all(c in words[k] for c in clues), f"{k} does not describe the available landmark")
    for k, text in words.items():
        require(all(t not in text for t in ("ridge", "stone", "try the worn root again", "take that new path home")),
                f"obsolete route or repeat task in {k}")
    require("battle mitwist?" in words["ask"], "encounter tells players the actual consequence")
    print(f"{candidate.name}: 13 revised Fen pages, 16 guide returns and six revisit aliases, northern hollow and eastern exit, honest battle offer. All other Fen pages match the parent.")


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("parent", type=Path)
    cli.add_argument("candidate", type=Path)
    args = cli.parse_args()
    check(args.parent, args.candidate)
