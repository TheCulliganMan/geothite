#!/usr/bin/env python3
"""Read-only comparison and native-pack pixel review; never export or mutate packs."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time
import textwrap

from PIL import Image

from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from render_archive import retain_render_file, save_render
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
LAYOUT = ROOT / 'planning/minnesota/zones/boundary-waters-safari.layout.json'
CABIN = ROOT / 'planning/minnesota/playable/root-beer-cabin.json'
OUTPUT = ROOT / 'planning/minnesota/renders/boundary-safari/native-compare'
OPPOSITE = dict(east='west', west='east', north='south', south='north')
FACING = dict(UP='north', DOWN='south', LEFT='west', RIGHT='east')
CLOSED_LINES = ['EXPEDITIONS CLOSED','Safari permits are','not available yet.','No fee is charged.']


def verify(pack, layout, cabin, art):
    """Compare supplied native modules with authored strokes and original art."""
    assert layout['dimensions_metatiles'] in ([48, 40], [64, 56]), 'Unsupported Safari epoch'
    width, height = layout['dimensions_metatiles']
    expanded = (width, height) == (64, 56)
    columns, rows = (width+31)//32, (height+23)//24
    ids = {(0,0):1, (1,0):2, (0,1):3, (1,1):4}
    if expanded:
        ids.update({(0,2):6, (1,2):7})
    expected_names = {f'MnBoundarySafariC{column:02}R{row:02}' for column,row in ids}
    group = {entry['name']:entry['mapId'] for entry in pack['data']['runtime_map_metadata'].values()
             if entry['groupId'] == 61}
    assert group == {**{f'MnBoundarySafariC{column:02}R{row:02}':index for (column,row),index in ids.items()},
                     cabin['id']:5}, 'Exact stable group61 module roster'
    expected = layout_blocks(layout)
    source_signs = {sign['id']:sign for sign in layout['signs']}
    actual = [None] * (width * height)
    native_actors, native_signs, chunks = {}, {}, []
    connection_records = 0
    for row in range(rows):
        for column in range(columns):
            name = f'MnBoundarySafariC{column:02}R{row:02}'
            module = pack['data']['maps'][name]
            attr = module['attributes']
            w, h = min(32, width-column*32), min(24, height-row*24)
            assert (attr['width'], attr['height']) == (w, h), name + ': clipping'
            assert attr['tileset_name'] == 'mn_boundary_safari'
            assert len(module['blocks']) == w*h
            metadata = pack['data']['runtime_map_metadata'][attr['map_constant']]
            assert metadata['groupId'] == 61 and metadata['mapId'] == ids[column,row]
            assert metadata['name'] == name and (metadata['width'], metadata['height']) == (w, h)
            for y in range(h):
                at = (row*24+y)*width+column*32
                actual[at:at+w] = module['blocks'][y*w:(y+1)*w]
            for actor in module['objects']:
                key = actor['object_identifier']
                assert key and key not in native_actors, 'Unique actual native actors'
                assert actor['pal'] == 0, 'Sprite helper requires native default palette'
                assert actor['event_flag'] == '-1', 'Review depicts always-present native bodies'
                tile = [actor['x']+column*64, actor['y']+row*48]
                native_actors[key] = dict(actor=actor, tile=tile)
            for sign in module['events']['bg_events']:
                key = sign['script']
                assert key not in native_signs
                native_signs[key] = [sign['x']+column*64, sign['y']+row*48]
                assert key in source_signs and sign['event_type'] == 'BGEVENT_READ'
                assert module['scripts'][key] == [dict(command='jumptext',args=[key+'Text'])]
                text = module['script_text_bodies'][key+'Text']['commands']
                lines = [entry['args'][0].strip('"') for entry in text if entry['command'] in {'text','line','para'}]
                # These two authored expedition prompts are deliberately
                # overridden by installed closed-admission text in both epochs.
                authored_lines = CLOSED_LINES if key in {'BWS_SIGN_01','BWS_SIGN_06'} else source_signs[key]['lines']
                # grand_marais_modpack::authoring::text wraps each authored
                # line at words to Crystal's18 interior columns. Preserve
                # every word and the exact resulting native line sequence.
                expected_lines = [wrapped for line in authored_lines for wrapped in
                                  textwrap.wrap(line,width=18,break_long_words=False,break_on_hyphens=False)]
                assert lines == expected_lines, key + ': native sign text'
            expected_edges = {direction:f'MnBoundarySafariC{c:02}R{r:02}'
                              for direction,c,r in [('west',column-1,row),('east',column+1,row),
                                                    ('north',column,row-1),('south',column,row+1)]
                              if 0 <= c < columns and 0 <= r < rows}
            assert len(attr['connections']) == len(expected_edges)
            assert {edge['direction']:edge['target_map'] for edge in attr['connections']} == expected_edges
            for edge in attr['connections']:
                back = pack['data']['maps'][edge['target_map']]['attributes']['connections']
                assert edge['offset'] == 0
                assert any(e['direction'] == OPPOSITE[edge['direction']] and e['target_map'] == name and e['offset'] == 0 for e in back)
                connection_records += 1
            assert name not in pack['data']['wild_encounters'], 'No ordinary wild table as Safari substitute'
            chunks.append(dict(map=name, size=[w, h], origin=[column*32, row*24]))
    assert actual == expected, f'All {width*height} native exterior blocks must match source strokes'
    source_actors = layout['retire_anchors']+layout['landmark_actors']+layout['decorations']
    assert len(source_actors) == 20
    for source in source_actors:
        native = native_actors[source['id']]
        assert native['tile'] == source['tile'] and native['actor']['sprite'] == source['sprite']
        facing = FACING[native['actor']['spritemovedata'].rsplit('_', 1)[-1]]
        assert facing == source['facing'], source['id'] + ': actual facing'
    gate = layout['entry_gate']['gate_tiles']
    bottom = max(y for x,y in gate)
    gate_bottom = sorted([x,y] for x,y in gate if y == bottom)
    assert len(gate) == 4 and len(gate_bottom) == 2
    assert gate_bottom[1] == [gate_bottom[0][0]+1,bottom]
    assert sorted(gate) == sorted([[x,y] for x,_ in gate_bottom for y in [bottom-1,bottom]])
    guards = dict(zip(['BWS_PENDING_GATE_WEST','BWS_PENDING_GATE_EAST'],gate_bottom))
    assert set(native_actors) == {a['id'] for a in source_actors} | set(guards)
    for key, tile in guards.items():
        native = native_actors[key]
        assert native['tile'] == tile and native['actor']['sprite'] == 'SPRITE_OFFICER'
        assert native['actor']['script'] == key
        assert native['actor']['spritemovedata'] == 'SPRITEMOVEDATA_STANDING_DOWN'
        module = pack['data']['maps'][f'MnBoundarySafariC{tile[0]//64:02}R{tile[1]//48:02}']
        assert module['scripts'][key] == [dict(command='jumptextfaceplayer', args=[key+'Text'])]
        text = module['script_text_bodies'][key+'Text']['commands']
        lines = [entry['args'][0].strip('"') for entry in text if entry['command'] in {'text','line','para'}]
        assert lines == CLOSED_LINES
    assert len(native_signs) == (18 if expanded else 13)
    assert native_signs == {s['id']: s['tile'] for s in layout['signs']}

    collision = pack['data']['tilesets']['mn_boundary_safari']['collision']
    def kind(x, y):
        return collision[f'{actual[y//2*width+x//2]:02x}'][y%2*2+x%2]
    core_width, core_height = width*2, height*2
    perimeter = {(x, y) for y in range(core_height) for x in range(core_width)
                 if x in {0, core_width-1} or y in {0, core_height-1}}
    assert len(perimeter) == (476 if expanded else 348)
    assert all(kind(*p) in {'WALL', 'BUOY'} for p in perimeter), 'Outer walking/Surf closure'
    if not expanded:
        assert kind(39, 46) == 'BUOY' and kind(90, 73) == 'WALL'
    occupied = {tuple(a['tile']) for a in native_actors.values()}
    def flood(start):
        seen = {start}; todo = deque([start])
        while todo:
            x, y = todo.popleft()
            for q in [(x-1,y), (x+1,y), (x,y-1), (x,y+1)]:
                a,b=q
                if not(0<=a<core_width and 0<=b<core_height) or q in seen or q in occupied or kind(a,b) not in {'FLOOR','TALL_GRASS','WATER'}:
                    continue
                seen.add(q); todo.append(q)
        return seen
    arrival = tuple(layout['entry_contract']['arrival_tile'])
    assert kind(*arrival) == 'FLOOR' and arrival not in occupied
    closed = flood(arrival)
    assert len(closed) == 42 and tuple(layout['entry_gate']['launch_target']) not in closed, 'Actual guards seal ordinary arrival'
    assert tuple(layout['entry_gate']['inside_tile']) not in closed
    seams = []
    for row in range(rows):
        for column in range(columns):
            name=f'MnBoundarySafariC{column:02}R{row:02}'
            if column+1 < columns:
                edge=(column+1)*64
                pairs=sum(kind(edge-1,y)==kind(edge,y)=='WATER' for y in range(row*48,min((row+1)*48,core_height)))
                seams.append(dict(pair=[name,f'MnBoundarySafariC{column+1:02}R{row:02}'],water_pairs=pairs))
            if row+1 < rows:
                edge=(row+1)*48
                pairs=sum(kind(x,edge-1)==kind(x,edge)=='WATER' for x in range(column*64,min((column+1)*64,core_width)))
                seams.append(dict(pair=[name,f'MnBoundarySafariC{column:02}R{row+1:02}'],water_pairs=pairs))
    assert len(seams) == (7 if expanded else 4)
    assert all(e['water_pairs'] > 0 for e in seams)

    # Compare actual native imported art, not a replacement/preview art catalog.
    own = art.blocks('mn_boundary_safari')
    johto = art.blocks('johto')
    assert len(own) == (133 if expanded else 132) and len(johto) == 128
    assert [recipe['id'] for recipe in layout['derived_blocks']] == (['80','81','82','83','84'] if expanded else ['80','81','82','83'])
    for block in range(128):
        assert own[block].tobytes() == johto[block].tobytes(), f'Johto pixels changed at{block:02x}'
        assert collision[f'{block:02x}'] == pack['data']['tilesets']['johto']['collision'][f'{block:02x}']
    for recipe in layout['derived_blocks']:
        source = art.blocks(recipe['source_tileset'])[int(recipe['source_block'],16)]
        if 'tile_row_order' in recipe:
            arranged = Image.new('RGB', (32,32))
            for row, original in enumerate(recipe['tile_row_order']):
                arranged.paste(source.crop((0,original*8,32,original*8+8)),(0,row*8))
            source = arranged
        assert own[int(recipe['id'],16)].tobytes() == source.tobytes(), 'Native row/palette recipe '+recipe['id']
        source_set = recipe.get('collision_source_tileset', recipe['source_tileset'])
        source_block = recipe.get('collision_source_block', recipe['source_block'])
        assert collision[recipe['id']] == pack['data']['tilesets'][source_set]['collision'][source_block]
    native_terrain = art.render('mn_boundary_safari',width,height,actual)
    source_terrain = art.render('johto',width,height,expected,derived=layout['derived_blocks'])
    assert native_terrain.tobytes() == source_terrain.tobytes(), 'Whole native/source pixel match'

    room = pack['data']['maps'][cabin['id']]
    assert (room['attributes']['width'],room['attributes']['height']) == (5,4)
    assert room['attributes']['tileset_name'] == 'traditional_house' and not room['objects']
    assert room['blocks'] == [int(b,16) for row in cabin['rows'] for b in row.split()]
    assert len(room['blocks']) == 20
    metadata = pack['data']['runtime_map_metadata']['MN_BOUNDARY_ROOT_BEER_CABIN']
    assert metadata['groupId'] == 61 and metadata['mapId'] == 5
    def room_kind(x,y):
        return pack['data']['tilesets']['traditional_house']['collision'][f'{room["blocks"][y//2*5+x//2]:02x}'][y%2*2+x%2]
    assert len(room['events']['warps']) == 3
    door = layout['root_beer_lady']['door']
    door_tile, return_tile = door['tile'], door['return_tile']
    owner = f'MnBoundarySafariC{door_tile[0]//64:02}R{door_tile[1]//48:02}'
    assert owner in expected_names
    assert (door_tile[0]//64,door_tile[1]//48) == (return_tile[0]//64,return_tile[1]//48)
    owner_constant = pack['data']['maps'][owner]['attributes']['map_constant']
    for warp, (index,tile,kind_name) in zip(room['events']['warps'],[(1,[5,7],'WARP_CARPET_DOWN'),(2,[6,7],'WARP_CARPET_DOWN'),(3,[5,6],'FLOOR')]):
        assert warp['index'] == index and [warp['x'],warp['y']] == tile
        assert warp['target_map'] == warp['target_map_constant'] == owner_constant
        assert warp['target_warp_id'] == 2
        assert room_kind(*tile) == kind_name
    exterior = pack['data']['maps'][owner]
    assert len(exterior['events']['warps']) == 2
    assert all(not pack['data']['maps'][name]['events']['warps'] for name in expected_names-{owner})
    for warp,(index,tile,kind_name) in zip(exterior['events']['warps'],[(1,door_tile,'DOOR'),(2,return_tile,'FLOOR')]):
        assert warp['index'] == index and [warp['x'],warp['y']] == [tile[0]%64,tile[1]%48]
        assert warp['target_map'] == warp['target_map_constant'] == 'MN_BOUNDARY_ROOT_BEER_CABIN'
        assert warp['target_warp_id'] == 3
        assert kind(*tile) == kind_name
    report = dict(pack_content_hash=pack['identity']['content_hash'],pack_maps=len(pack['data']['maps']),
                  layout_canonical_json_sha256=hashlib.sha256(json.dumps(layout,sort_keys=True).encode()).hexdigest(),
                  dimensions_metatiles=[width,height],chunks=chunks,exterior_blocks=width*height,cabin_blocks=20,source_actors=20,closed_guards=2,
                  signs=len(native_signs),reciprocal_connection_records=connection_records,water_seams=seams,outer_closed_cells=len(perimeter),
                  closed_forecourt_reachable_cells=len(closed),original_johto_blocks_pixel_collision_equal=128,
                  cabin_exterior=owner,cabin_door_global=door_tile,cabin_return_global=return_tile,
                  imported_native_art_recipes_equal=[r['id'] for r in layout['derived_blocks']],
                  whole_terrain_source_native_pixel_equal=True,actual_default_cabin_actors=0,
                  outside_dorothy=1,arrival_index=3,carpet_indices=[1,2],admission='CLOSED',
                  status='Read-only published-pack comparison and source-pixel render; production controller/captures are separate')
    return native_terrain,source_terrain,native_actors,room,report


def negative_checks(pack, layout, cabin, art):
    """Reject targeted mutations without changing any supplied pack or source."""
    rejected=[]
    for case in ['stable map ID','missing sign','moved closed guard','opened perimeter',
                 'broken reciprocal seam','wrong cabin return']:
        changed=dict(pack);changed['data']=dict(pack['data'])
        changed['data']['maps']=dict(pack['data']['maps'])
        if case == 'stable map ID':
            changed['data']['runtime_map_metadata']=deepcopy(pack['data']['runtime_map_metadata'])
            next(value for value in changed['data']['runtime_map_metadata'].values()
                 if value['name']=='MnBoundarySafariC00R00')['mapId']=99
        else:
            owner='MnBoundarySafariC00R00'
            if case == 'wrong cabin return':owner=cabin['id']
            if case == 'moved closed guard':
                gate=layout['entry_gate']['gate_tiles'];x,y=next(p for p in gate if p[1]==max(p[1] for p in gate))
                owner=f'MnBoundarySafariC{x//64:02}R{y//48:02}'
            module=deepcopy(pack['data']['maps'][owner]);changed['data']['maps'][owner]=module
            if case == 'missing sign':module['events']['bg_events'].pop()
            elif case == 'moved closed guard':
                next(actor for actor in module['objects'] if actor['object_identifier']=='BWS_PENDING_GATE_WEST')['x']+=1
            elif case == 'opened perimeter':module['blocks'][0]=0x35
            elif case == 'broken reciprocal seam':module['attributes']['connections'].pop()
            elif case == 'wrong cabin return':module['events']['warps'][0]['target_warp_id']=1
        try:
            verify(changed,layout,cabin,art)
        except AssertionError:
            rejected.append(case)
        else:
            raise AssertionError('Native negative fixture accepted: '+case)
    return rejected


def render(pack, layout, cabin, output, self_test=False):
    started = time.perf_counter(); art = SourceArt(pack)
    terrain, source_terrain, actors, room, report = verify(pack,layout,cabin,art)
    if self_test:
        report['negative_cases_rejected']=negative_checks(pack,layout,cabin,art)
    scene = terrain.convert('RGBA')
    for entry in actors.values():
        actor=entry['actor'];x,y=entry['tile']
        facing=FACING[actor['spritemovedata'].rsplit('_',1)[-1]]
        scene.alpha_composite(oriented_sprite(pack,actor['sprite'],facing),(x*16,y*16-4))
    output.mkdir(parents=True,exist_ok=True)
    def save(name,image):
        # Preserve the original mode and dimensions. Include both in the key so
        # unscaled RGBA scenes and RGB previews cannot replace one another.
        identity=f'{image.mode}:{image.width}x{image.height}:'.encode()+image.tobytes()
        digest=hashlib.sha256(identity).hexdigest()[:12]
        path=output/f'{name}-{digest}.png'
        if path.exists():
            with Image.open(path) as previous:
                assert previous.mode==image.mode and previous.size==image.size and previous.tobytes()==image.tobytes(), 'Immutable render collision: '+str(path)
            retain_render_file(path)
        else:
            save_render(image,path)
        return path.name
    report['whole_original_rgba_image']=save('whole-native-preserve-original-rgba',scene)
    report['whole_image']=save('whole-native-preserve',scene.convert('RGB'))
    report['terrain_image']=save('native-terrain',terrain)
    report['source_terrain_image']=save('source-terrain',source_terrain)
    room_image=art.render('traditional_house',5,4,room['blocks'])
    report['cabin_image']=save('actual-empty-cabin',room_image)
    report['cabin_4x_image']=save('actual-empty-cabin-4x',room_image.resize((640,512),Image.Resampling.NEAREST))
    views=[];report['camera_images']={};report['camera_original_images']={};report['camera_source_pixel_images']={}
    # Exact160×144 source-pixel crops; no player sprite or gameplay state is invented.
    central = next(view for view in layout['viewpoints'] if view['id'] == 'CENTRAL_CHANNEL')
    gate = layout['entry_gate']['gate_tiles']
    cameras=layout['viewpoints']+[dict(id='CENTRAL_OPEN_WATER',tile=central['tile']),
                                dict(id='CLOSED_GUARDS',tile=[min(x for x,y in gate),max(y for x,y in gate)+1])]
    for view in cameras:
        x,y=view['tile'];original=scene.crop((x*16-80,y*16-72,x*16+80,y*16+72))
        name='camera-'+view['id'].lower()
        report['camera_original_images'][view['id']]=save(name+'-original-rgba-160x144',original)
        image=original.convert('RGB')
        report['camera_source_pixel_images'][view['id']]=save(name+'-160x144',image)
        views.append((view['id'],image))
        report['camera_images'][view['id']]=save(name,image.resize((640,576),Image.Resampling.NEAREST))
    report['camera_sheet']=save('native-camera-sheet',room_sheet(views,columns=4))
    report['render_compare_ms']=round((time.perf_counter()-started)*1000,1)
    report['ok']=True
    identity=report['pack_content_hash'][:12]
    report_text=json.dumps(report,indent=2)+'\n'
    report_digest=hashlib.sha256(report_text.encode()).hexdigest()[:12]
    # Keep each run's evidence too; the stable report remains a convenience.
    (output/f'native-compare-{identity}-{report_digest}.json').write_text(report_text)
    (output/f'native-compare-{identity}.json').write_text(report_text)
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-boundary-waters.crystalpack')
    parser.add_argument('--output',type=Path,default=OUTPUT)
    parser.add_argument('--layout',type=Path,default=LAYOUT)
    parser.add_argument('--cabin',type=Path,default=CABIN)
    parser.add_argument('--self-test',action='store_true')
    args=parser.parse_args()
    report=render(load_pack(args.pack),json.loads(args.layout.read_text()),json.loads(args.cabin.read_text()),args.output,args.self_test)
    print(json.dumps(report,indent=2))


if __name__=='__main__':
    main()
