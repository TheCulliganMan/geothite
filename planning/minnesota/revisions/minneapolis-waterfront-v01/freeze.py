#!/usr/bin/env python3
"""Freeze exact native glyph copies and prove bank packing without pack writes."""
import argparse,copy,hashlib,io,json,sys
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[4];sys.path[:0]=[str(ROOT/'planning/minnesota/tools'),str(Path(__file__).parent)]
from render_reference import SourceArt
from render_archive import save_render
from author import BASE,SHA,TS,CITY,sha,canonical

def physical(glyph,palette,count):return (glyph&127)+count//2 if palette&8 else glyph

def freeze(source,out):
 import cbor2
 raw=BASE.read_bytes();assert sha(raw)==SHA;p=cbor2.loads(raw[22:]);art=SourceArt(p);spec=json.load(open(source/'art.contract.json'));files=p['runtime_files'];old=bytes(files[f'data/tilesets/{TS}_metatiles.bin']);oldgfx=bytes(files[f'gfx/tilesets/{TS}.2bpp']);assert len(oldgfx)==192*16
 cache={}
 def color(ts,b,i):
  key=ts,b,i
  if key not in cache:cache[key]=b''.join(art.blocks(ts,group)[b].crop((i%4*8,i//4*8,i%4*8+8,i//4*8+8)).tobytes()for group in ['morn','day','nite','dark'])
  return cache[key]
 existing={}
 for b in range(169):
  for i,g in enumerate(old[b*16:(b+1)*16]):existing.setdefault(color(TS,b,i),g)
 copies=[];recipes=[];chosen={}
 for recipe in spec['new_metatiles']:
  ids=[]
  for donor in recipe['glyphs']:
   ts=donor['tileset'];b=donor['block'];i=donor['y']*4+donor['x'];key=color(ts,b,i)
   if key in existing:g=existing[key]
   elif key in chosen:g=chosen[key]
   else:
    g=224+len(copies);assert g<=255 and g not in old
    chosen[key]=g;layout=bytes(files[f'data/tilesets/{ts}_metatiles.bin']);sg=layout[b*16+i];pal=p['data']['tilesets'][ts]['palette_map'][sg];bits=bytes(files[f'gfx/tilesets/{ts}.2bpp']);slot=physical(sg,pal,len(bits)//16);pixels=bits[slot*16:slot*16+16];assert len(pixels)==16
    assert f'gfx/tilesets/{ts}.pal'not in files and f'data/tilesets/{ts}_foregrounds.json'not in files
    copies.append(dict(target=g,source_tileset=ts,source_glyph=sg,source_palette=pal,pixels_sha256=sha(pixels),four_bank_rgb_sha256=sha(key)))
   ids.append(g)
  recipes.append(dict(target=recipe['target'],glyphs=ids,collision=recipe['collision']))
 q=copy.deepcopy(p);d=q['data']['tilesets'][TS];d['palette_map'].extend([0]*32);gfx=bytearray(256*16);gfx[:128*16]=oldgfx[:128*16];gfx[128*16:224*16]=oldgfx[96*16:192*16]
 for g in set(old):
  pal=d['palette_map'][g];before=physical(g,pal,192);after=physical(g,pal,256);assert gfx[after*16:after*16+16]==oldgfx[before*16:before*16+16]
 for c in copies:
  pixels=bytes(files[f"gfx/tilesets/{c['source_tileset']}.2bpp"]);slot=physical(c['source_glyph'],c['source_palette'],len(pixels)//16);gfx[c['target']*16:(c['target']+1)*16]=pixels[slot*16:slot*16+16];d['palette_map'][c['target']]=(c['source_palette']&7)|8
 im=Image.new('L',(128,128));px=im.load()
 for y in range(128):
  for x in range(128):
   n=(y//8)*16+x//8;v=n*16+y%8*2;bit=7-x%8;px[x,y]=[255,170,85,0][((gfx[v]>>bit)&1)|(((gfx[v+1]>>bit)&1)<<1)]
 stream=io.BytesIO();im.save(stream,format='PNG');q['runtime_files'][f'gfx/tilesets/{TS}.png']=stream.getvalue();q['runtime_files'][f'gfx/tilesets/{TS}.2bpp']=bytes(gfx);metas=bytearray(old)
 for r in recipes:
  assert r['target']==len(metas)//16;metas.extend(r['glyphs']);d['collision'][f"{r['target']:02x}"]=r['collision']
 q['runtime_files'][f'data/tilesets/{TS}_metatiles.bin']=bytes(metas);a=SourceArt(q)
 for group in ['morn','day','nite','dark']:
  assert all(x.tobytes()==y.tobytes()for x,y in zip(art.blocks(TS,group),a.blocks(TS,group)[:169]))
  for r,authored in zip(recipes,spec['new_metatiles']):
   expected=Image.new('RGB',(32,32))
   for i,g in enumerate(authored['glyphs']):expected.paste(art.blocks(g['tileset'],group)[g['block']].crop((g['x']*8,g['y']*8,g['x']*8+8,g['y']*8+8)),(i%4*8,i//4*8))
   assert expected.tobytes()==a.blocks(TS,group)[r['target']].tobytes()
 for c in spec['cells']:
  x,y=c['metatile'];assert q['data']['maps'][CITY]['blocks'][y*40+x]==c['before'];assert p['data']['tilesets'][TS]['collision'][f"{c['before']:02x}"]==d['collision'][f"{c['after']:02x}"];q['data']['maps'][CITY]['blocks'][y*40+x]=c['after']
 expected=Image.open(ROOT/'planning/minnesota/renders/minneapolis-waterfront-v01'/source.name/'whole-city-candidate.png').convert('RGB');written=a.render(TS,40,32,q['data']['maps'][CITY]['blocks']);assert expected.tobytes()==written.tobytes()
 save_render(written,ROOT/'planning/minnesota/renders/minneapolis-waterfront-v01'/source.name/'bank-resolved-whole-city.png')
 owned=[f'gfx/tilesets/{TS}.2bpp',f'gfx/tilesets/{TS}.png',f'data/tilesets/{TS}_metatiles.bin',f'data/tilesets/{TS}_palette_map.json',f'data/tilesets/{TS}.json'];inputs=set(owned)|{'gfx/tilesets/bg_tiles.pal'}
 for ts in {c['source_tileset']for c in copies}:inputs.update([f'gfx/tilesets/{ts}.2bpp',f'data/tilesets/{ts}_metatiles.bin'])
 plan={k:spec[k]for k in ['schema_version','id','base_pack_sha256','base_identity','base_module_sha256','base_definition_sha256','map','tileset','expected_blocks','cells']};plan.update(glyph_copies=copies,blocks=recipes,source_files_sha256={path:sha(bytes(files[path]))for path in sorted(inputs)},donor_definition_sha256={ts:sha(canonical(p['data']['tilesets'][ts]))for ts in sorted({c['source_tileset']for c in copies})},source_art_contract_sha256=sha((source/'art.contract.json').read_bytes()))
 assert not out.exists();out.write_text(json.dumps(plan,indent=2)+'\n');print(json.dumps(dict(contract=str(out),sha256=sha(out.read_bytes()),new_glyphs=len(copies),new_recipes=len(recipes),cells=len(spec['cells']),old_metatile_four_bank_pixels_exact=169*1024*4,source_candidate_pixels_exact=1280*1024),indent=2))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--source',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();freeze(a.source,a.output)
