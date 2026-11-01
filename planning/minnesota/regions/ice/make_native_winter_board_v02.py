#!/usr/bin/env python3
"""Review written winter terrain with actual actors, without publishing a pack."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
from PIL import Image, ImageChops, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT/'planning/minnesota/tools'))
from render_archive import save_render
from render_progress_atlas import font
from render_reference import load_pack
from review_major_city import ReviewArt, render_map

PACK_SHA = None
BASE = ROOT/'planning/minnesota/renders/ice'
SOURCE = [BASE/'region-v01/final-v02-native-slots/whole-native-source.png',
          BASE/'sections-v01/review-v03/r17/after/whole-source-scene.png',
          BASE/'sections-v01/source-review-v04-v02/r11/whole-source-scene.png']
TERRAIN_SOURCE = [BASE/'region-v01/final-v02-native-slots/whole-native-source-terrain.png',
                  BASE/'sections-v01/review-v03/r17/after/whole-terrain-original.png',
                  BASE/'sections-v01/source-review-v04-v02/r11/whole-terrain-original.png']


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def objects(module):
    return [dict(id=o.get('object_identifier') or o['script'], sprite=o['sprite'],
                 tile=[o['x'], o['y']],
                 facing={'UP':'north','DOWN':'south','LEFT':'west','RIGHT':'east'}.get(o['spritemovedata'].rsplit('_',1)[-1], 'south'),
                 native_palette_override=o.get('pal', 0)) for o in module['objects']]


def route_modules(pack, prefix):
    result=[]
    for name, module in pack['data']['maps'].items():
        match=re.fullmatch(prefix+r'C(\d+)R(\d+)', name)
        if match:
            c,r=map(int, match.groups());result.append((name,module,(c*32,r*24)))
    return sorted(result)


def compare(actual, source_path, directory):
    source=Image.open(source_path).convert('RGBA');actual=actual.convert('RGBA')
    assert actual.size==source.size, (actual.size,source.size)
    save_render(source,directory/'reviewed-source-original.png')
    diff=ImageChops.difference(actual, source);save_render(diff,directory/'native-vs-source-difference.png')
    changed=[(i%actual.width,i//actual.width) for i,(a,b) in enumerate(zip(actual.getdata(),source.getdata())) if a!=b]
    return dict(source=str(source_path.relative_to(ROOT)), source_sha256=sha(source_path),
                dimensions=list(actual.size), compared_pixels=actual.width*actual.height,
                changed_pixels=len(changed),
                difference_bounds_exclusive=None if not changed else [min(x for x,y in changed),min(y for x,y in changed),max(x for x,y in changed)+1,max(y for x,y in changed)+1])


def main():
    global PACK_SHA
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--pack',type=Path,required=True);p.add_argument('--atlas',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--pack-sha256',required=True);p.add_argument('--roof-proof',type=Path,required=True);args=p.parse_args();PACK_SHA=args.pack_sha256
    assert sha(args.pack)==PACK_SHA
    atlas=json.loads((args.atlas/'native-connection-audit.json').read_text());assert atlas['pack_sha256']=='aa1aa251f7383640412b82b9b3ed9d6e0bb1304569230a79624c75274db60e5a'
    roof_proof=json.loads(args.roof_proof.read_bytes());assert roof_proof['status']=='READ_ONLY_FINAL_WRITTEN_SOURCE_ALIGNMENT_PASS' and roof_proof['after_pack_sha256']==PACK_SHA
    prior_path=ROOT/'content-packs/minnesota-northern-winter-v01.crystalpack';assert sha(prior_path)==atlas['pack_sha256'];prior=load_pack(prior_path)
    pack=load_pack(args.pack);art=ReviewArt(pack);args.output.mkdir(parents=True,exist_ok=False)
    for name in ['MnRainyRiverIcefields','MnR20C00R00']:
        assert pack['data']['maps'][name]==prior['data']['maps'][name], 'Inherited native atlas seam module changed'
        alias=pack['data']['maps'][name]['attributes']['tileset_name'];assert pack['data']['tilesets'][alias]==prior['data']['tilesets'][alias]
        for path,data in prior['runtime_files'].items():
            if alias in path:assert pack['runtime_files'][path]==data
    source_bundle=ROOT/'planning/minnesota/regions/ice/sections/source-v04';source_contract=json.loads((source_bundle/'final-source.contract.json').read_bytes())
    for path,expected in source_contract['sources'].items():assert sha(ROOT/path)==expected
    groups=[('Rainy River Icefields','Complete new winter region · 32 × 24 metatiles', [('MnRainyRiverIcefields',pack['data']['maps']['MnRainyRiverIcefields'],(0,0))]),
            ('Border Pines / R17','Installed cedar woods and marsh · existing southern snowline', route_modules(pack,'MnR17')),
            ('North Shore / R11','Installed upper gorge frost · full open coast and buoy cap', route_modules(pack,'MnR11'))]
    wholes=[];receipts=[]
    for index,(title,caption,modules) in enumerate(groups):
        width=max(x+m['attributes']['width'] for n,m,(x,y) in modules);height=max(y+m['attributes']['height'] for n,m,(x,y) in modules)
        whole=Image.new('RGBA',(width*32,height*32));ground=whole.copy();cells=set();module_proof=[]
        for name,module,(x,y) in modules:
            a=module['attributes'];assert (a['time_of_day'] or 'day')=='day';native_objects=objects(module)
            for xx in range(x,x+a['width']):
                for yy in range(y,y+a['height']):assert (xx,yy) not in cells;cells.add((xx,yy))
            bg,scene,masked=render_map(art,module,native_objects,'day',args.output/'modules'/name)
            ground.paste(bg,(x*32,y*32));whole.paste(scene,(x*32,y*32))
            module_proof.append(dict(map=name,alias=a['tileset_name'],dimensions=[a['width'],a['height']],origin_metatiles=[x,y],actors=len(native_objects),foreground_priority_pixels=masked,module_sha256=hashlib.sha256(json.dumps(module,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()))
        assert len(cells)==width*height, 'No invented gap filler in stitched route'
        directory=args.output/f'whole-{index+1}';save_render(ground,directory/'native-terrain-original.png');save_render(whole,directory/'native-scene-original.png')
        comparison=compare(whole,SOURCE[index],directory);wholes.append(whole)
        terrain_comparison=compare(ground,TERRAIN_SOURCE[index],directory/'terrain-comparison')
        receipts.append(dict(title=title,modules=module_proof,source_comparison=comparison,terrain_source_comparison=terrain_comparison,native_scene_sha256=sha(directory/'native-scene-original.png')))
    cap=pack['data']['maps']['MnR11C00R00']['blocks'][20]
    assert cap==1, 'Actual R11 native north cap changed'
    # Secondary context derives from the actual atlas map originals, not a
    # hypothetical terrain bridge between separated source panels.
    context=Image.new('RGB',(512,224));left=Image.open(args.atlas/'maps/MnRainyRiverIcefields/scene-rgb.png').convert('RGB');right=Image.open(args.atlas/'maps/MnR20C00R00/scene-rgb.png').convert('RGB')
    context.paste(left.crop((25*32,17*32,32*32,24*32)),(0,0));context.paste(right.crop((0,17*32,9*32,24*32)),(7*32,0));save_render(context,args.output/'actual-rainy-r20-seam-original.png')
    pad=32;header=276;board=Image.new('RGB',(sum(im.width for im in wholes)+pad*4,max(im.height for im in wholes)+header+48),'#131d27');d=ImageDraw.Draw(board)
    d.text((pad,20),'MINNESOTA WINTER · NATIVE PACK TERRAIN',font=font(46,True),fill='white')
    d.text((pad,82),f'Written pack {PACK_SHA[:16]} · native fixture/input and GPU evidence recorded separately',font=font(28),fill='#e5c891')
    d.text((pad,126),'Three separate whole-area panels. Every whole retains native 32px/metatile scale; one original pixel per board pixel.',font=font(25),fill='#bad0d8')
    d.text((pad,164),'Static native terrain/actor composites, not whole-map GPU captures. Controller fixtures do not prove earned campaign access.',font=font(25),fill='#bad0d8')
    x=pad
    for (title,caption,_),im in zip(groups,wholes):
        d.text((x,216),title,font=font(34,True),fill='white');d.text((x,258),caption,font=font(18),fill='#bad0d8');board.paste(im,(x,header+24));x+=im.width+pad
    y=header+24+wholes[0].height+62
    d.text((pad,y),'Actual Rainy River ↔ R20 seam / native 1× context',font=font(24,True),fill='white');board.paste(context,(pad,y+44))
    d.text((pad,y+44+context.height+18),'Native east/west offset 0; Floor (63,44/45) ↔ (0,44/45).',font=font(21),fill='#bad0d8')
    d.text((pad,y+44+context.height+54),'The snowy region meets the existing green R20 at this real port.',font=font(21),fill='#bad0d8')
    save_render(board,args.output/'board-full-resolution.png');save_render(board.resize((board.width//2,board.height//2),Image.Resampling.NEAREST),args.output/'board-display.png')
    proof=dict(status='ACTUAL_WRITTEN_PACK_NATIVE_TERRAIN_NOT_GPU_CAPTURE',pack=str(args.pack),pack_sha256=PACK_SHA,native_gameplay_and_gpu_verification=dict(sections_geometry_parent='aa1aa251f7383640412b82b9b3ed9d6e0bb1304569230a79624c75274db60e5a', sections_initial_four_pass_one_turn_first_test_failure='.local/winter-sections-controller-v01.log', corrected_cut_pass='.local/winter-cedar-cut-controller-v02.log', region_three_pass='.local/winter-region-controller-v02.log', final_roof_gpu_acceptance='separate Root evidence required; no whole-map GPU claim'), final_source_contract_sha256=sha(source_bundle/'final-source.contract.json'), roof_written_proof=dict(path=str(args.roof_proof),sha256=sha(args.roof_proof)),native_map_count=len(pack['data']['maps']),whole_map_scale='native 32px/metatile; one source pixel per board pixel',panels=receipts,r11_installed_cap=dict(map='MnR11C00R00',metatile=[20,0],block='01',included_in_native_and_reviewed_source=True),
        seam_context=dict(inherited_atlas_pack_sha256=atlas['pack_sha256'],final_pack_maps_and_all_alias_art_exact=True,baseline_to_final_residual_proof_sha256=sha(args.roof_proof),sources=[str(args.atlas/'maps/MnRainyRiverIcefields/scene-rgb.png'),str(args.atlas/'maps/MnR20C00R00/scene-rgb.png')],crop_metatiles=dict(rainy=[25,17,32,24],r20=[0,17,9,24]),sha256=sha(args.output/'actual-rainy-r20-seam-original.png')),
        board_sha256=sha(args.output/'board-full-resolution.png'),limits=['Actors shown at authored coordinates/facings before dynamic movement or event-flag filtering.','Static native art frame, not animation/GPU acceptance.','Terrain and scene differences report every pixel without masks or alignment fitting.'])
    assert sha(args.pack)==PACK_SHA
    (args.output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
    print(json.dumps({'board':str(args.output/'board-display.png'),'source_pixel_changes':[r['source_comparison']['changed_pixels'] for r in receipts]}))


if __name__=='__main__':main()
