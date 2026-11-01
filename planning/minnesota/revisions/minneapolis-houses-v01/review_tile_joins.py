#!/usr/bin/env python3
"""Repair native façade grammar from existing glyphs, without pack writes."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
import time

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "planning/minnesota/tools"))
from render_archive import save_render
from render_reference import SourceArt, load_pack
from review_north_shore import oriented_sprite


def render(pack, folder):
    started = time.perf_counter()
    folder.mkdir(parents=True, exist_ok=False)
    source = Path(__file__).resolve().parent
    inventory = json.loads((source / "exterior-inventory-v01.json").read_bytes())
    module = pack["data"]["maps"]["MnMinneapolis"]
    ts = module["attributes"]["tileset_name"]
    art = SourceArt(pack)
    native = art.blocks(ts)
    definitions = pack["data"]["tilesets"][ts]
    raw = bytes(pack["runtime_files"][f"data/tilesets/{ts}_metatiles.bin"])
    old_grid = list(module["blocks"])
    new_grid = list(old_grid)
    recipes = []
    faults = []
    changes = []
    buildings = []

    def glyph(block, x, y):
        return raw[block * 16 + y * 4 + x]

    # Existing logical glyphs, not recolored copies. These are already native
    # modern wall/cap/window/foundation and Kanto right-end glyphs in the alias.
    brick = glyph(0x10, 3, 0)
    window = glyph(0x10, 1, 1)
    left_wall = glyph(0x10, 0, 0)
    right_wall = glyph(0x11, 3, 0)
    foot = glyph(0x10, 1, 3)
    left_foot = glyph(0x10, 0, 3)
    right_foot = glyph(0x11, 3, 3)
    door_glyphs = {glyph(0x1a, x, y) for y in [2, 3] for x in [2, 3]}
    blue_cap = glyph(0x20, 0, 0)
    blue_foot_cap = glyph(0x20, 0, 3)
    basic_body = {brick, window, left_wall, right_wall, foot, left_foot,
                  right_foot, blue_cap, blue_foot_cap,
                  *[glyph(0x20, 1, y) for y in range(4)]}

    def note(record, gx, gy, role, old, new):
        x0, y0, _, _ = record["footprint_metatiles"]
        px, py = x0 * 32 + gx * 8, y0 * 32 + gy * 8
        faults.append(dict(building=record["id"], role=role,
                           source_pixel=[px, py], world_glyph=[px//8, py//8],
                           source_metatile=[px//32, py//32],
                           before_logical_glyph=f"{old:02x}", after_logical_glyph=f"{new:02x}"))

    for record in inventory:
        x0, y0, x1, y1 = record["footprint_metatiles"]
        width, height = (x1 - x0) * 4, (y1 - y0) * 4
        before = [[0] * width for _ in range(height)]
        for gy in range(height):
            for gx in range(width):
                block = old_grid[(y0 + gy//4) * 40 + x0 + gx//4]
                before[gy][gx] = glyph(block, gx%4, gy%4)
        after = [list(row) for row in before]
        first = old_grid[y0 * 40 + x0]
        style = "masonry" if first == 0x80 else "blue" if first == 0x18 else "special"
        if style == "blue":
            # A roof has one left end, a repeating central surface and one
            # right end. Repeating native block18 repeated its left endcap.
            for gy in range(4):
                for gx in range(width):
                    expected = (glyph(0x18, 0, gy) if gx == 0 else
                                glyph(0x19, 3, gy) if gx == width-1 else
                                glyph(0x18, 1, gy))
                    if before[gy][gx] != expected:
                        note(record,gx,gy,"roof endcap inside roof",before[gy][gx],expected)
                    after[gy][gx] = expected
            # Native20 is a ground-floor block: DA/DD are its left edge and
            # DF is a ground foundation. Upper floors instead use the actual
            # no-foundation27/23/28 native wall/window grammar.
            for my in range(1, y1-y0-1):
                ids = old_grid[(y0+my)*40+x0:(y0+my)*40+x1]
                assert all(b == 0x20 for b in ids), (record["id"], ids)
                for sy in range(4):
                    gy = my*4+sy
                    for gx in range(width):
                        expected = (left_wall if gx == 0 else right_wall if gx == width-1
                                    else window if sy == 2 else brick)
                        if before[gy][gx] != expected:
                            role = "upper-floor ground foundation" if sy == 3 else "upper-floor interior edge or wall grammar"
                            note(record,gx,gy,role,before[gy][gx],expected)
                        after[gy][gx] = expected
        # All blue-brick ground fronts, including service logos and the radio
        # office, keep their native windows/sign/door glyphs. Only false side
        # caps inside the wall and missing true outside caps are repaired.
        if style == "blue" or record["id"] in {"Pokecenter1F","Mart","DispatchOffice"}:
            for sy in range(4):
                gy = height-4+sy
                for gx in range(width):
                    value = after[gy][gx]
                    expected = value
                    if 0 < gx < width-1:
                        if value in {left_wall, right_wall}:
                            expected = brick
                        elif value in {left_foot, right_foot}:
                            expected = foot
                        elif value == blue_cap:
                            expected = glyph(0x20, 1, 1) if sy == 1 else brick
                        elif value == blue_foot_cap:
                            expected = foot
                        elif value in {glyph(0x20,1,0),glyph(0x20,1,2)}:
                            expected = brick
                        elif value == glyph(0x20,1,3):
                            expected = foot
                    elif value in basic_body and value not in door_glyphs:
                        expected = ((left_wall if gx == 0 else right_wall) if sy < 3
                                    else (left_foot if gx == 0 else right_foot))
                    if expected != value:
                        note(record,gx,gy,"ground interior side cap / missing exterior cap",value,expected)
                        after[gy][gx] = expected
        if style == "masonry":
            # Interior door3a does not have a right endcap. It was used as an
            # outside end on every two-wide masonry house.
            for sy in range(4):
                gy = height-4+sy
                expected = glyph(0x88, 3, sy)
                if after[gy][-1] != expected:
                    note(record,width-1,gy,"missing masonry right outside cap",after[gy][-1],expected)
                    after[gy][-1] = expected
        if style == "blue":
            for gy in range(4, height-4):
                assert after[gy][0] == left_wall and after[gy][-1] == right_wall,record["id"]
                assert all(g not in {left_wall,right_wall,left_foot,right_foot,foot,blue_cap,blue_foot_cap}
                           for g in after[gy][1:-1]),record["id"]
            for gy in range(height-4,height):
                assert all(g not in {left_wall,right_wall,left_foot,right_foot,blue_cap,blue_foot_cap}
                           for g in after[gy][1:-1]),(record["id"],gy)
        if style == "masonry":
            assert all(after[height-4+sy][-1] == glyph(0x88,3,sy) for sy in range(4)),record["id"]
        affected = 0
        for my in range(y1-y0):
            for mx in range(x1-x0):
                index = (y0+my)*40+x0+mx
                original = old_grid[index]
                values = [after[my*4+dy][mx*4+dx] for dy in range(4) for dx in range(4)]
                if values == list(raw[original*16:original*16+16]):
                    continue
                collision = definitions["collision"][f"{original:02x}"]
                existing = next((b for b in recipes if b["glyphs"] == values and b["collision"] == collision),None)
                if existing is None:
                    existing = dict(id=f"{157+len(recipes):02x}", glyphs=values,
                                    collision=collision, first_building=record["id"])
                    recipes.append(existing)
                new_grid[index] = int(existing["id"],16)
                changes.append(dict(metatile=[x0+mx,y0+my],before=f"{original:02x}",
                                    after=existing["id"],building=record["id"],collision=collision))
                affected += 1
        buildings.append(dict(id=record["id"],style=style,changed_metatiles=affected,
                              footprint_metatiles=record["footprint_metatiles"],door_core=record["door_core"]))

    tile_cache = {}
    for block in range(157):
        for cell in range(16):
            tile_cache.setdefault(glyph(block,cell%4,cell//4),native[block].crop(
                (cell%4*8,cell//4*8,cell%4*8+8,cell//4*8+8)))
    new_blocks = {}
    for recipe in recipes:
        image = Image.new("RGB",(32,32))
        for i,g in enumerate(recipe["glyphs"]):
            image.paste(tile_cache[g],(i%4*8,i//4*8))
        new_blocks[int(recipe["id"],16)] = image

    def scene(grid):
        image=Image.new("RGB",(1280,1024))
        for i,b in enumerate(grid): image.paste(new_blocks[b] if b in new_blocks else native[b],(i%40*32,i//40*32))
        return image

    old_image,new_image=scene(old_grid),scene(new_grid)
    save_render(old_image,folder/"whole-city-before.png")
    save_render(new_image,folder/"whole-city-candidate.png")
    annotated=old_image.copy();draw=ImageDraw.Draw(annotated)
    for fault in faults:
        x,y=fault["source_pixel"];color="#ff9000" if "foundation" in fault["role"] else "#ff245b"
        draw.rectangle((x,y,x+7,y+7),outline=color)
    save_render(annotated,folder/"whole-city-before-faults-annotated.png")
    focuses=[("BoatHouse",[4,30]),("SupplyCoop",[7,10]),("NortheastHome",[74,10])]
    camera_sheet=Image.new("RGB",(640,len(focuses)*320),"#14212b");camera_draw=ImageDraw.Draw(camera_sheet)
    for row,(name,(x,y)) in enumerate(focuses):
        record=next(r for r in inventory if r["id"]==name);x0,y0,x1,y1=record["footprint_metatiles"]
        box=(x0*32,y0*32,x1*32,y1*32)
        pair=Image.new("RGB",((x1-x0)*32*8*2+32,(y1-y0)*32*8+32),"#14212b");pd=ImageDraw.Draw(pair)
        for column,(tag,image) in enumerate([("before",old_image),("candidate",new_image)]):
            crop=image.crop(box);save_render(crop,folder/f"{name}-{tag}-facade-native.png")
            pair.paste(crop.resize((crop.width*8,crop.height*8),Image.Resampling.NEAREST),(column*(crop.width*8+16),32));pd.text((column*(crop.width*8+16)+4,8),tag,fill="white")
            player=image.convert("RGBA");player.alpha_composite(oriented_sprite(pack,"SPRITE_CHRIS","north"),(x*16,y*16))
            left=min(max(x*16-80,0),1120);top=min(max(y*16-72,0),880)
            camera=player.crop((left,top,left+160,top+144));save_render(camera,folder/f"{name}-{tag}-native160.png")
            camera_sheet.paste(camera.resize((320,288),Image.Resampling.NEAREST),(column*320,row*320+26));camera_draw.text((column*320+8,row*320+8),f"{name}: {tag}",fill="white")
        save_render(pair,folder/f"{name}-joined-facade-before-candidate-8x.png")
    save_render(camera_sheet,folder/"player-cameras-before-candidate.png")
    # Complete forty-front visual audit in consistent native-pixel scale.
    audit=Image.new("RGB",(5*512,8*340),"#14212b");audit_draw=ImageDraw.Draw(audit)
    for i,record in enumerate(inventory):
        x,y=i%5*512,i//5*340
        x0,y0,x1,y1=record["footprint_metatiles"]
        box=(max(0,x0*32-16),max(0,y0*32-16),min(1280,x1*32+16),min(1024,y1*32+32))
        for column,(tag,image) in enumerate([("before",old_image),("candidate",new_image)]):
            crop=image.crop(box)
            audit.paste(crop,(x+column*256+(256-crop.width)//2,y+34))
            audit_draw.text((x+column*256+8,y+7),f"{i+1:02d} {record['id']}: {tag}",fill="white")
            save_render(crop,folder/f"audit-{i+1:02d}-{record['id']}-{tag}.png")
    save_render(audit,folder/"all-40-joined-fronts-before-candidate-native.png")
    assert len(recipes)+157<=256
    for patch in changes:
        assert definitions["collision"][patch["before"]] == next(r["collision"] for r in recipes if r["id"]==patch["after"])
    result=dict(schema_version=1,status="CORRECTIVE_TILE_JOIN_SOURCE_NOT_NATIVE_PUBLICATION",map="MnMinneapolis",tileset=ts,
                parent_pack_sha256="57eefb675b9cb32ac66a01819f56961392e51678a6f9dd959888fbb9427e4e62",new_bitmap_glyphs=0,
                source_metatile_count=157,new_metatiles=recipes,map_patches=changes,building_audit=buildings,
                grammar="Roof caps and wall caps only at true exterior sides; no native ground-foundation rows above ground storey; preserve actual doors, footprints, every collision quadrant, all existing scripts/services/story.")
    (folder/"tile-join-correction.contract.json").write_text(json.dumps(result,indent=2)+"\n")
    (folder/"faults.json").write_text(json.dumps(faults,indent=2)+"\n")
    (folder/"receipt.json").write_text(json.dumps(dict(status=result["status"],new_metatiles=len(recipes),changed_map_cells=len(changes),affected_buildings=sum(bool(b["changed_metatiles"])for b in buildings),audited_buildings=len(buildings),fault_glyphs=len(faults),new_bitmap_glyphs=0,palette_entries_changed=0,collision_quadrants_changed=0,warm_render_seconds=round(time.perf_counter()-started,3)),indent=2)+"\n")
    print(json.dumps(dict(output=str(folder),recipes=len(recipes),cells=len(changes),faults=len(faults),seconds=time.perf_counter()-started)))


if __name__=="__main__":
    parser=argparse.ArgumentParser();parser.add_argument("--output",type=Path,required=True);args=parser.parse_args()
    pack_path=ROOT/"content-packs/minnesota-city-river-bend-v01.crystalpack"
    assert hashlib.sha256(pack_path.read_bytes()).hexdigest()=="57eefb675b9cb32ac66a01819f56961392e51678a6f9dd959888fbb9427e4e62"
    render(load_pack(pack_path),args.output)
