#!/usr/bin/env python3
"""Exact headless winter GPU review using the unchanged connected-camera reviewer.

Extends only this process's private imported reviewer with North Shore glyph20
scrolls and coherent flower/water/falls phases. No production image is edited,
masked, registered or translated by fitting. All captures and references remain.
"""
import importlib.util
import json
import sys
from pathlib import Path

from PIL import ImageChops

ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_archive import save_render
from review_passage_native import digest
from review_city_native import phases as native_water_phases

SHARED=ROOT/'planning/minnesota/tools/review_connected_route_native.py'
spec=importlib.util.spec_from_file_location('winter_private_connected_reviewer',SHARED)
shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
NORTH={'mn_north_shore','mn_winter_north_shore'}
STATIC_FOREST={'mn_border_pines','mn_winter_outdoor'}
original_sources=shared.sources
original_camera_origin=shared.camera_origin


def connected_sources(pack,name):
    """Exact ordinary Rust two-hop placements, active/depth2/depth1 order.

    battle_entry_progress.rs connected_render_map_placements_with_window uses
    a stable breadth-first queue and visits (name,x,y), not only map names.
    Original Crystal's special Route40 gate drawing is outside this Minnesota
    review scope and is explicitly rejected rather than silently approximated.
    """
    assert name.startswith('Mn'), 'winter reviewer only covers Minnesota maps'
    maps=pack['data']['maps'];pending=[(maps[name],0,0,0)];visited={(name,0,0)}
    cursor=0
    while cursor<len(pending):
        source,x,y,depth=pending[cursor];cursor+=1
        if depth>=2:continue
        a=source['attributes']
        for link in a['connections']:
            target=maps[link['target_map']];b=target['attributes'];offset=link['offset']*32
            dx,dy={'north':(offset,-b['height']*32),'south':(offset,a['height']*32),
                   'west':(-b['width']*32,offset),'east':(a['width']*32,offset)}[link['direction'].lower()]
            key=(target['id'],x+dx,y+dy)
            if key not in visited:
                visited.add(key);pending.append((target,x+dx,y+dy,depth+1))
    pending.sort(key=lambda row:(row[3]!=0,-row[3]))
    return [(m,x,y)for m,x,y,d in pending]


def camera_origin(regions,at):
    """Camera clamp still uses only immediate connections, as in native Rust."""
    base=regions[0];a=base[0]['attributes'];immediate=[base]
    for link in a['connections']:
        target=next(m for m,x,y in regions if m['id']==link['target_map']);b=target['attributes'];offset=link['offset']*32
        x,y={'north':(offset,-b['height']*32),'south':(offset,a['height']*32),
             'west':(-b['width']*32,offset),'east':(a['width']*32,offset)}[link['direction'].lower()]
        assert any(m is target and (ox,oy)==(x,y)for m,ox,oy in regions)
        immediate.append((target,x,y))
    return original_camera_origin(immediate,at)


def water_phases(pack,module,scene,box,folder):
    # Preserve the source Border Pines alias's currently static behavior.
    if module['attributes']['tileset_name'] in STATIC_FOREST:
        return iter(())
    return native_water_phases(pack,module,scene,box,folder)


def phases(art,regions,scene,box):
    falling=[];flowers=[]
    for module,ox,oy in regions:
        a=module['attributes'];name=a['tileset_name']
        if name not in NORTH:continue
        files=art.pack['runtime_files'];layout=bytes(files[f'data/tilesets/{name}_metatiles.bin'])
        definition=art.pack['data']['tilesets'][name];blocks=art.blocks(name)
        for index,block in enumerate(module['blocks']):
            for j,glyph in enumerate(layout[block*16:(block+1)*16]):
                if glyph not in {3,0x20}:continue
                x=ox+index%a['width']*32+j%4*8-box[0];y=oy+index//a['width']*32+j//4*8-box[1]
                if x>=scene.width or x+8<=0 or y>=scene.height or y+8<=0:continue
                sx,sy=j%4,j//4
                if glyph==0x20:
                    token=definition['collision'][f'{block:02x}'][(sy//2)*2+sx//2]
                    assert token=='WATERFALL','derived20 may only scroll actual waterfall faces'
                    tile=blocks[block].crop((sx*8,sy*8,sx*8+8,sy*8+8)).convert('RGBA')
                    falling.append((tile,x,y))
                else:
                    # Decode the same source frames and glyph palette as native.
                    pmap=definition['palette_map'];palette=palette_for(art.pack,name,pmap[glyph]&7)
                    frames=[]
                    for filename in ['cgb_1.2bpp','cgb_2.2bpp']:
                        raw=bytes(files['gfx/tilesets/flower/'+filename]);assert len(raw)==16
                        frame=blocks[block].crop((sx*8,sy*8,sx*8+8,sy*8+8)).convert('RGBA')
                        for yy in range(8):
                            for xx in range(8):
                                bit=7-xx;level=(raw[yy*2]>>bit&1)|((raw[yy*2+1]>>bit&1)<<1)
                                frame.putpixel((xx,yy),tuple(palette[level])+(255,))
                        frames.append(frame)
                    flowers.append((frames,x,y))
    for flower in range(2 if flowers else 1):
        for stage in range(8 if falling else 1):
            candidate=scene.copy()
            for tile,x,y in falling:candidate.paste(ImageChops.offset(tile,0,stage*3%8),(x,y))
            for frames,x,y in flowers:candidate.paste(frames[flower],(x,y))
            yield stage+flower*8,candidate


def palette_for(pack,name,index):
    # Reuse source native colors via the palette parser already used in art work.
    from preview_winter_edges_v01 import day_bank
    return day_bank(pack)[index]


def clock_witness(water,fall,flower):
    for tick in range(25080):
        water_at=tick//22%4
        fall_at=0 if tick<16 else ((tick-16)//19+1)%8
        flower_at=tick//60%2
        if (water,fall,flower)==(water_at,fall_at,flower_at):return tick
    raise AssertionError('not a coherent native animation state')


original_compare=shared.compare

def compare(pack,art,view,actual,directory):
    result=original_compare(pack,art,view,actual,directory)
    encoded=result.pop('consistent_falls_scroll_stage')
    if encoded is None:
        result.update(consistent_north_shore_falls_stage=None,consistent_flower_phase=None,coherent_animation_tick_witness=None)
        return result
    fall=encoded%8;flower=encoded//8
    result.update(consistent_north_shore_falls_stage=fall,consistent_flower_phase=flower,
        coherent_animation_tick_witness=clock_witness(result['consistent_water14_phase'],fall,flower),
        tick_witness_is_production_timestamp=False,
        animation_scope='same water phase and same three-pixel falls scroll across every occurrence; no per-glyph phase fitting')
    return result


def configure():
    """Modify only this process's privately imported shared reviewer object."""
    shared.sources=connected_sources;shared.camera_origin=camera_origin
    shared.phases=water_phases;shared.falls_phases=phases;shared.compare=compare
    shared.__doc__=__doc__


if __name__=='__main__':
    configure()
    output=None
    if '--output' in sys.argv:output=Path(sys.argv[sys.argv.index('--output')+1])
    try:shared.main()
    finally:
        if output and output.exists():
            (output/'review_winter_native.py.snapshot').write_bytes(Path(__file__).read_bytes())
            (output/'reviewer-provenance.json').write_text(json.dumps({
                'winter_extension_sha256':digest(Path(__file__)),
                'unchanged_shared_reviewer_sha256':digest(SHARED),
                'production_images_modified':False,'image_masks_or_registration':False,
                'water14':{'frame_ticks':22,'frames':4},
                'north_shore20':{'frame_ticks':19,'phase_offset':16,'advance_on_phase_offset':True,'pixels_down_per_step':3,'stages':8},
                'flower03':{'frame_ticks':60,'frames':2},
                'native_cadence_requires_runtime_tests':True},indent=2)+'\n')
