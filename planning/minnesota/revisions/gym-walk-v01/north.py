#!/usr/bin/env python3
"""Inspect Brainerd beach pixels, its immediate walking approach, and Bemidji's two Cut clearings."""
import argparse
from collections import Counter, deque
from itertools import product
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
MN = HERE.parents[1]
sys.path.insert(0, str(MN / "tools"))
from audit_story_flow import instructions, native_pages, read_branch, require
from render_reference import SourceArt, load_pack

GYMS = runpy.run_path(str(HERE.parent / "badge-trails-v01/check-gyms.py"))
LANES = GYMS["lanes"]
BEACH = "MnBrainerdGym"
GARDEN = "MnBemidjiGym"
RIPPLES = {0x22, 0x2D, 0x5A, 0x52, 0x64, 0x6C, 0x73, 0x79}
TREES = [(3, 12), (7, 6)]
HEDGES = ["EVENT_MN_GYM_7_CUT_SOUTH_HEDGE_PRUNED", "EVENT_MN_GYM_7_CUT_NORTH_HEDGE_PRUNED"]
WINS = [f"EVENT_MN_GYM_7_TRAINER_{n}_DEFEATED" for n in (1, 2, 3)]
LEADER = "EVENT_MN_GYM_7_DEFEATED"


def geometry(data, room, flags):
    attrs = room["attributes"]
    blocks = room["blocks"].copy()
    LANES["run"](room, room["id"] + "RefreshGates", set(flags), blocks)
    bank = data["tilesets"][attrs["tileset_name"]]["collision"]
    w, h = attrs["width"], attrs["height"]
    return {(x,y): bank[f"{blocks[(y//2)*w+x//2]:02x}"][2*(y%2) + x%2]
            for y in range(2*h) for x in range(2*w)}


def distances(room, terrain, start):
    blocked = {LANES["coords"](o) for o in room["objects"]}
    seen, queue = {start:0}, deque([start])
    while queue:
        pos = queue.popleft()
        for dest in LANES["adjacent"](pos) - blocked:
            if dest not in seen and terrain.get(dest) == "FLOOR":
                seen[dest] = seen[pos]+1
                queue.append(dest)
    return seen


def garden(parent, child):
    room = child["maps"][GARDEN]
    old = parent["maps"][GARDEN]
    for key in ["objects", "events", "blocks", "trainer_scripts", "attributes", "map_script_section_commands"]:
        require(room[key] == old[key], f"changed garden {key}")
    require(instructions(room, GARDEN+"GuideRepeat") == instructions(old,GARDEN+"GuideRepeat"),
            "Champion circuit or pre-Champion branch changed")
    guide = instructions(room, GARDEN+"Guide")
    require(guide == [("writetext","MnGardenCutText") if row == ("writetext",GARDEN+"GuideCutGiftText") else row
                      for row in instructions(old,GARDEN+"Guide")], "gift, full pocket, or retry changed")
    canonical = json.loads((MN/"story.json").read_text())["dialogue_overhaul"]["texts"][GARDEN]
    labels = {"MnGarden"+suffix+"Text" for suffix in ("Cut", "Fork", "West", "North", "Leader", "Return")}
    for label in labels:
        pages = native_pages(room["scripts"][label])
        require(pages == canonical[label]
                and pages == native_pages(room["script_text_bodies"][label]["commands"])
                and pages == native_pages(child["map_scripts"][label]), f"installed garden pages {label}")
        require(all(1 <= len(pg) <= 2 and all(0 < len(line) <= 18 for line in pg) for pg in pages), label)
    start = GYMS["public_door"](child,room)
    require(start == (4,17),"garden door")
    for flags in product([False,True], repeat=4):
        state = {f for f,v in zip([*WINS,LEADER], flags) if v}
        a,b,_,won = flags
        expected = "Return" if won else "Leader" if a and b else "North" if a else "West" if b else "Fork"
        actions, text, after = read_branch(room, GARDEN+"GuideRepeat", state)
        require(text == [f"MnGarden{expected}Text"] and state == after
                and ("startbattle",) not in actions, f"wrong garden advice: {state} {text}")
    actors = [o for o in room["objects"] if o["object_type"]=="OBJECTTYPE_TRAINER"]
    for south,north,link in product([False,True], repeat=3):
        flags = {f for f,v in zip([*HEDGES,WINS[2]],(south,north,link)) if v}
        terrain = geometry(child, room, flags)
        reached = distances(room, terrain, start)
        bodies = {LANES["coords"](o) for o in room["objects"]}
        spots = [LANES["rays"](actor,LANES["coords"](actor),bodies,lambda p:terrain.get(p,"WALL"))
                 for actor in actors]
        available = [bool(set(s) & reached.keys()) for s in spots]
        require(available == [south,north,south], "cut trees must protect the two respective clearings")
        require(all((terrain[tree] == "FLOOR") == cut for tree,cut in zip(TREES,(south,north))),
                "garden native Cut restoration")
        if not any((south,north,link)):
            require((reached.get((3,13)),reached.get((7,7))) == (5,17), "left/right initial approaches")
        require(terrain[6,12] == "WALL" and geometry(child,room,flags|{LEADER})[6,12] == "FLOOR",
                "center gate opens only with leader victory")
    print("Bemidji: 16 guide states, 8 independent hedge/shortcut combinations, either keeper first; two initial Cut approaches, 5 and 17 steps.")


def pixels(frame):
    return tuple((frame[2*y]>>(7-x)&1) | ((frame[2*y+1]>>(7-x)&1)<<1) for y in range(8) for x in range(8))


def beach(parent, child):
    a,b=parent["data"],child["data"]
    old,room=a["maps"][BEACH],b["maps"][BEACH]
    for key in ("objects","events","trainer_scripts","attributes","map_script_section_commands"):
        require(room[key]==old[key], f"Brainerd changed {key}")
    guide = BEACH + "GuideText"
    changed = {label for label in room["scripts"].keys() | old["scripts"].keys()
               if room["scripts"].get(label) != old["scripts"].get(label)}
    require(changed == {guide}, "unexpected Brainerd script change")
    beach_pages = native_pages(room["scripts"][guide])
    require(beach_pages == native_pages(room["script_text_bodies"][guide]["commands"])
            and beach_pages == native_pages(b["map_scripts"][guide])
            and "dark pools" in " ".join(line for page in beach_pages for line in page),
            "missing Brainerd walk/canoe directions")
    spec=json.loads((MN/"gyms/brainerd-lakeside-v01/shallows.json").read_text())
    require(old["blocks"] in (spec["prior"],spec["room"]), "incorrect beach parent")
    start=GYMS["public_door"](b,room)
    surf=geometry(b,room,set())
    reached=distances(room,surf,start)
    pools = json.loads((HERE / "brainerd-pools.json").read_text())["canoe_water"]
    want_water = {tuple(p) for p in pools}
    require(len(want_water) == 19 and {p for p, role in surf.items() if role == "WATER"} == want_water,
            "Brainerd optional deep pools")
    require(start==(8,12) and reached[(8,4)]==8 and all((8,y) in reached for y in range(4,13)), "straight wade to Sivren")
    bodies={LANES["coords"](o) for o in room["objects"]}
    require(bodies=={(8,3),(4,8),(13,5),(7,11)} and all(surf[p]=="FLOOR" for p in bodies),"sandbar footing")
    fi=child["runtime_files"]
    clips={int(k):v for k,v in json.loads(bytes(fi["data/tilesets/mn_gym_6_animations.json"])).items()}
    require(RIPPLES==clips.keys(),"all eight isolated native ripples")
    tiles=bytes(fi["data/tilesets/mn_gym_6_metatiles.bin"])
    layout=[tiles[k*16:k*16+16] for k in room["blocks"]]
    require(all(len(block)==16 for block in layout), "unbound beach tile")
    placed=Counter(tile for block in layout for tile in block)
    sheet=bytes(fi["gfx/tilesets/mn_gym_6.2bpp"])
    palette=b["tilesets"]["mn_gym_6"]["palette_map"]
    for glyph,clip in clips.items():
        frames=[bytes(f) for f in clip["frames"]]
        require(len(set(frames))==len(frames)==4 and clip["frame_ticks"]==22 and placed[glyph]>0, f"frozen water {glyph:x}")
        require(frames[0]==sheet[glyph*16:glyph*16+16] and palette[glyph]==3,f"ripple frame zero/palette {glyph:x}")
        colors=Counter(pixels(frames[0]))
        require(all(Counter(pixels(f))==colors for f in frames),"ripple changed indexed depth/brightness")
    art=SourceArt(child).render("mn_gym_6",9,7,room["blocks"],"day")
    sand=tuple((v<<3)|(v>>2) for v in [29,26,18])
    for x,y in bodies:
        feet = art.crop((16*x,16*y+8,16*x+16,16*y+16))
        require(sum(count for count,color in feet.getcolors() if color == sand) > 64,"visible dry footing")
    require(sum(placed[t] for t in RIPPLES)==39,"unexpected ripple locations")
    for actor in room["objects"]:
        if actor["object_type"]=="OBJECTTYPE_TRAINER":
            sight = set(LANES["rays"](actor,LANES["coords"](actor),bodies,lambda p:surf.get(p,"WALL")))
            require(sight and sight <= reached.keys(),
                    f"lost beach trainer approach {actor['script']}")
    print("Brainerd: 39 placed ripples, 8 four-frame water glyphs, 22 ticks per frame; all four actors on visible sand, 8 steps to the leader.")


if __name__=="__main__":
    cli=argparse.ArgumentParser(description=__doc__)
    cli.add_argument("parent",type=Path)
    cli.add_argument("child",type=Path)
    args=cli.parse_args()
    old,new=map(load_pack,(args.parent,args.child))
    beach(old,new)
    garden(old["data"],new["data"])
