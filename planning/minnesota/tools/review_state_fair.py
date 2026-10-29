#!/usr/bin/env python3
"""Fast native-tile fair layout review with separate original NPC sprites."""
import argparse
import hashlib
import json
from collections import deque
from pathlib import Path

from PIL import Image

from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]


def resolve_icon_sprites(pack, tokens):
    """Mirror native species-icon fallback in memory; never export another asset."""
    files = dict(pack["runtime_files"])
    aliases = []
    for token in tokens:
        name = token.removeprefix("SPRITE_").lower()
        target = f"gfx/sprites/{name}.png"
        if target in files or f"gfx/pokemon/{name}/overworld.2bpp" in files:
            continue
        icon = pack["data"]["menu_icons"].get(name.upper(), "").lower().removeprefix("icon_")
        candidates = [f"gfx/icons/{name}.png", f"gfx/icons/{icon}.png"]
        source = next(path for path in candidates if path in files)
        files[target] = files[source]
        aliases.append(dict(sprite=token, native_source=source,
                            sha256=hashlib.sha256(bytes(files[source])).hexdigest()))
    return dict(pack, runtime_files=files), aliases


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--layout", type=Path, default=ROOT / "planning/minnesota/zones/state-fairgrounds.layout.json")
    parser.add_argument("--dossier", type=Path, default=ROOT / "planning/minnesota/zones/state-fairgrounds.json")
    parser.add_argument("--native-barn-capture", type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    for source in (args.layout, args.dossier):
        payload = source.read_bytes()
        digest = hashlib.sha256(payload).hexdigest()
        snapshot = args.output / f"{source.stem}-{digest}.json"
        if snapshot.exists():
            assert snapshot.read_bytes() == payload, "source snapshot drift"
        else:
            snapshot.write_bytes(payload)
    layout = json.loads(args.layout.read_text())
    zone = json.loads(args.dossier.read_text())
    design = zone["native_map"]
    roster = json.loads((ROOT / "planning/minnesota/pokemon/roster.json").read_text())
    regional = {member["name"].upper() for family in roster["families"]
                for member in family["members"]}
    for trainer in zone["native_trainers"]:
        for member in trainer["party"]:
            assert member["species"] in regional, (trainer["id"], "nonregional party species")
    exhibits = [a for a in design["residents"] if a["id"].startswith("MnFairExhibit")
                and not a["id"].startswith("MnFairExhibitJudge")]
    for actor in exhibits:
        assert actor["sprite"].removeprefix("SPRITE_") in regional, actor["id"]
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    width, height = layout["dimensions_metatiles"]
    blocks = layout_blocks(layout)
    collision = dict(pack["data"]["tilesets"][layout["tileset"]]["collision"])
    for recipe in layout["derived_blocks"]:
        collision[recipe["id"]] = recipe["collision"]

    def kind(tile):
        x, y = tile
        return collision[f"{blocks[y // 2 * width + x // 2]:02x}"][y % 2 * 2 + x % 2]

    residents = design["residents"] + zone["native_trainers"]
    crowd = zone["crowd_contract"]
    assert len(residents) == crowd["total_outdoor_npcs"] <= 255
    assert len(zone["native_trainers"]) == crowd["battle_trainers"] == 35
    exhibits = [a for a in residents if a["id"].startswith("MnFairExhibit")
                and not a["id"].startswith("MnFairExhibitJudge")]
    judges = [a for a in residents if a["id"].startswith("MnFairExhibitJudge")]
    assert len(residents) - len(exhibits) >= 200, "fair needs 200+ people and trainers"
    assert len(exhibits) == len(judges) == crowd["exhibition_pokemon"]
    for exhibit, judge in zip(exhibits, judges):
        assert judge["tile"] == [exhibit["tile"][0] + 1, exhibit["tile"][1]]
    bodies = {tuple(actor["tile"]) for actor in residents}
    assert len(bodies) == len(residents), "overlapping NPCs"
    assert all(actor.get("lines") for actor in design["residents"]), "every resident needs text"
    moving_tiles = set()
    for movement in zone["native_patrols"] + zone.get("native_ambient", []):
        actor = next(a for a in residents if a["id"] == movement["id"])
        if movement.get("behavior") == "rotate":
            assert movement["size_core"] == [0, 0]
            continue
        x, y = actor["tile"]
        w, h = movement["size_core"]
        route = {(xx, yy) for xx in range(x, x + w + 1) for yy in range(y, y + h + 1)
                 if xx in (x, x + w) or yy in (y, y + h)}
        assert all(kind(tile) == "FLOOR" for tile in route), movement["id"]
        assert not route & (bodies - {(x, y)}), movement["id"]
        assert not route & moving_tiles, movement["id"]
        moving_tiles.update(route)
    start = tuple(zone.get("expansion_contract", {}).get("entry_core", [16, 44]))
    seen = {start}
    queue = deque([start])
    while queue:
        x, y = queue.popleft()
        for q in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
            if (q in seen or q in bodies or not (0 <= q[0] < width * 2 and 0 <= q[1] < height * 2)
                    or kind(q) not in {"FLOOR", "TALL_GRASS", "DOOR"}):
                continue
            seen.add(q)
            if kind(q) != "DOOR":
                queue.append(q)
    for actor in residents:
        at = tuple(actor["tile"])
        assert kind(at) == "FLOOR", (actor["id"], "NPC is not on staff FLOOR", at, kind(at))
        if actor.get("counter_tile"):
            assert kind(actor["counter_tile"]) == "COUNTER", actor["id"]
            assert kind((at[0], at[1] + 2)) == "FLOOR", (actor["id"], "customer tile must stay clear")
            assert tuple(actor["approach"]) in seen, (actor["id"], "customer approach disconnected")
        else:
            x, y = at
            assert any(q in seen for q in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1))), actor["id"]
    for door in design["doors"]:
        assert tuple(door["tile"]) in seen and kind(door["tile"]) == "DOOR", door
        assert tuple(door["return_tile"]) in seen and kind(door["return_tile"]) == "FLOOR", door
    for tile in seen:
        if tile[0] in (0, width * 2 - 1) or tile[1] in (0, height * 2 - 1):
            allowed = {(95,64), (95,65)} if design['ports'][0]['side'] == 'E' else {(16, height * 2 - 1), (17, height * 2 - 1)}
            assert tile in allowed, ("unbound fair edge", tile)
    image = art.render(layout["tileset"], width, height, blocks, derived=layout["derived_blocks"]).convert("RGBA")
    for actor in residents:
        sprite = oriented_sprite(pack, actor["sprite"], actor["facing"])
        image.alpha_composite(sprite, (actor["tile"][0] * 16, actor["tile"][1] * 16))
    save_render(image, args.output / "whole.native.png")
    save_render(image.resize((image.width * 2, image.height * 2), Image.Resampling.NEAREST), args.output / "whole.2x.png")
    views = zone.get("expansion_contract", {}).get("review_views", [])
    if not views:
        views = [dict(id=label, core=at) for label, at in [("cookie-counter", (47, 26)), ("milk-counter", (53, 32)), ("midway", (13, 29)), ("entry", (17, 44)), ("north-end", (47, 16))]]
    for view in views:
        label, at = view["id"], view["core"]
        assert tuple(at) not in bodies and kind(at) == "FLOOR", (label, "review position must be free floor")
        x, y = at
        crop = image.crop((x * 16 - 72, y * 16 - 64, x * 16 + 88, y * 16 + 80))
        save_render(crop, args.output / f"{label}.160x144.png")
        save_render(crop.resize((640, 576), Image.Resampling.NEAREST), args.output / f"{label}.4x.png")
    if args.native_barn_capture:
        from review_city_rooms_native import compare
        module = pack["data"]["maps"]["MnFairBarn"]
        resolved, aliases = resolve_icon_sprites(pack, [o["sprite"] for o in module["objects"]])
        folder = args.output / "native-barn-comparison"
        folder.mkdir(parents=True, exist_ok=True)
        result = compare(resolved, SourceArt(resolved), module,
                         Image.open(args.native_barn_capture).convert("RGB"), (3, 6), folder)
        result.update(native_icon_sources=aliases,
                      capture_sha256=hashlib.sha256(args.native_barn_capture.read_bytes()).hexdigest(),
                      source_pack_unchanged=True)
        (folder / "proof.json").write_text(json.dumps(result, indent=2) + "\n")
        assert result["pixel_verified"], result
    print(f"Native source review: {width}x{height} metatiles, {len(residents)} separate NPCs, {len(seen)} reachable core tiles; all customer approaches and doors connected.")


if __name__ == "__main__":
    main()
