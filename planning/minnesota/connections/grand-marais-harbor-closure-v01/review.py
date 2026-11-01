#!/usr/bin/env python3
"""Review a bounded native-art harbor closure; never write a content pack."""
import collections
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

from PIL import Image, ImageChops, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT/'planning/minnesota/tools'))
from audit_city_expansions import module_hash
from audit_outdoor_connections import collision, project
from audit_reachable_surf_boundaries import inspect
from render_archive import save_render
from render_progress_atlas import native_room
from render_reference import SourceArt, layout_blocks, load_pack
from review_north_shore import oriented_sprite
from review_connected_route_native import sources

HERE = Path(__file__).resolve().parent
PACK = ROOT/'content-packs/minnesota-state-cohesion-v01.crystalpack'
EXPECTED = 'c4816242295a236fdfe2a6a96fb248cd3eab11de1b526be0decb7df36e5e95eb'
OUT = ROOT/'planning/minnesota/renders/outdoor-connections/grand-marais-harbor-closure-v01/source-v01'
sha = lambda data: hashlib.sha256(data).hexdigest()
assert sha(PACK.read_bytes()) == EXPECTED
pack = load_pack(PACK)
base = pack['data']['maps']['MnGrandMarais']
assert [base['attributes']['width'], base['attributes']['height']] == [20, 19]
current = deepcopy(base)
patches = []
for x in range(20):
    before = base['blocks'][18*20+x]
    assert before == 0x35
    after = 0x38 if x == 0 else 0x3a if x == 19 else 0x39
    current['blocks'][18*20+x] = after
    patches.append(dict(metatile=[x,18], before_block=before, after_block=after))
# No actor, road, doorway, room warp, script or attribute changes are allowed.
unchanged = deepcopy(current)
unchanged['blocks'] = base['blocks']
assert unchanged == base
old, new = collision(pack,base), collision(pack,current)
delta = [dict(core=list(p),before=old[p],after=new[p]) for p in old if old[p] != new[p]]
assert len(delta) == 42 and all(r['before']=='WATER' and r['after']=='BUOY' for r in delta)
assert all(new[x,37]=='BUOY' for x in range(40))
assert all(new[x,36]=='WATER' for x in range(1,39))
for record in base['objects']+base['events']['warps']+base['events']['bg_events']+base['events']['coord_events']:
    p = record['x'],record['y']
    assert old[p] == new[p]
virtual = dict(pack)
virtual['data'] = dict(pack['data'])
virtual['data']['maps'] = dict(pack['data']['maps'],MnGrandMarais=current)
checks = []
for opened in [False, True]:
    audit = inspect(virtual,opened)
    assert not any(r['map']=='MnGrandMarais' for r in audit['unmatched_edge_records'])
    assert not any(r['same_component_as_proven_river'] for r in audit['unmatched_edge_records'])
    checks.append(dict(gate_state=audit['gate_state'],grand_marais_unmatched_edges=0,
                       other_unmatched_edge_count=len(audit['unmatched_edge_records'])))
# The two endcaps join real blocking neighbor terrain at both outer corners.
joins = []
for side,p in [('west',(0,37)),('east',(39,37))]:
    edge = next(e for e in base['attributes']['connections'] if e['direction']==side)
    target = pack['data']['maps'][edge['target_map']]
    q = project(base['attributes'],target['attributes'],edge,p)
    kind = collision(pack,target)[q]
    assert kind == 'WALL'
    joins.append(dict(direction=side,core=list(p),neighbor=edge['target_map'],neighbor_core=list(q),neighbor_collision=kind))
layout = dict(schema_version=2,id='MnGrandMaraisHarborClosureCandidate',tileset='johto',compiled_tileset='johto',
              dimensions_metatiles=[20,19],shore_fitting='explicit_source_edges',reject_cliff_cuts=True,
              source_pixels_modified=False,layers=[])
for y in range(19):
    for x in range(20):
        layout['layers'].append(dict(kind='block_rect',bounds=[x,y,x+1,y+1],block=f'{current["blocks"][y*20+x]:02x}'))
assert layout_blocks(layout) == current['blocks']
layout_path = HERE/'MnGrandMarais.candidate.layout.json'
layout_path.write_text(json.dumps(layout,indent=2)+'\n')
contract = dict(schema_version=1,status='SOURCE_CANDIDATE_NOT_NATIVE_PUBLICATION_OR_CONTROLLER_PROOF',
    map='MnGrandMarais',base_pack=str(PACK.relative_to(ROOT)),base_pack_sha256=EXPECTED,
    base_module_sha256=module_hash(base),base_blocks_sha256=sha(bytes(base['blocks'])),
    candidate_blocks_sha256=sha(bytes(current['blocks'])),candidate_layout=str(layout_path.relative_to(ROOT)),
    candidate_layout_sha256=sha(layout_path.read_bytes()),dimensions_metatiles=[20,19],
    intent='Close only the reachable, unconnected south harbor edge with native rock breakwater art and BUOY collision.',
    cell_patches=patches,core_collision_changes=delta,shore_joins=joins,
    original_anchor_count=len(base['objects']+base['events']['warps']+base['events']['bg_events']+base['events']['coord_events']),
    original_anchors_collision_unchanged=True,unchanged_except_listed_blocks=True,
    native_recipes={f'{b:02x}':pack['data']['tilesets']['johto']['collision'][f'{b:02x}'] for b in [0x35,0x38,0x39,0x3a]},
    source_checks=checks,remaining_harbor_water_core_cells=sum(k=='WATER' for k in new.values()),
    limitations=['Source candidate only; native681 pack is unchanged.',
      'Future Rust delta must guard exact module/blocks preimages and preserve all other content.',
      'Future native controller test must launch Surf from legal shore and reject every southward exit while retaining harbor access.',
      'Review water-camera Surf sprite is a native source-art marker, not GPU or controller evidence.'])
(HERE/'closure.contract.json').write_text(json.dumps(contract,indent=2)+'\n')
OUT.mkdir(parents=True,exist_ok=True)
art = SourceArt(pack)
base_img = native_room(base,pack,archive=OUT/'base-originals',native_origin=True)
image = native_room(current,pack,archive=OUT/'candidate-originals',native_origin=True)
save_render(base_img,OUT/'whole-town-before.png')
save_render(image,OUT/'whole-town-candidate.png')
save_render(ImageChops.difference(base_img,image),OUT/'whole-town-difference.png')
# Exact 160x144 native player camera: 10x9 core tiles, no seam invention.
for label, point, sprite in [('south-breakwater',(28,35),'SPRITE_SURF'),('west-shore-launch',(3,17),'SPRITE_CHRIS')]:
    x,y = point
    box = (x*16-80,y*16-72,x*16+80,y*16+72)
    left,top = box[0]//32*32,box[1]//32*32
    right,bottom = (box[2]+31)//32*32,(box[3]+31)//32*32
    scene = Image.new('RGBA',(right-left,bottom-top))
    border = art.blocks('johto')[base['attributes']['border_block']]
    for yy in range(0,scene.height,32):
        for xx in range(0,scene.width,32):
            scene.paste(border,(xx,yy))
    regions = sources(virtual,'MnGrandMarais')
    for module,ox,oy in [*regions[1:],regions[0]]:
        a = module['attributes']
        ground = art.render(a['tileset_name'],a['width'],a['height'],module['blocks']).convert('RGBA')
        scene.alpha_composite(image.convert('RGBA') if module is current else ground,(ox-left,oy-top))
    marker = oriented_sprite(pack,sprite,'south')
    save_render(marker,OUT/(label+'-native-source-marker.png'))
    scene.alpha_composite(marker,(x*16-left,y*16-top))
    camera = scene.crop((box[0]-left,box[1]-top,box[2]-left,box[3]-top))
    save_render(camera,OUT/(label+'-native160x144.png'))
    save_render(camera.resize((640,576),Image.Resampling.NEAREST),OUT/(label+'-review4x.png'))
assert sha(PACK.read_bytes()) == EXPECTED
proof = dict(status='SOURCE_REVIEW_PASS_NOT_NATIVE_PLAY',pack_unchanged=True,
    contract_sha256=sha((HERE/'closure.contract.json').read_bytes()),layout_sha256=sha(layout_path.read_bytes()),
    changed_metatiles=20,changed_core_cells=42,south_edge_blocking_core_cells=40,
    checks=checks,shore_joins=joins,personally_visually_reviewed=False)
(OUT/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
print(json.dumps(proof))
