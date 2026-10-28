#!/usr/bin/env python3
"""Bounded native canopy-only roof fix; source preview, never writes a pack."""
import argparse
import copy
import hashlib
import io
import json
from pathlib import Path
import sys
from PIL import Image,ImageChops,ImageDraw
ROOT=Path(__file__).resolve().parents[5]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from render_reference import load_pack,SourceArt
from render_archive import save_render
from render_progress_atlas import font
from preview_winter_art_v02 import physical
from preview_winter_edges_v01 import remap
from review_major_city import ReviewArt,render_map,player_camera
NAME='mn_winter_north_shore';MAP='MnR11C00R00'


def canonical(x):return json.dumps(x,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
def sha(x):return hashlib.sha256(x).hexdigest()


def derive(base,spec):
    assert spec['schema_version']==1 and spec['base_identity']==base['identity']['content_hash']
    assert sha(canonical(base['data']['maps'][MAP]))==spec['base_module_sha256']
    assert sha(canonical(base['data']['tilesets'][NAME]))==spec['base_definition_sha256']
    for path,expected in spec['source_files_sha256'].items():assert sha(bytes(base['runtime_files'][path]))==expected,path
    output=copy.deepcopy(base);files=output['runtime_files'];definition=output['data']['tilesets'][NAME]
    metapath=f'data/tilesets/{NAME}_metatiles.bin';gfxpath=f'gfx/tilesets/{NAME}.2bpp';fgpath=f'data/tilesets/{NAME}_foregrounds.json'
    oldmeta=bytes(files[metapath]);assert len(oldmeta)==137*16
    graphics=bytearray(files[gfxpath]);assert len(graphics)==4096
    foregrounds=json.loads(bytes(files[fgpath]));tree_colors=foregrounds[str(oldmeta[0x86*16])]
    donor=base['data']['tilesets']['mn_north_shore'];raw=bytes(files['gfx/tilesets/mn_north_shore.2bpp'])
    for row,(source,destination) in zip(spec['glyphs'],[(0x1e,0xfa),(0x1f,0xfb),(0x3e,0xfc),(0x3f,0xfd)]):
        assert row['source']==source and row['target']==destination and destination not in oldmeta
        assert definition['palette_map'][destination]==0 and str(destination)not in foregrounds and graphics[destination*16:(destination+1)*16]==bytes(16)
        assert donor['palette_map'][source]&7==2
        slot=physical(source,donor['palette_map'][source],len(raw)//16)
        glyph=remap(raw[slot*16:(slot+1)*16]);assert sha(glyph)==row['remapped_pixels_sha256']
        graphics[destination*16:(destination+1)*16]=glyph;definition['palette_map'][destination]=11;foregrounds[str(destination)]=tree_colors
    blocks=bytearray(oldmeta);mapping={r['source']:r['target']for r in spec['glyphs']}
    for row,(source,target) in zip(spec['blocks'],[(0x2c,0x89),(0x2d,0x8a)]):
        assert row['source']==source and row['target']==target and len(blocks)==target*16
        original=oldmeta[source*16:(source+1)*16];expected=bytes(mapping[g]if i<8 else g for i,g in enumerate(original));assert list(expected)==row['glyphs']
        assert row['collision']==definition['collision'][f'{source:02x}']==['HEADBUTT_TREE','HEADBUTT_TREE','WALL','WALL']
        blocks.extend(expected);definition['collision'][f'{target:02x}']=row['collision']
    files[metapath]=bytes(blocks);files[gfxpath]=bytes(graphics);files[fgpath]=canonical(foregrounds)
    files[f'data/tilesets/{NAME}.json']=canonical(definition['collision']);files[f'data/tilesets/{NAME}_palette_map.json']=canonical(definition['palette_map'])
    sheet=Image.new('L',(128,128));pixels=sheet.load()
    for i in range(256):
        for y in range(8):
            for x in range(8):
                bit=7-x;level=(graphics[i*16+y*2]>>bit&1)|((graphics[i*16+y*2+1]>>bit&1)<<1);pixels[i%16*8+x,i//16*8+y]=[255,170,85,0][level]
    buf=io.BytesIO();sheet.save(buf,format='PNG');files[f'gfx/tilesets/{NAME}.png']=buf.getvalue()
    module=output['data']['maps'][MAP]
    for row,(x,before,after)in zip(spec['cells'],[(11,0x2c,0x89),(12,0x2d,0x8a)]):
        assert row==dict(metatile=[x,9],before=before,after=after)
        index=9*32+x;assert module['blocks'][index]==before;module['blocks'][index]=after
    assert output['data']['maps'].keys()==base['data']['maps'].keys()
    for name,m in base['data']['maps'].items():
        if name!=MAP:assert output['data']['maps'][name]==m
    for time in ['morn','day','nite','dark']:
        before=SourceArt(base).blocks(NAME,time);after=SourceArt(output).blocks(NAME,time)
        assert len(after)==139
        assert all(a.tobytes()==b.tobytes()for a,b in zip(before,after))
        for original,new in [(0x2c,0x89),(0x2d,0x8a)]:assert before[original].crop((0,16,32,32)).tobytes()==after[new].crop((0,16,32,32)).tobytes()
    return output


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--pack',type=Path,required=True);p.add_argument('--contract',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    raw=args.pack.read_bytes();base=load_pack(args.pack);spec=json.loads(args.contract.read_text());assert sha(raw)==spec['base_pack_sha256'];candidate=derive(base,spec);args.output.mkdir(parents=True,exist_ok=False)
    actual=base['data']['maps'][MAP];changed=candidate['data']['maps'][MAP];objects=[dict(id=o.get('object_identifier')or o['script'],sprite=o['sprite'],tile=[o['x'],o['y']],facing={'UP':'north','DOWN':'south','LEFT':'west','RIGHT':'east'}.get(o['spritemovedata'].rsplit('_',1)[-1],'south'),native_palette_override=o.get('pal',0))for o in actual['objects']]
    frames=[];pixel_changes=[]
    for label,pack,module in [('before',base,actual),('candidate',candidate,changed)]:
        art=ReviewArt(pack);ground,scene,masked=render_map(art,module,objects,'day',args.output/label)
        # Two tiles nearer the house than the expanded production GPU view:
        # this native160x144 detail includes the complete embedded canopy.
        core=[23,22];block=module['blocks'][core[1]//2*32+core[0]//2]
        assert pack['data']['tilesets'][NAME]['collision'][f'{block:02x}'][core[1]%2*2+core[0]%2]=='FLOOR'
        assert all(obj['tile']!=core for obj in objects)
        camera,_=player_camera(art,module,objects,ground,scene,'day',dict(core=core),args.output/label/'camera')
        frames.append(camera)
    contact=Image.new('RGB',(1312,660),'#131d27');d=ImageDraw.Draw(contact);d.text((16,12),'WRITTEN NATIVE BEFORE / SOURCE CANDIDATE · SAME HOUSE, SAME COLLISION',font=font(21,True),fill='white')
    for i,(label,frame)in enumerate(zip(['Actual aa1aa251 roof trees','Candidate snowy roof trees (89/8a)'],frames)):
        d.text((16+i*648,52),label,font=font(20),fill='#c5d8d8');contact.paste(frame.resize((640,576),Image.Resampling.NEAREST),(16+i*648,84))
    save_render(contact,args.output/'roof-canopy-before-candidate.png')
    old=Image.open(args.output/'before/whole-terrain-original.png').convert('RGB');new=Image.open(args.output/'candidate/whole-terrain-original.png').convert('RGB');diff=ImageChops.difference(old,new);save_render(diff,args.output/'whole-terrain-difference.png');changed_pixels=[(i%old.width,i//old.width)for i,(a,b)in enumerate(zip(old.getdata(),new.getdata()))if a!=b]
    assert changed_pixels and all(11*32<=x<13*32 and 9*32<=y<9*32+16 for x,y in changed_pixels)
    proof=dict(status='SOURCE_ONLY_NOT_PUBLISHED',base_pack_sha256=sha(raw),contract_sha256=sha(args.contract.read_bytes()),source_camera_core=[23,22],owned_metatiles=spec['cells'],new_blocks=['89','8a'],new_glyphs=['fa','fb','fc','fd'],unused_remaining=['fe','ff'],all_old_blocks_pixels_exact_all_four_timebanks=True,all_roof_bottom_rows_exact_all_four_timebanks=True,all_other_maps_exact=True,actors_events_programs_same=True,changed_terrain_pixels=len(changed_pixels),changed_pixel_bounds_exclusive=list(diff.getbbox()),candidate_metatiles_sha256=sha(bytes(candidate['runtime_files'][f'data/tilesets/{NAME}_metatiles.bin'])),candidate_graphics_sha256=sha(bytes(candidate['runtime_files'][f'gfx/tilesets/{NAME}.2bpp'])),limits=['Native source-art preview, not an emitted pack or GPU frame.','Only the two top canopy rows change; warm roof timber, walls and door retained.'])
    (args.output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n');assert args.pack.read_bytes()==raw
    print(json.dumps(proof))


if __name__=='__main__':main()
