"""Prepare explicit replacement profiles without activating unfinished species.

The existing Rust catalog/compiler remains the runtime validation/export path.
"""
import copy
import colorsys
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / "pokemon"
STAT_KEYS = ("hp", "attack", "defense", "speed", "special_attack", "special_defense")
LEVELS = (1, 1, 5, 9, 13, 17, 21, 25, 30, 35, 41, 47, 54)

# Each row is one deliberately authored family, in implementation-plan order.
STATS = [
    [[45,35,65,15,45,65], [65,45,85,20,65,85], [90,55,110,30,90,110]],
    [[45,60,35,80,35,45], [75,100,60,115,55,75]],
    [[40,30,40,40,65,55], [60,45,60,55,85,75], [80,60,75,80,115,90]],
    [[75,80,70,65,105,85]],
    [[60,70,65,30,30,45], [100,115,110,40,40,80]],
    [[45,55,40,65,35,40], [65,75,55,85,50,60], [85,105,75,105,65,85]],
    [[35,30,35,65,65,50], [50,45,50,80,90,70], [70,60,65,105,110,85]],
    [[40,55,45,65,30,40], [70,90,80,100,45,75]],
    [[35,35,35,55,65,50], [50,50,50,70,85,65], [75,60,65,95,115,80]],
    [[45,55,55,40,35,50], [75,105,80,110,65,75]],
    [[80,85,80,60,105,80]],
    [[45,50,45,60,50,50], [80,90,80,100,95,85]],
    [[70,60,90,105,95,80]],
    [[50,60,45,35,25,35], [70,85,70,45,35,50], [95,120,100,65,45,80]],
]
CLASSES = [
    ["Alcove", "Mantle", "Sanctuary"], ["Snow Brush", "Trail Sweep"],
    ["Gap Owlet", "Hollow Wing", "Forest Gap"], ["Warning"],
    ["Basalt Cub", "Talus"], ["Trail Cub", "Wayfinder", "Path Keeper"],
    ["Torpor", "Slow Pulse", "Still Wing"], ["Burrow Beak", "Bank Delver"],
    ["Snow Fluff", "Snow Vane", "Snow Spiral"], ["Ice Runner", "Sail Wing"],
    ["Touch Map"], ["New Tail", "Decoy Tail"], ["Pendulum"],
    ["Sett Cub", "Brace Paw", "Earth Shear"],
]
HEIGHTS = [[8,20,43], [12,35], [9,25,43], [24], [20,63], [15,31,43],
           [7,16,35], [6,18], [5,12,28], [11,39], [22], [10,35], [17], [10,24,47]]
WEIGHTS = [[35,140,720], [60,280], [30,150,390], [210], [380,4980],
           [110,410,1060], [18,65,210], [12,95], [9,48,160], [65,370],
           [460], [28,410], [190], [65,370,1650]]
# Reuse supported native icon/footprint/cry vocabulary, never another species' art.
TEMPLATES = ["Mudnip", "Kindlit", "Vanechick", "Dewbit", "Berrcub",
             "Yipup", "Midgebit", "Taplet", "Vanechick", "Rondive",
             "Augrub", "Hushiss", "Nibbin", "Nibbin"]
EGGS = [("BUG", "MINERAL"), ("GROUND", "FAIRY"), ("FLYING", "INDETERMINATE"),
        ("PLANT", "FAIRY"), ("GROUND", "MINERAL"), ("GROUND", "FAIRY"),
        ("FLYING", "INDETERMINATE"), ("FLYING", "GROUND"), ("FLYING", "FAIRY"),
        ("BUG", "DRAGON"), ("GROUND", "MONSTER"), ("DRAGON", "GROUND"),
        ("GROUND", "MINERAL"), ("GROUND", "MONSTER")]
MOVES = [
    "TACKLE WITHDRAW ROCK_THROW CHARM FAIRY_WIND ROLLOUT PROTECT ANCIENTPOWER LIGHT_SCREEN DAZZLING_GLEAM SAFEGUARD ROCK_SLIDE MOONBLAST",
    "POUND GROWL POWDER_SNOW QUICK_ATTACK CHARM ICY_WIND DOUBLE_KICK FAIRY_WIND AGILITY ICE_PUNCH PLAY_ROUGH DOUBLE_EDGE BLIZZARD",
    "PECK GROWL LICK FORESIGHT GUST NIGHT_SHADE HYPNOSIS CONFUSE_RAY WING_ATTACK SHADOW_BALL DREAM_EATER SKY_ATTACK FUTURE_SIGHT",
    "ABSORB LEER POISON_STING PURSUIT ACID FAINT_ATTACK POISON_GAS BITE GIGA_DRAIN SLUDGE_BOMB CRUNCH TOXIC SYNTHESIS",
    "SCRATCH DEFENSE_CURL ROCK_THROW MUD_SLAP MAGNITUDE ROLLOUT BITE PROTECT SLASH ROCK_SLIDE EARTHQUAKE CRUNCH REST",
    "TACKLE TAIL_WHIP SAND_ATTACK QUICK_ATTACK FAIRY_WIND BITE DIG FORESIGHT TAKE_DOWN PLAY_ROUGH EARTHQUAKE CRUNCH EXTREMESPEED",
    "LEECH_LIFE GROWL CONFUSION GUST DETECT PSYBEAM WING_ATTACK LIGHT_SCREEN AGILITY PSYCHIC_M RAZOR_WIND FUTURE_SIGHT REST",
    "PECK SAND_ATTACK GUST MUD_SLAP FURY_ATTACK QUICK_ATTACK DIG WING_ATTACK DETECT DRILL_PECK EARTHQUAKE AGILITY MIRROR_MOVE",
    "PECK GROWL POWDER_SNOW GUST MIST ICY_WIND QUICK_ATTACK WING_ATTACK HAZE ICE_BEAM WHIRLWIND BLIZZARD SKY_ATTACK",
    "TACKLE HARDEN POWDER_SNOW QUICK_ATTACK FURY_CUTTER ICY_WIND DRAGONBREATH WING_ATTACK SLASH ICE_BEAM AGILITY OUTRAGE BLIZZARD",
    "SCRATCH FORESIGHT MUD_SLAP CONFUSION DEFENSE_CURL DIG DETECT PSYBEAM MAGNITUDE SLASH PSYCHIC_M EARTHQUAKE FUTURE_SIGHT",
    "TACKLE TAIL_WHIP TWISTER CHARM QUICK_ATTACK DRAGON_RAGE FAIRY_WIND SLAM DRAGONBREATH AGILITY PLAY_ROUGH OUTRAGE MOONBLAST",
    "POUND TAIL_WHIP ROCK_THROW CONFUSION QUICK_ATTACK ROLLOUT DETECT PSYBEAM AGILITY ANCIENTPOWER BATON_PASS PSYCHIC_M ROCK_SLIDE",
    "SCRATCH LEER MUD_SLAP DEFENSE_CURL ROCK_SMASH DIG KARATE_CHOP VITAL_THROW SLASH SUBMISSION EARTHQUAKE CROSS_CHOP REVERSAL",
]
TM_MOVES = [
    "PROTECT REST TOXIC ATTRACT RETURN DOUBLE_TEAM ROLLOUT SANDSTORM",
    "PROTECT REST TOXIC ATTRACT RETURN DOUBLE_TEAM ICY_WIND BLIZZARD IRON_TAIL",
    "PROTECT REST TOXIC ATTRACT RETURN DOUBLE_TEAM SHADOW_BALL DREAM_EATER FLY",
    "PROTECT REST TOXIC ATTRACT RETURN DOUBLE_TEAM GIGA_DRAIN SLUDGE_BOMB THIEF",
    "PROTECT REST TOXIC ATTRACT RETURN DOUBLE_TEAM EARTHQUAKE DIG STRENGTH ROCK_SMASH",
    "PROTECT REST TOXIC ATTRACT RETURN DOUBLE_TEAM EARTHQUAKE DIG ROAR IRON_TAIL",
    "PROTECT REST TOXIC ATTRACT RETURN DOUBLE_TEAM PSYCHIC_M SWIFT FLY FLASH",
    "PROTECT REST TOXIC ATTRACT RETURN DOUBLE_TEAM DIG EARTHQUAKE FLY SWIFT",
    "PROTECT REST TOXIC ATTRACT RETURN DOUBLE_TEAM BLIZZARD ICY_WIND FLY SWIFT",
    "PROTECT REST TOXIC ATTRACT RETURN DOUBLE_TEAM BLIZZARD ICY_WIND DRAGONBREATH SWIFT",
    "PROTECT REST TOXIC ATTRACT RETURN DOUBLE_TEAM DIG EARTHQUAKE PSYCHIC_M FLASH",
    "PROTECT REST TOXIC ATTRACT RETURN DOUBLE_TEAM DRAGONBREATH IRON_TAIL SUNNY_DAY",
    "PROTECT REST TOXIC ATTRACT RETURN DOUBLE_TEAM ROLLOUT PSYCHIC_M SANDSTORM FLASH",
    "PROTECT REST TOXIC ATTRACT RETURN DOUBLE_TEAM DIG EARTHQUAKE ROCK_SMASH STRENGTH DYNAMICPUNCH",
]
DEX = [
    ("It pulls its head under its shell when rain stops.", "Small bugs rest beside its warm folded mantle."),
    ("Its shell lip curls into a dry shelter at dusk.", "It seals the gap with its mantle during hard rain."),
    ("Two deep alcoves in its shell stay warm all winter.", "It travels slowly so its sheltered guests can sleep."),
    ("Its wide hind feet brush loose snow across its tracks.", "It holds its ears flat when it hears ice break nearby."),
    ("Its broad feet sweep snow over the whole trail.", "It circles a lost traveler until a safe path appears."),
    ("Pale gaps in its feathers look like moonlit leaves.", "It stays so still that small birds perch beside it."),
    ("Two holes in its folded wings let no light pass.", "A passing breeze echoes inside its hollow plumage."),
    ("Its wings resemble dark gaps between branches at night.", "The forest seems to vanish behind its dark wings."),
    ("Bright berries draw prey toward its hidden thorns.", "It changes their scent to copy a nearby berry bush."),
    ("Three stone knobs grow above its strong shoulders.", "It sharpens blunt claws by scraping lakeside basalt."),
    ("Its back columns lock as it braces for a charge.", "Its shed plates form shelters on the rocky shore."),
    ("Its forked ankle marks glow after fresh snowfall.", "It retraces the tracks of its pack before going home."),
    ("Its pale leg bands split into small forked paths.", "It scouts a pass, then returns to guide the pups."),
    ("Its tracks glow until the last pup has passed.", "It finds lost pups before its pale markings fade."),
    ("It saves moments while sleeping through winter.", "It wakes and moves before its shadow can catch up."),
    ("Its pulse slows until a falling drop seems still.", "Stored motion lets it dart forward without a sound."),
    ("It holds air still around its thick winter fur.", "One wingbeat frees a whole night of stored motion."),
    ("Its broad toes scoop loose sand behind its body.", "It tests the bank with its beak before it digs."),
    ("It carries sand out in its deep wing folds.", "It dives beside a bluff face, then vanishes into it."),
    ("It fluffs its pale wing tips to hide from biting wind.", "Paired foot marks trail behind it in loose snow."),
    ("Curved feathers guide loose snow around its body.", "It turns into the wind to shelter a smaller flockmate."),
    ("Its long tail bends like a turning wind vane.", "Its flock follows its pale wings through squalls."),
    ("Six hooked feet grip the ice on its old pond.", "It folds its wing buds flat to slip below snow crusts."),
    ("Four sail wings catch wind over a frozen lake.", "It steers with its curved abdomen and six hooked feet."),
    ("Its nose reads tiny ripples in damp soil.", "It draws the shape of a tunnel with one broad paw."),
    ("Its bright tail draws foes away from its head.", "A lost tail grows back with bright new scales."),
    ("Its two tail tips twitch like small hidden heads.", "It sheds a tail tip to draw foes off a nest."),
    ("Its weighted tail swings before each long leap.", "A tremor in the ground can turn its tail midair."),
    ("Its flat front paws press loose soil into a roof.", "It braces with its tail when a burrow wall slips."),
    ("It braces one paw, then shoves with the other.", "Its shoulder ridges bend force toward the ground."),
    ("Its paws split hard soil along hidden seams.", "It shifts its weight to turn a charge aside."),
]


def page(text):
    """Find three complete word-bounded native lines; reject overflow."""
    words = text.split()
    choices = []
    for first in range(1, len(words) - 1):
        for second in range(first + 1, len(words)):
            lines = [" ".join(words[:first]), " ".join(words[first:second]),
                     " ".join(words[second:])]
            if all(0 < len(line) <= 18 for line in lines):
                choices.append((sum(len(line) ** 2 for line in lines), lines))
    if not choices:
        raise ValueError(f"Native dex page overflows: {text}")
    return " @ ".join(min(choices)[1])


def main():
    plan = json.loads((ROOT / "replacement-225-20261104.json").read_text())
    roster = json.loads((ROOT / "roster.json").read_text())
    existing = {p["id"]: p for p in json.loads((ROOT / "species-profiles.json").read_text())["species"]}
    names = {m["name"]: m["id"] for f in roster["families"] for m in f["members"]}
    profiles = []
    expected = sum(len(family["members"]) for family in plan["families"])
    assert len(plan["families"]) == len(STATS) == len(CLASSES) == len(TEMPLATES)
    assert len(DEX) == expected
    for family_index, family in enumerate(plan["families"]):
        template = existing[names[TEMPLATES[family_index]]]
        count = len(family["members"])
        for stage, member in enumerate(family["members"]):
            final = stage == count - 1
            profile = copy.deepcopy(template)
            profile.update(
                id=member["id"], classification=CLASSES[family_index][stage],
                height_inches=HEIGHTS[family_index][stage],
                weight_tenths_lb=WEIGHTS[family_index][stage],
                dex_pages=[page(t) for t in DEX[len(profiles)]],
                base_stats=dict(zip(STAT_KEYS, STATS[family_index][stage])),
                catch_rate=190 if stage == 0 and count > 1 else 45 if final else 90,
                base_exp=55 + stage * 65 + (70 if count == 1 else 0),
                gender_ratio=127, hatch_cycles=20, growth_rate="GROWTH_MEDIUM_FAST",
                egg_groups=["EGG_" + group for group in EGGS[family_index]],
                evolution=None if final else {"method": "LEVEL", "level":
                    (32 if family["family_id"] == "MN_F157" else 28) if count == 2 else 18 if stage == 0 else 36},
                held_items=[None, None], shiny_rgb5=shiny_palette(member["normal_rgb5"]),
                level_moves=list(map(list, zip(LEVELS, MOVES[family_index].split()))),
                tmhm_moves=TM_MOVES[family_index].split(), egg_moves=[],
            )
            profile["cry"]["pitch"] = -40 + stage * 24 + family_index * 3
            profile["cry"]["length"] = 120 + stage * 24
            assert len(profile["level_moves"]) == len(LEVELS)
            profiles.append(profile)
    assert len(profiles) == expected
    output = ROOT / "replacement-profiles-20261104.json"
    output.write_text(json.dumps({"schema_version": 1, "species": profiles}, indent=2) + "\n")
    print(f"Prepared {expected} complete pending profiles; active roster unchanged.")


def shiny_palette(normal):
    """Keep the authored light/shadow separation in a distinct vivid shiny hue."""
    result = []
    for rgb in normal[1:3]:
        hue, saturation, value = colorsys.rgb_to_hsv(*(v / 31 for v in rgb))
        color = colorsys.hsv_to_rgb((hue + 0.32) % 1, max(0.65, saturation), value)
        result.append([round(v * 31) for v in color])
    return result


if __name__ == "__main__":
    main()
