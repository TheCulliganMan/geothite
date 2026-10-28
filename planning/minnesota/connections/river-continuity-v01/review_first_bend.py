#!/usr/bin/env python3
"""Review an explicit river-bend candidate against immutable native preimages.

Read-only pack use. This writes authoring layouts, bounded cell contracts and
native-art previews, never a game pack. There is no terrain generation.
"""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from audit_city_expansions import module_hash
from render_progress_atlas import font, native_room
from render_reference import load_pack
from review_north_shore import oriented_sprite
from audit_water_edges import production_permissions, numeric, blocked

PLAN = Path(__file__).resolve().parent
EXPECTED_PACK = '92bb645535db23c78491b6982ddf5fa17b72b1f0434dfb6456a395fbf8fdbd33'

# Explicit rows, not inferred shoreline. Existing native 83/84/85 form the
# north/south walking deck and east/west Surf underpass. 80 is a retaining edge.
ROWS = {
    'MnR03C00R00': {
        0: '05 05 05 05 05 05 58 35 59 05 01 05 05 05 05 05 05 05 05 05 05 05 05 05',
        1: '05 05 05 05 05 05 58 35 35 83 84 85 76 76 76 76 76 76 76 76 76 76 76 76',
        2: '05 05 05 05 05 05 58 35 35 83 84 85 35 35 35 35 35 35 35 35 35 35 35 35',
        3: '05 05 05 05 02 02 80 80 80 83 84 85 80 80 80 35 35 35 59 35 35 35 35 35',
        4: '05 05 05 05 02 02 02 02 03 03 03 45 05 05 05 58 35 35 59 39 39 39 39 39',
    },
    'MnMississippiConfluenceC00R00': {
        1: '76 76 76 76 76 76',
        2: '35 35 35 35 35 35',
        3: '35 35 35 35 35 35',
        4: '39 39 39 39 38 35',
        5: '05 05 05 05 05 58',
    },
}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


def layout(module, label):
    width, height = module['attributes']['width'], module['attributes']['height']
    layers = []
    for y in range(height):
        row = module['blocks'][y * width:(y + 1) * width]
        start = 0
        while start < width:
            end = start + 1
            while end < width and row[end] == row[start]:
                end += 1
            layers.append(dict(name=f'Explicit row {y:02} x{start:02}..{end-1:02}',
                               kind='block_rect', bounds=[start, y, end, y+1],
                               block=f'{row[start]:02x}'))
            start = end
    return dict(schema_version=2, id=module['id'], status=label,
                dimensions_metatiles=[width, height],
                tileset=module['attributes']['tileset_name'],
                compiled_tileset=module['attributes']['tileset_name'],
                source_pixels_modified=False, shore_fitting='explicit_source_edges',
                reject_cliff_cuts=True, layers=layers)


def composite(pack, placements, names, scale, output, title, annotations=()):
    rows = [r for r in placements if r['map'] in names]
    x0 = min(r['origin_metatiles'][0] for r in rows)
    y0 = min(r['origin_metatiles'][1] for r in rows)
    x1 = max(r['origin_metatiles'][0]+r['dimensions_metatiles'][0] for r in rows)
    y1 = max(r['origin_metatiles'][1]+r['dimensions_metatiles'][1] for r in rows)
    im = Image.new('RGB', ((x1-x0)*scale, (y1-y0)*scale+88), '#121f2b')
    draw = ImageDraw.Draw(im)
    draw.text((12, 9), title, font=font(20, True), fill='white')
    draw.text((12, 38), 'Native source art. Actual map positions. No filler terrain or invented joins.',
              font=font(14), fill='#c5d9d7')
    draw.text((12, 59), 'Source candidate only; production controller verification is pending.',
              font=font(14), fill='#e6c289')
    origins = {}
    for row in rows:
        name = row['map']
        x, y = row['origin_metatiles']
        width, height = row['dimensions_metatiles']
        scene = native_room(pack['data']['maps'][name], pack, native_origin=True)
        origin = ((x-x0)*scale, (y-y0)*scale+88)
        im.paste(scene.resize((width*scale, height*scale), Image.Resampling.NEAREST), origin)
        origins[name] = origin
    for name, bounds, text in annotations:
        ox, oy = origins[name]
        a,b,c,d = bounds
        rect = (ox+a*scale,oy+b*scale,ox+c*scale-1,oy+d*scale-1)
        draw.rectangle(rect, outline='#ff7d54', width=max(2,scale//8))
        tx,ty = rect[0]+4,rect[1]+4
        box = draw.textbbox((tx,ty),text,font=font(13,True))
        draw.rectangle((box[0]-2,box[1]-2,box[2]+2,box[3]+2), fill='#17232e')
        draw.text((tx,ty),text,font=font(13,True),fill='#ffe7a9')
    im.save(output)
    return im


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'Preserve every prior pass; choose a fresh output.'
    args.output.mkdir(parents=True)
    raw = args.pack.read_bytes()
    assert sha(raw) == EXPECTED_PACK, 'Continuity parent is not the reviewed immutable pack.'
    before = load_pack(args.pack)
    after = dict(before)
    after['data'] = dict(before['data'])
    after['data']['maps'] = dict(before['data']['maps'])
    for name in ROWS:
        after['data']['maps'][name] = deepcopy(before['data']['maps'][name])
    maps = after['data']['maps']
    changes = []
    for name, rows in ROWS.items():
        original = before['data']['maps'][name]
        module = maps[name]
        width = module['attributes']['width']
        cells = []
        for y, text in rows.items():
            values = [int(v,16) for v in text.split()]
            assert len(values) == width
            for x, value in enumerate(values):
                i = y*width+x
                old = module['blocks'][i]
                if value != old:
                    cells.append(dict(metatile=[x,y], before=f'{old:02x}', after=f'{value:02x}'))
                    module['blocks'][i] = value
        expected = deepcopy(original)
        expected['blocks'] = module['blocks']
        assert expected == module, 'Only declared terrain cells may change.'
        event_changes = []
        if name == 'MnR03C00R00':
            sign = next(e for e in module['events']['bg_events'] if e['script'] == 'MN_R03_NORTH_SIGN')
            assert [sign['x'],sign['y']] == [22,4]
            sign['y'] = 8
            event_changes.append(dict(group='bg_events',script=sign['script'],before_core=[22,4],after_core=[22,8],
                                      reason='Retain the existing sign beside the widened bridge, on a visible native sign block.'))
        changes.append(dict(map=name, module_sha256=module_hash(original),
                            dimensions_metatiles=[width, module['attributes']['height']],
                            tileset=module['attributes']['tileset_name'], cells=cells,event_changes=event_changes))
        write_json(args.output/'source'/f'{name}.before.layout.json', layout(original,'FROZEN_NATIVE_PREIMAGE'))
        write_json(args.output/'source'/f'{name}.candidate.layout.json', layout(module,'DETACHED_SOURCE_CANDIDATE_NOT_PUBLISHED'))
        for epoch, pack in [('before',before),('candidate',after)]:
            native_room(pack['data']['maps'][name],pack,native_origin=True).save(args.output/f'{name}-{epoch}-whole.png')
    placements = json.loads((ROOT/'planning/minnesota/renders/ice/native-geography-v03/atlas/native-connection-audit.json').read_bytes())['placements']
    manifest = json.loads((ROOT/'planning/minnesota/connections/mississippi-v02/manifest.json').read_bytes())
    all_names = {r['id'] for r in manifest['new_maps']} | {'MnMinneapolis','MnStPaul','MnR04C00R00','MnR03C00R00','MnWinona','MnStCloud','MnR05C00R00','MnR05C00R01','MnR05C00R02'}
    close_names = {'MnMinneapolis','MnR03C00R00','MnMississippiConfluenceC00R00','MnMississippiConfluenceC01R00','MnR04C00R00','MnStPaul','MnMississippiConfluenceC02R00','MnMississippiConfluenceC03R00'}
    for epoch, pack in [('before',before),('candidate',after)]:
        composite(pack,placements,all_names,12,args.output/f'whole-corridor-{epoch}.png',f'MISSISSIPPI CONTINUITY / {epoch.upper()}')
        composite(pack,placements,close_names,20,args.output/f'twin-cities-{epoch}.png',f'TWIN CITIES / {epoch.upper()}')
        # These source cameras use complete actually abutting maps. They are
        # static native-art previews, not GPU or reached gameplay captures.
        strip = Image.new('RGB',(40*32,56*32),'#121f2b')
        strip.paste(native_room(pack['data']['maps']['MnMinneapolis'],pack,native_origin=True),(0,0))
        strip.paste(native_room(pack['data']['maps']['MnR03C00R00'],pack,native_origin=True),(10*32,32*32))
        strip.paste(native_room(pack['data']['maps']['MnMississippiConfluenceC00R00'],pack,native_origin=True),(34*32,32*32))
        strip.save(args.output/f'junction-native32-{epoch}-original.png')
        for label,(x,y),facing in [('bridge-south',(20,9),'north'),('bridge-west-surf',(17,5),'east'),('sign-approach',(22,9),'north')]:
            px,py=10*32+x*16,32*32+y*16
            crop=strip.crop((px-72,py-64,px+88,py+80)).convert('RGBA')
            sprite='SPRITE_SURF' if label=='bridge-west-surf' else 'SPRITE_CHRIS'
            crop.alpha_composite(oriented_sprite(pack,sprite,facing),(72,64))
            crop.save(args.output/f'{label}-{epoch}-native160.png')
            crop.resize((640,576),Image.Resampling.NEAREST).save(args.output/f'{label}-{epoch}-4x.png')
    # Exact two-sided seam in gameplay coordinates; lower retaining edge is
    # allowed to reduce the last row to one legal water quadrant.
    from audit_outdoor_connections import collision
    kinds = {name:collision(after,maps[name]) for name in ROWS}
    seam = []
    for y in range(16):
        a = kinds['MnR03C00R00'][(47,y)]
        b = kinds['MnMississippiConfluenceC00R00'][(0,y)]
        assert a == b, (y,a,b)
        seam.append(dict(r03_core=[47,y],confluence_core=[0,y],permission=a))
    # All existing event coordinates retain their exact native foot permission.
    # The north sign remains on a walkable bridge deck, not in the new channel.
    anchors = []
    for name in ROWS:
        old = before['data']['maps'][name]
        old_kinds = collision(before,old)
        for group, values in [('objects',old['objects']), *old['events'].items()]:
            for obj in values:
                point = obj['x'],obj['y']
                moved = name == 'MnR03C00R00' and group == 'bg_events' and obj['script'] == 'MN_R03_NORTH_SIGN'
                candidate_point = (22,8) if moved else point
                prior,current = old_kinds[point],kinds[name][candidate_point]
                assert prior == current or {prior,current} <= {'FLOOR','BRIDGE_EW_DECK'}, (name,group,point,prior,current)
                anchors.append(dict(map=name,group=group,before_core=list(point),candidate_core=list(candidate_point),before=prior,after=current))
    contract = dict(schema_version=1,status='DETACHED_SOURCE_CANDIDATE_NOT_PUBLISHED',
                    parent_pack=str(args.pack),parent_pack_sha256=EXPECTED_PACK,
                    parent_identity=before['identity']['content_hash'],changes=changes,
                    retained_connections='Every native connection is byte-for-byte unchanged.',
                    preserved='Every module field except listed blocks and one declared sign bg-event coordinate; every other map, art, program, event, NPC, room, audio and registry is unchanged in this in-memory proposal. A Rust publisher must update the matching native event declaration and mirrors.',
                    required_native_tests=['Original city gate and waterfall denials still hold.',
                      'Walk the north/south R03 bridge in both directions; read the existing north sign.',
                      'Surf through all three bridge rows in both directions.',
                      'Cross the widened two-sided R03/Confluence seam in both directions.',
                      'Follow Winona–St. Cloud through the complete long wooded reaches.'])
    write_json(args.output/'candidate.contract.json',contract)
    tokens,water,wall,engine_hashes=production_permissions()
    blocked_mounts=set(after['data']['field_moves']['surf']['blocked_collisions'])
    def permission(name,p):
        return numeric(kinds[name][p],tokens)
    def enter(value,direction,mode):
        if 0xe0<=value<=0xe3:
            return value in {0xe0,0xe2} if mode=='walk' else not (direction in {'north','south'} if value in {0xe2,0xe3} else direction in {'west','east'})
        if blocked(value,direction):
            return False
        return value not in wall and (mode=='surf' or value not in water)
    def exit_allowed(value,direction,mode):
        if mode=='surf' and value in {0xe2,0xe3} and direction in {'north','south'}:
            return False
        if mode=='surf' and value in {0xe0,0xe1} and direction in {'west','east'}:
            return False
        return not blocked(value,direction,True)
    paths=[]
    def prove(label,path,mode):
        name='MnR03C00R00'
        for a,b in zip(path,path[1:]):
            dx,dy=b[0]-a[0],b[1]-a[1]
            direction={(0,-1):'north',(0,1):'south',(-1,0):'west',(1,0):'east'}[(dx,dy)]
            va,vb=permission(name,a),permission(name,b)
            assert exit_allowed(va,direction,mode) and enter(vb,direction,mode),(label,a,b,va,vb)
        paths.append(dict(label=label,mode=mode,map=name,core=[list(p)for p in path]))
    foot=[(20,y)for y in range(13)]
    prove('North to south road and return',foot,'walk');prove('South to north road',list(reversed(foot)),'walk')
    for y in [2,3,4,5,6,7]:
        path=[(x,y)for x in range(17,26)]
        prove(f'EW underpass y{y}',path,'surf');prove(f'WE underpass y{y}',list(reversed(path)),'surf')
    prove('Existing north sign approach',[(20,y)for y in range(10)]+[(21,9),(22,9)],'walk')
    assert permission('MnR03C00R00',(22,8)) in wall
    assert all(not enter(permission('MnR03C00R00',(x,y)),'west' if x==18 else 'east','walk') for x in [18,23] for y in range(2,8))
    assert all(not exit_allowed(permission('MnR03C00R00',(x,y)),d,'surf') for x in range(18,24)for y in range(2,8)for d in ['north','south'])
    assert all(before['data']['maps'][n] is maps[n] for n in maps if n not in ROWS)
    write_json(args.output/'bridge-static-proof.json',dict(status='STATIC_SOURCE_ONLY',engine_hashes=engine_hashes,
        exact_paths=paths,assertions=['Both foot directions remain on the production deck.',
          'All six core rows of the widened bridge admit east/west Surf in both directions.',
          'Both bridge rails reject lateral Foot entry.',
          'Every bridge permission rejects Surf north/south exit onto the upper road.',
          'The relocated sign has WALL collision at its target and a reachable FLOOR approach.',
          'All other maps are unchanged; no connections or gates changed.'],
        blocked_mount_permissions=sorted(blocked_mounts),note='Actual input and native sign dialogue must still be tested after Rust publication.'))
    write_json(args.output/'static-evidence.json',dict(status='STATIC_SOURCE_ONLY',
        seam=seam,unchanged_anchor_permissions=anchors,
        changed_cells=sum(len(r['cells'])for r in changes),
        note='Bridge underpass is production-specific. Static preview does not prove input, HM gates, battle, Save/Continue or earned progression.'))
    print(json.dumps(dict(output=str(args.output),changed_cells=sum(len(r['cells'])for r in changes))))


if __name__ == '__main__':
    main()
