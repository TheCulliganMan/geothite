#!/usr/bin/env python3
"""Inventory the actual complete city's building fronts without editing maps."""
import argparse
import json
from pathlib import Path
import sys

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "planning/minnesota/tools"))
from render_archive import save_render


def inventory(folder):
    folder.mkdir(parents=True, exist_ok=False)
    source = Path(__file__).resolve().parent
    layout = json.loads((source / "MnMinneapolis.layout.previous.json").read_bytes())
    dossier = json.loads((source / "MnMinneapolis.previous.json").read_bytes())
    preimage = json.loads((source / "native-preimage.json").read_bytes())
    scene = Image.open(ROOT / "planning/minnesota/renders/minneapolis-houses-v01/pilot-v03/whole-city-before.png").convert("RGB")
    aliases = {"Pokecenter1F":"Clinic", "Gym":"Powerhouse gym", "Mart":"Mart",
               "DispatchOffice":"Rail dispatch", "DispatcherHouse":"Dispatcher family home"}
    records = []
    for door in dossier["doors"]:
        building = door["target"].removeprefix("MnMinneapolis")
        prefix = aliases.get(building, building) + " "
        points = [layer["position"] for layer in layout["layers"]
                  if layer["kind"] == "point" and layer.get("name", "").startswith(prefix)]
        assert points, building
        x0, y0 = min(p[0] for p in points), min(p[1] for p in points)
        x1, y1 = max(p[0] for p in points) + 1, max(p[1] for p in points) + 1
        blocks = [preimage["native_blocks"][y*40+x] for y in range(y0,y1) for x in range(x0,x1)]
        pilot = building in {"NortheastHome", "RiverNaturalist", "GardenFlat", "PatternStudio"}
        residential = any(token in building for token in ["Home", "Flat", "House", "Cabin", "Inn"])
        style = ("flat masonry" if 0x80 in blocks else
                 "cyan brick, tall blue cap" if 0x18 in blocks else "native special frontage")
        records.append(dict(id=building, door_core=door["tile"], passive_return_core=door["return_tile"],
                            footprint_metatiles=[x0,y0,x1,y1], footprint_core=[(x1-x0)*2,(y1-y0)*2],
                            current_style=style, residential=residential,
                            pilot_v03=pilot, current_block_ids=[f"{b:02x}" for b in blocks]))
    records.sort(key=lambda r:(not r["residential"],r["footprint_metatiles"][1],r["footprint_metatiles"][0]))
    panel_w, panel_h, columns = 256, 340, 5
    sheet = Image.new("RGB",(panel_w*columns,panel_h*((len(records)+columns-1)//columns)),"#14212b")
    draw = ImageDraw.Draw(sheet)
    for i, record in enumerate(records):
        x, y = i%columns*panel_w,i//columns*panel_h
        x0,y0,x1,y1 = record["footprint_metatiles"]
        box = (max(0,x0*32-16),max(0,y0*32-16),min(1280,x1*32+16),min(1024,y1*32+32))
        crop = scene.crop(box)
        assert crop.width<=240 and crop.height<=272,(record["id"],crop.size)
        sheet.paste(crop,(x+(panel_w-crop.width)//2,y+36))
        draw.text((x+8,y+7),f"{i+1:02d} {record['id']}",fill="#98e3b6" if record["pilot_v03"] else "white")
        draw.text((x+8,y+20),record["current_style"],fill="#bfd0db")
        draw.text((x+8,y+320),f"door {tuple(record['door_core'])}; {record['footprint_core']} core",fill="white")
        save_render(crop,folder/f"{i+1:02d}-{record['id']}-native-source.png")
    save_render(sheet,folder/"all-40-native-exterior-fronts.png")
    (folder/"inventory.json").write_text(json.dumps(dict(status="ACTUAL_PARENT_SOURCE_INVENTORY_NOT_REDESIGN",parent_pack_sha256=preimage["pack_sha256"],building_count=len(records),residential_count=sum(r["residential"] for r in records),buildings=records),indent=2)+"\n")
    (source/"exterior-inventory-v01.json").write_text(json.dumps(records,indent=2)+"\n")
    print(json.dumps(dict(output=str(folder),buildings=len(records),residential=sum(r["residential"] for r in records))))


if __name__ == "__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--output",type=Path,required=True)
    inventory(parser.parse_args().output)
