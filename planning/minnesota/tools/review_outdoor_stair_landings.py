#!/usr/bin/env python3
"""Preview/audit native outdoor stair landing rows without writing game packs."""
import argparse
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw

from render_reference import SourceArt, load_pack
from review_connected_route_native import sources

PLAN = Path(__file__).resolve().parents[1] / 'revisions/outdoor-stair-landings-v01/recipes.json'
MANIFEST = 'minnesota-outdoor-stair-landings-v1'


def preview(parent, recipes):
    """Compose exact native rows in memory for source review only."""
    files = dict(parent['runtime_files'])
    for recipe in recipes:
        ts = recipe['tileset']; stair = int(recipe['stair'], 16)
        landing = int(recipe['outdoor_landing'], 16)
        path = f'data/tilesets/{ts}_metatiles.bin'
        blocks = bytearray(files[path])
        if len(blocks) < (stair+1)*16:
            assert recipe.get('newly_imported') and stair*16 == len(blocks)
            assert not any(m['attributes']['tileset_name']==ts and stair in m['blocks'] for m in parent['data']['maps'].values())
            continue  # Rust must allocate glyphs for the new, currently unused recipe.
        blocks[stair*16:stair*16+8] = blocks[landing*16:landing*16+8]
        files[path] = bytes(blocks)
    return dict(parent, runtime_files=files)


def audit(parent, child, recipes):
    north='mn_north_shore'
    old_north=parent['data']['tilesets'][north];new_north=child['data']['tilesets'][north]
    imported='83' not in old_north['collision']
    assert dict(new_north,collision=old_north['collision'],palette_map=old_north['palette_map']) == old_north
    assert new_north['collision'] == dict(old_north['collision'], **{'83':child['data']['tilesets']['cave']['collision']['36']})
    restored_data=dict(child['data'],tilesets=dict(child['data']['tilesets'],**{north:old_north}))
    assert restored_data == parent['data'], 'gameplay data changed'
    assert child['format_version']==parent['format_version']
    assert set(child['runtime_files']) == set(parent['runtime_files'])
    expected=preview(parent,recipes)
    north_paths={f'data/tilesets/{north}_metatiles.bin',f'data/tilesets/{north}.json',f'data/tilesets/{north}_palette_map.json',f'gfx/tilesets/{north}.png',f'gfx/tilesets/{north}.2bpp'}
    for path,value in expected['runtime_files'].items():
        if path not in north_paths:assert bytes(child['runtime_files'][path])==bytes(value),path
    path=f'data/tilesets/{north}_metatiles.bin';old_raw=bytes(parent['runtime_files'][path]);new_raw=bytes(child['runtime_files'][path])
    assert new_raw[:len(old_raw)]==bytes(expected['runtime_files'][path])
    assert len(new_raw)==len(old_raw)+(16 if imported else 0)
    assert all(old_north['palette_map'][g]==new_north['palette_map'][g] for g in set(old_raw))
    manifests=parent['report']['manifests']+(['minnesota-north-shore-corner-stairs-v1'] if imported else [])+[MANIFEST]
    assert child['report']==dict(parent['report'],manifests=manifests)
    normalized=dict(child,data=parent['data'],runtime_files=parent['runtime_files'],report=parent['report'],identity=parent['identity'])
    assert normalized==parent,'unrelated pack content changed'
    a,b=SourceArt(parent),SourceArt(child);detail=[]
    for ts in sorted({r['tileset'] for r in recipes}):
        before,after=a.blocks(ts),b.blocks(ts)
        assert len(after)==len(before)+(1 if ts==north and imported else 0)
        selected={int(r['stair'],16):r for r in recipes if r['tileset']==ts}
        for i,image in enumerate(before):
            expected_image=image.copy()
            if i in selected:
                landing=int(selected[i]['outdoor_landing'],16)
                expected_image.paste(before[landing].crop((0,0,32,16)),(0,0))
            assert after[i].tobytes()==expected_image.tobytes(),(ts,i)
        for stair,r in selected.items():
            landing=int(r['outdoor_landing'],16);native=int(r['native_stair'],16)
            expected_image=a.blocks('cave')[native].copy()
            expected_image.paste(before[landing].crop((0,0,32,16)),(0,0))
            assert after[stair].tobytes()==expected_image.tobytes(),r
            detail.append(r)
    return dict(status='PASS',gameplay_audio_and_all_other_content_exact=True,native_lower_steps_pixel_exact=True,outdoor_upper_rows_pixel_exact=True,north_shore36_imported=imported,recipes=detail)


def render(pack, recipes, output):
    art = SourceArt(pack); matches = {(r['tileset'], int(r['stair'],16)):r for r in recipes}
    views = []
    for name, module in pack['data']['maps'].items():
        a = module['attributes']; ts = a['tileset_name']
        if not name.startswith('Mn') or a['environment'] not in {'TOWN','ROUTE'}:continue
        found = [(i,b) for i,b in enumerate(module['blocks']) if (ts,b) in matches]
        if not found:continue
        terrain = art.render(ts,a['width'],a['height'],module['blocks'])
        terrain.save(output/(name+'-whole-source.png'))
        for i,b in found:
            x,y = i%a['width'],i//a['width']; ident=f'{name}-{x}-{y}'
            left,top=(x-3)*32,(y-3)*32
            crop=Image.new('RGB',(192,160));border=art.blocks(ts)[a['border_block']]
            for yy in range(0,160,32):
                for xx in range(0,192,32):crop.paste(border,(xx,yy))
            regions=sources(pack,name)
            for neighbor,ox,oy in [*regions[1:],regions[0]]:
                attrs=neighbor['attributes']
                image=art.render(attrs['tileset_name'],attrs['width'],attrs['height'],neighbor['blocks'])
                crop.paste(image,(ox-left,oy-top))
            crop.save(output/(ident+'-original.png'))
            views.append(dict(id=ident,map=name,core=[x*2+int(matches[(ts,b)]['native_stair']=='12'),y*2+1],source_stair=matches[(ts,b)]['native_stair'],metatile=[x,y],crop=crop))
    sheet = Image.new('RGB',(1152,340*((len(views)+2)//3)),'#192027');draw = ImageDraw.Draw(sheet)
    for i,v in enumerate(views):
        x,y=i%3*384,i//3*340
        sheet.paste(v['crop'].resize((384,320),Image.Resampling.NEAREST),(x,y+20));draw.text((x+3,y+3),v['id']+' /'+v['source_stair'],fill='white')
    sheet.save(output/'all-outdoor-joins.png')
    native_views=[{k:v[k] for k in ['id','map','core']} for v in views]
    (output/'native-views.json').write_text(json.dumps(native_views,indent=2)+'\n')
    return dict(placed_stairs=len(views),views=native_views,source_only=True)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--parent',type=Path,required=True);p.add_argument('--pack',type=Path)
    p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    args.output.mkdir(parents=True,exist_ok=False)
    sources=[args.parent,PLAN]+([args.pack] if args.pack else [])
    hashes={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in sources}
    recipes=json.loads(PLAN.read_text())['recipes'];parent=load_pack(args.parent)
    child=load_pack(args.pack) if args.pack else preview(parent,recipes)
    result=audit(parent,child,recipes) if args.pack else dict(status='SOURCE PREVIEW; not a published pack')
    result.update(render(child,recipes,args.output));result['inputs']=hashes
    assert all(hashlib.sha256(Path(p).read_bytes()).hexdigest()==h for p,h in hashes.items())
    (args.output/'receipt.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','placed_stairs','source_only']}),flush=True)


if __name__=='__main__':main()
