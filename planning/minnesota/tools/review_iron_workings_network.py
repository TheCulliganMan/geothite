#!/usr/bin/env python3
"""Render actual native mine floors at one scale with their retained shaft pairs.

Letters identify native warps between distinct floors, not invented walking
seams. This is default closed-gate terrain; controller state review is separate.
"""
import argparse
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw

from render_reference import SourceArt, load_pack, npc_palette_pack
from review_passage_native import actors

ROOT = Path(__file__).resolve().parents[3]
ROOMS = ['MnIronHaulage', 'MnIronVentilation', 'MnIronSump']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--include-pump', action='store_true', help='Add the current pump floor and its original drill-core shaft.')
    parser.add_argument('--include-core', action='store_true', help='Show all five rock floors including the drill core; implies --include-pump.')
    args = parser.parse_args()
    args.include_pump = args.include_pump or args.include_core
    if args.pack is None:
        world = json.loads((ROOT/'planning/minnesota/world.json').read_text())
        args.pack = ROOT/world[world['active_local_review_revision']]['pack']
    args.output.mkdir(parents=True, exist_ok=False)
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    native = npc_palette_pack(pack, 'nite')
    network = json.loads((ROOT/'planning/minnesota/lairs/iron-workings/network.json').read_text())
    rooms = [*ROOMS, 'MnIronPumpGallery'] if args.include_pump else ROOMS
    if args.include_core:
        rooms.append('MnIronDrillCore')
    images = {}
    for room in rooms:
        module = pack['data']['maps'][room]
        if room == 'MnIronDrillCore':
            # Reconcile the actual default callback: the crew appears only
            # after shutdown. This changes actor visibility, never pixels.
            callback = module['scripts']['MnNDrillCoreActors']
            assert any(c['command'] == 'disappear' and 'MN_NDRILL_FINN' in c['args'] for c in callback)
            assert any('EVENT_MN_DRILL_STOPPED' in c['args'] for c in callback)
            module = dict(module)
            module['objects'] = [o for o in module['objects'] if o['script'] != 'MN_NDRILL_FINN']
        a = module['attributes']
        assert a['time_of_day'] == 'nite'
        scene = actors(native, art.render(a['tileset_name'], a['width'], a['height'], module['blocks'], 'nite'), module).convert('RGB')
        scene.save(args.output/(room+'-original.png'))
        images[room] = scene
    positions = {'MnIronHaulage': (24, 104), 'MnIronVentilation': (688, 104),
                 'MnIronSump': (316, 642)}
    if args.include_pump:
        positions.update(MnIronSump=(24, 624), MnIronPumpGallery=(752, 624))
    if args.include_core:
        positions['MnIronDrillCore'] = (448,1136)
    sheet = Image.new('RGB', (1288, 1700 if args.include_core else 1290 if args.include_pump else 1210), '#171d24')
    draw = ImageDraw.Draw(sheet)
    draw.text((24, 18), 'IRON WORKINGS - ACTUAL NATIVE FLOORS', fill='#f4f1e4')
    draw.text((24, 40), 'Same native scale. Letters identify real shaft pairs; they are not walking seams.', fill='#b6cbd5')
    draw.text((24, 60), 'Default terrain: pressure shutters closed. Operate HAUL and VENT before Sump pump access.', fill='#d4b77b')
    for room in rooms:
        x, y = positions[room]
        draw.text((x, y-21), room, fill='#f4f1e4')
        sheet.paste(images[room], (x, y))
        draw.rectangle((x-1, y-1, x+images[room].width, y+images[room].height), outline='#506774')
    colors = ['#ffcc66', '#70e6c9', '#b8a2ff', '#89beff', '#ff9b89', '#ec9be6', '#ccdf73', '#e2e6ed', '#9cdb8a']
    links = list(network['links'])
    if args.include_pump:
        links.append(dict(id='drill_core',
            a=dict(map='MnIronPumpGallery', active=[25,4], arrival=[24,5]),
            b=dict(map='MnIronDrillCore', active=[5,14], arrival=[4,15])))
    bindings = []
    for i, link in enumerate(links):
        letter, color = chr(65+i), colors[i]
        for end, other in [(link['a'], link['b']), (link['b'], link['a'])]:
            room = end['map']
            module = pack['data']['maps'][room]
            warp = next(w for w in module['events']['warps'] if [w['x'], w['y']] == end['active'])
            target = pack['data']['maps'][other['map']]
            assert warp['target_map_constant'] == target['attributes']['map_constant']
            arrival = next(w for w in target['events']['warps'] if w['index'] == warp['target_warp_id'])
            assert [arrival['x'], arrival['y']] == other['arrival']
            if room not in positions:
                continue
            x, y = positions[room]
            px, py = x+end['active'][0]*16, y+end['active'][1]*16
            draw.rectangle((px-1, py-1, px+16, py+16), outline=color, width=2)
            draw.rectangle((px+2, py-13, px+13, py-2), fill='#171d24')
            draw.text((px+4, py-13), letter, fill=color)
        label = f"{letter}  {link['id']}: {link['a']['map']} <-> {link['b']['map']}"
        draw.text((24+(i%2)*632, (1500 if args.include_core else 1120 if args.include_pump else 1066)+(i//2)*27), label, fill=color)
        bindings.append(dict(letter=letter, **link))
    controls = [('HAUL', 'MnIronHaulage', (9,8)),
                ('VENT', 'MnIronVentilation', (9,6)),
                ('BOTH', 'MnIronSump', (16,3))]
    if args.include_pump:
        controls.extend([('FEED', 'MnIronPumpGallery', (12,3)),
                         ('FEED', 'MnIronPumpGallery', (7,8))])
    for label, room, at in controls:
        x, y = positions[room]
        px, py = x+at[0]*32, y+at[1]*32
        draw.rectangle((px, py, px+31, py+31), outline='#ffdd75', width=2)
        draw.text((px-2, py+34), label, fill='#ffdd75', stroke_width=1, stroke_fill='#171d24')
    sheet.save(args.output/'connected-native-floors-original.png')
    sheet.resize((sheet.width*2, sheet.height*2), Image.Resampling.NEAREST).save(args.output/'connected-native-floors-2x.png')
    receipt = dict(status='PASS', scope='Actual default closed terrain and reciprocal shaft bindings; input/course evidence is separate.',
                   pack=str(args.pack), pack_sha256=hashlib.sha256(args.pack.read_bytes()).hexdigest(),
                   network_sha256=hashlib.sha256((ROOT/'planning/minnesota/lairs/iron-workings/network.json').read_bytes()).hexdigest(),
                   rooms=rooms, native_shaft_pairs=bindings)
    (args.output/'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
    print('PASS', len(rooms), 'native floors and', len(bindings), 'original reciprocal shaft pairs')


if __name__ == '__main__':
    main()
