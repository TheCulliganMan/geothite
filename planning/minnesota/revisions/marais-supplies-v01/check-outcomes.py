"""Exercise Marais archive branches for every gift result and pocket combination."""
from collections import Counter
from copy import deepcopy
from dataclasses import dataclass, field
from itertools import product

ROOM = "MnGrandMaraisFieldStation"
LIGHT = "MnGrandMaraisLighthouse"
ACTOR, DOOR = "MnGrandMaraisWelcome", "MnMaraisDoorwayPart"
SIGNAL, STARTED = ("EVENT_MN_GRAND_MARAIS_" + name for name in ("SIGNAL", "STARTER"))
OPEN, BALLS, POTIONS = ("EVENT_MN_MARAIS_" + name for name in ("SUPPLIES_OPEN", "BALLS_RECEIVED", "POTIONS_RECEIVED"))
PART, BALL, POTION = "MACHINE_PART", "POKE_BALL", "POTION"
READY = "SCENE_MN_FIELD_READY"


@dataclass
class State:
    flags: set = field(default_factory=set)
    items: Counter = field(default_factory=Counter)
    partners: list = field(default_factory=list)
    scene: str = "SCENE_MN_FIELD_STATION"


def visit(scripts, label, state, *, yes=None, location=None, blocked=()):
    """Follow installed commands with supplied native party/box and bag outcomes."""
    last, index = None, 0
    stack, text, choices = [], [], []
    for _ in range(150):
        ins = scripts[label][index]
        op, args = ins["command"], ins["args"]
        index += 1
        if op == "checkevent":
            last = args[0] in state.flags
        elif op == "checkitem":
            last = state.items[args[0]] > 0
        elif op == "setevent":
            state.flags.add(args[0])
        elif op == "clearevent":
            state.flags.discard(args[0])
        elif op == "setscene":
            state.scene = args[0]
        elif op in ("writetext", "jumptextfaceplayer"):
            text.append(args[0])
            if op == "jumptextfaceplayer":
                return text, choices
        elif op == "yesorno":
            assert yes is not None, ("unexpected decision", label)
            choices.append(yes)
            last = yes
        elif op in ("giveitem", "verbosegiveitem"):
            last = args[0] not in blocked
            if last:
                state.items[args[0]] += int(args[1]) if len(args) == 2 else 1
        elif op == "takeitem":
            assert state.items[args[0]] > 0, ("missing required item", args)
            state.items[args[0]] -= 1
        elif op == "givepoke":
            assert location in (0, 1, 2), ("unexpected gift", label)
            last = location
            if location != 2:
                state.partners.append((location, *args))
        elif op == "scall":
            stack.append((label, index))
            label, index = args[0], 0
        elif op in ("sjump", "iftrue", "iffalse", "ifequal", "ifnotequal"):
            take = (op == "sjump" or (op == "iftrue" and last) or (op == "iffalse" and not last)
                    or (op == "ifequal" and last == int(args[0])) or (op == "ifnotequal" and last != int(args[0])))
            if take:
                label, index = args[-1], 0
        elif op == "end":
            if not stack:
                return text, choices
            label, index = stack.pop()
        else:
            assert op in ("faceplayer", "opentext", "closetext", "promptbutton", "waitbutton"), (label, op)
    raise AssertionError(("runaway station conversation", label))


def unchanged_return(old, new):
    for repaired, remaining in product((False, True), (0, 2)):
        case = State({STARTED} | ({SIGNAL} if repaired else set()),
                     Counter({PART: int(not repaired), BALL: remaining, POTION: remaining}),
                     scene=READY)
        for scripts in (old, new):
            texts, choices = visit(scripts, ACTOR, case)
            assert texts == ["MnGrandMaraisTamarackThanksText" if repaired else "MnGrandMaraisReminderText"]
            assert not choices and not case.partners and not case.flags.intersection({OPEN, BALLS, POTIONS})
            assert case.items[BALL] == case.items[POTION] == remaining


def verify(old_maps, maps):
    old, new = (rooms[ROOM]["scripts"] for rooms in (old_maps, maps))
    unchanged_return(old, new)
    for repaired, partner, balls_full, items_full in product((False, True), (0, 1, 2), (False, True), (False, True)):
        prior = {SIGNAL} if repaired else set()
        s = State(set(prior), Counter({PART: int(not repaired)}))
        blocked = {item for item, full in [(BALL, balls_full), (POTION, items_full)] if full}
        offer = "MnMaraisGiftAfterRepairText" if repaired else "MnGrandMaraisWelcomeText"
        # Both exit lanes remain traversable after a refusal. Neither can adopt Birchit.
        assert visit(new, ACTOR, s, yes=False) == ([offer, "MnMaraisPartnerDeclinedText"], [False])
        assert visit(new, DOOR, s) == ([], [])
        assert s.flags == prior and not s.partners and s.scene != READY
        texts, choices = visit(new, ACTOR, s, yes=True, location=partner, blocked=blocked)
        assert choices == [True]
        if partner == 2:
            assert texts == [offer, "MnMaraisPartnerFullText"]
            assert s.flags == prior and not s.partners and s.scene != READY
            assert s.items[BALL] == s.items[POTION] == 0
            visit(new, ACTOR, s, yes=True, location=0)
            assert s.partners == [(0, "BIRCHIT", "8", "BERRY")]
            assert (s.items[BALL], s.items[POTION], s.scene) == (5, 3, READY)
            continue
        assert s.partners == [(partner, "BIRCHIT", "8", "BERRY")] and s.scene == READY
        first = ("MnMaraisPartnerBoxText" if partner else
                 "MnMaraisPartnerAfterRepairText" if repaired else "MnGrandMaraisPartnerText")
        waiting = {(True, True): "MnMaraisBothWaitingText", (True, False): "MnMaraisBallsWaitingText",
                   (False, True): "MnMaraisPotionsWaitingText"}.get((balls_full, items_full))
        received = ([] if balls_full else ["MnMaraisBallsReceivedText"]) + (
            [] if items_full else ["MnMaraisPotionsReceivedText"])
        assert texts == [offer, first] + received + ([waiting] if waiting else [])
        assert s.flags == prior | {STARTED} | ({OPEN} if blocked else set()) | (
            set() if balls_full else {BALLS}) | (set() if items_full else {POTIONS})
        assert (s.items[BALL], s.items[POTION]) == (0 if balls_full else 5, 0 if items_full else 3)
        if blocked:
            assert visit(new, ACTOR, s, blocked=blocked) == ([waiting], [])
        if len(blocked) == 2:
            # Recover the two pockets in either order, without duplicate claims.
            for still_full, amount, given, awaiting, final in [
                (POTION, 5, "Balls", "Potions", "Potions"),
                (BALL, 3, "Potions", "Balls", "Balls"),
            ]:
                retry = deepcopy(s)
                expected = [f"MnMarais{given}ReceivedText", f"MnMarais{awaiting}WaitingText"]
                assert visit(new, ACTOR, retry, blocked={still_full}) == (expected, [])
                assert sum(retry.items[item] for item in (BALL, POTION)) == amount and OPEN in retry.flags
                assert visit(new, ACTOR, retry) == ([f"MnMarais{final}ReceivedText"], [])
                assert (retry.items[BALL], retry.items[POTION], len(retry.partners)) == (5, 3, 1)
        if blocked:
            assert visit(new, ACTOR, s) == (
                (["MnMaraisBallsReceivedText"] if balls_full else []) +
                (["MnMaraisPotionsReceivedText"] if items_full else []), [])
        for _ in range(2):
            visit(new, ACTOR, s)
        assert (s.items[BALL], s.items[POTION], len(s.partners)) == (5, 3, 1)
        assert s.flags == prior | {STARTED, BALLS, POTIONS}
    for entry in (DOOR, ACTOR):
        s = State()
        visit(new, entry, s, blocked={PART})
        assert not s.items[PART] and not s.flags and not s.partners
        visit(new, entry, s)
        assert s.items[PART] == 1 and not s.partners
        assert visit(new, DOOR, s) == ([], [])
        keeper = maps[LIGHT]["scripts"]
        visit(keeper, "MnGrandMaraisKeeper", s, blocked={"SUPER_POTION"})
        assert SIGNAL in s.flags and not s.items[PART] and not s.items["SUPER_POTION"]
        assert visit(new, DOOR, s) == ([], [])
        for _ in range(2):
            visit(keeper, "MnGrandMaraisKeeper", s)
        assert s.items["SUPER_POTION"] == 1 and not s.partners
