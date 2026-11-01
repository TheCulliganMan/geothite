#!/usr/bin/env python3
"""Preview the complete explicit Ely layout using unchanged native source art."""
import argparse
import json
import os
from pathlib import Path
import subprocess

from PIL import Image, ImageChops

from render_archive import retain_render_file, save_render
from render_reference import SourceArt, layout_blocks, load_pack
from render_stages import render_stage
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "planning/minnesota/renders/ely-league"
VIEWS = [("registration", "MnEly", 25, 15), ("clinic", "MnEly", 12, 23),
         ("supply-loop", "MnEly", 33, 27), ("north-arrival", "MnEly", 24, 3),
         ("clinic-room", "MnElyPokecenter1F", 3, 5), ("mart-room", "MnElyMart", 3, 3),
         ("lobby", "MnLeagueLobby", 1, 4), ("fern", "MnLeagueRoom1", 4, 10),
         ("flint", "MnLeagueRoom2", 4, 10), ("veil", "MnLeagueRoom3", 4, 10),
         ("reed", "MnLeagueRoom4", 4, 10), ("casey", "MnLeagueChampion", 4, 10),
         ("hall", "MnLeagueHallOfFame", 4, 10)]


def capture(pack, selected):
    env = dict(os.environ, CRYSTAL_RENDER_TEST_HEADLESS="1", CRYSTAL_RENDER_TEST_CAPTURE_TARGET="image",
               CRYSTAL_RENDER_TEST_WINDOW_SIZE="1280x800")
    for key, name, x, y in VIEWS:
        if selected and key not in selected:
            continue
        output = OUT / f"ely-{key}-native.png"
        retain_render_file(output)
        with (ROOT / ".local" / f"ely-capture-{key}.log").open("w") as log:
            subprocess.run([str(ROOT / "target/northstar-story/wasm-dev/examples/render_at_location"),
                            "--pack", str(pack.resolve()), "--map", name, "--x", str(x), "--y", str(y),
                            "--view", "2d", "--hour", "12", "--screenshot", str(output)],
                           env=env, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, check=True)
        retain_render_file(output)
        print("Native headless capture:", key, flush=True)


def published(pack, art):
    maps = pack["data"]["maps"]
    names = ["MnEly", "MnElyPokecenter1F", "MnElyMart", "MnLeagueLobby"] + [f"MnLeagueRoom{i}" for i in range(1,5)] + ["MnLeagueChampion", "MnLeagueHallOfFame"]
    scenes = {}
    for name in names + [f"MnR13C00R{i:02}" for i in range(3)]:
        m = maps[name]
        a = m["attributes"]
        group = "indoor" if name == "MnLeagueHallOfFame" else "day"
        scene = render_stage(art.render(a["tileset_name"], a["width"], a["height"], m["blocks"], group),
                             "convert", OUT, name, "RGBA")
        for actor in m["objects"]:
            facing = {"UP":"north", "LEFT":"west", "RIGHT":"east"}.get(actor["spritemovedata"].rsplit("_",1)[-1],"south")
            scene.alpha_composite(oriented_sprite(pack,actor["sprite"],facing),(actor["x"]*16,actor["y"]*16))
        save_render(scene, OUT / f"{name}-published-whole.png")
        scenes[name] = scene
    atlas = Image.new("RGBA", (1024,2624), "#17232c")
    for row in range(3): atlas.alpha_composite(scenes[f"MnR13C00R{row:02}"],(0,row*768))
    atlas.alpha_composite(scenes["MnEly"], (160,2048))
    save_render(atlas, OUT / "r13-ely-connected-source.png")
    # Same 16px position, south-facing trainer and noon source art as capture.
    for key,name,x,y in VIEWS[:3]:
        scene = scenes[name]
        canvas = Image.new("RGBA", (scene.width+512,scene.height+512))
        border = art.blocks("kanto")[maps[name]["attributes"]["border_block"]]
        for by in range(0,canvas.height,32):
            for bx in range(0,canvas.width,32): canvas.paste(border,(bx,by))
        canvas.alpha_composite(scene,(256,256))
        px,py = 256+x*16,256+y*16
        canvas.alpha_composite(oriented_sprite(pack,"SPRITE_CHRIS","south"),(px,py))
        camera=render_stage(canvas,"crop",OUT,key+"-native-reference",(px-256,py-192,px+256,py+192))
        camera=render_stage(camera,"resize",OUT,key+"-native-reference",(1536,1152),Image.Resampling.NEAREST)
        camera=render_stage(camera,"crop",OUT,key+"-native-reference",(128,176,1408,976))
        save_render(camera,OUT/f"ely-{key}-native-reference.png")
        actual=OUT/f"ely-{key}-native.png"
        if actual.exists():
            delta=ImageChops.difference(Image.open(actual).convert("RGB"),camera.convert("RGB"))
            save_render(delta,OUT/f"ely-{key}-native-difference.png")
            print(key,"source/native differing pixels:",sum(pixel!=(0,0,0)for pixel in delta.getdata()))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-north-shore-story.crystalpack")
    parser.add_argument("--published", action="store_true")
    parser.add_argument("--capture", action="store_true")
    parser.add_argument("--capture-view", action="append", choices=[v[0] for v in VIEWS])
    args = parser.parse_args()
    if args.capture:
        assert args.published, "--capture requires a published pack"
        capture(args.pack, args.capture_view)
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    town = json.loads((ROOT / "planning/minnesota/towns/MnEly.json").read_text())
    layout = json.loads((ROOT / "planning/minnesota/towns/MnEly.layout.json").read_text())
    blocks = layout_blocks(layout)
    if args.published:
        assert pack["data"]["maps"]["MnEly"]["blocks"] == blocks
    w, h = layout["dimensions_metatiles"]
    terrain = art.render("kanto", w, h, blocks)
    save_render(terrain, OUT / "ely-whole-terrain.png")
    scene = render_stage(terrain, "convert", OUT, "ely-scene", "RGBA")
    for actor in town["residents"]:
        sprite = oriented_sprite(pack, actor["sprite"], actor["facing"])
        x, y = actor["tile"]
        scene.alpha_composite(sprite, (x * 16, y * 16))
    save_render(scene, OUT / "ely-whole-source.png")
    for name, x, y in [("registration",25,15),("clinic",12,23),("supply-loop",33,27),("north-arrival",24,3)]:
        camera = render_stage(scene, "crop", OUT, name, (x*16-128,y*16-96,x*16+128,y*16+96))
        camera = render_stage(camera, "resize", OUT, name, (1024,768))
        save_render(camera, OUT / f"ely-{name}-source.png")
    print(f"Ely source: {w}x{h} metatiles; whole town and four camera views retained in {OUT}")
    if args.published:
        published(pack,art)


if __name__ == "__main__":
    main()
