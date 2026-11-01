#!/usr/bin/env python3
"""Audit the actual Rust-written private art append and whole playable city."""
import argparse,copy,hashlib,json,sys
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[4];sys.path[:0]=[str(ROOT/'planning/minnesota/tools'),str(Path(__file__).parent)]
from render_reference import SourceArt,layout_blocks,npc_palette_pack
from render_archive import save_render
from review_north_shore import oriented_sprite
from review_prairie_farm import check_water_boundary,negative_boundary_checks
from author import BASE,SHA,TS,CITY

def main():
 import cbor2
 ap=argparse.ArgumentParser();ap.add_argument('--pack',type=Path,required=True);ap.add_argument('--source',type=Path,required=True);ap.add_argument('--epoch',required=True);a=ap.parse_args();out=ROOT/'planning/minnesota/renders/minneapolis-waterfront-v01'/a.epoch;out.mkdir(parents=True,exist_ok=False)
 raw=BASE.read_bytes();assert hashlib.sha256(raw).hexdigest()==SHA;base=cbor2.loads(raw[22:]);written=a.pack.read_bytes();pack=cbor2.loads(written[22:]);spec=json.load(open(a.source/'art.contract.json'));native=json.load(open(a.source/'native.contract.json'));old,new=SourceArt(base),SourceArt(pack);m=pack['data']['maps'][CITY];original=base['data']['maps'][CITY]
 cells={tuple(c['metatile']):c for c in spec['cells']};assert len(cells)==16
 expected=list(original['blocks'])
 for c in cells.values():
  x,y=c['metatile'];assert expected[y*40+x]==c['before'];expected[y*40+x]=c['after']
 assert m['blocks']==expected
 before=copy.deepcopy(base['data']);after=copy.deepcopy(pack['data']);after['maps'][CITY]=before['maps'][CITY];after['tilesets'][TS]=before['tilesets'][TS];label=original['attributes']['blocks_label'];after['map_blocks'][label]=before['map_blocks'][label];assert before==after,'unrelated typed game data changed'
 assert pack['compiled_audio']==base['compiled_audio']and pack['audio_manifest']==base['audio_manifest']and pack['report']==base['report']
 owned={f'gfx/tilesets/{TS}.2bpp',f'gfx/tilesets/{TS}.png',f'data/tilesets/{TS}_metatiles.bin',f'data/tilesets/{TS}_palette_map.json',f'data/tilesets/{TS}.json'}
 assert set(pack['runtime_files'])==set(base['runtime_files'])
 assert {k for k,v in base['runtime_files'].items()if pack['runtime_files'][k]!=v}==owned
 bd,nd=base['data']['tilesets'][TS],pack['data']['tilesets'][TS];assert nd['palette_map'][:224]==bd['palette_map']
 for b,n in zip(original['blocks'],m['blocks']):assert bd['collision'][f'{b:02x}']==nd['collision'][f'{n:02x}']
 for group in ['morn','day','nite','dark']:
  assert all(x.tobytes()==y.tobytes()for x,y in zip(old.blocks(TS,group),new.blocks(TS,group)[:169]))
  for r in spec['new_metatiles']:
   ref=Image.new('RGB',(32,32))
   for i,g in enumerate(r['glyphs']):ref.paste(old.blocks(g['tileset'],group)[g['block']].crop((g['x']*8,g['y']*8,g['x']*8+8,g['y']*8+8)),(i%4*8,i//4*8))
   assert ref.tobytes()==new.blocks(TS,group)[r['target']].tobytes()
  im=new.render(TS,40,32,m['blocks'],group);save_render(im,out/('whole-city-'+group+'.png'))
 assert Image.open(ROOT/'planning/minnesota/renders/minneapolis-waterfront-v01'/a.source.name/'whole-city-candidate.png').convert('RGB').tobytes()==new.render(TS,40,32,m['blocks']).tobytes()
 for group in ['day','nite']:
  im=new.render(TS,40,32,m['blocks'],group).convert('RGBA')
  for o in m['objects']:
   face={'UP':'north','LEFT':'west','RIGHT':'east'}.get(o['spritemovedata'].rsplit('_',1)[-1],'south');im.alpha_composite(oriented_sprite(npc_palette_pack(pack,group),o['sprite'],face),(o['x']*16,o['y']*16))
  save_render(im,out/('whole-city-'+group+'-actors.png'));save_render(im.crop((0,11*32,17*32,25*32)),out/('whole-waterfront-'+group+'.png'))
 # Inspect the actual compiled R02 rather than substituting a newer source
 # branch. This small drainage is deliberately unmountable, unlike a Surf lake.
 from audit_outdoor_connections import collision
 route=pack['data']['maps']['MnR02C00R00'];width=route['attributes']['width']*2;height=route['attributes']['height']*2
 def channel_state(candidate):
  kinds=collision(candidate,candidate['data']['maps']['MnR02C00R00']);water={p for p,k in kinds.items()if k=='WATER'};land={p for p,k in kinds.items()if k in ['FLOOR','TALL_GRASS','DOOR','WARP_CARPET_DOWN','LADDER','STAIRCASE']};pairs={(p,(p[0]+dx,p[1]+dy))for p in land for dx,dy in[(1,0),(-1,0),(0,1),(0,-1)]if(p[0]+dx,p[1]+dy)in water};outer={p for p in water if p[0]in[0,width-1]or p[1]in[0,height-1]};return kinds,water,pairs,outer
 kinds,water,pairs,outer=channel_state(pack)
 assert len(water)==152 and len(outer)==8 and not pairs
 assert all(kinds[x,y]=='WALL'for x in range(28,36)for y in[22,27])
 negatives=[]
 for name,x,y,block in [('north bridge rail',16,11,0x43),('south bridge rail',16,13,0x43),('bank fence',14,5,0x01)]:
  candidate=copy.deepcopy(pack);candidate['data']['maps']['MnR02C00R00']['blocks'][y*30+x]=block;assert channel_state(candidate)[2],('failed native mounting mutation',name);negatives.append(name)
 layout=json.load(open(ROOT/'planning/minnesota/routes/MN_R02.layout.json'));source_blocks=layout_blocks(layout)
 channel=dict(pack_sha256=hashlib.sha256(written).hexdigest(),basis='Actual compiled native R02 module, not a source-branch substitute',water_tiles=len(water),outer_water_tiles=len(outer),all_land_water_mount_pairs=len(pairs),reachable_outer_surf_tiles=0,north_south_bridge_rails_blocking=True,negative_mounting_mutations_rejected=negatives,source_branch_block_differences=sum(x!=y for x,y in zip(source_blocks,route['blocks'])),decision='Retain the fenced channel; no player-reachable water or unbound Surf exit exists.')
 (out/'prairie-channel-qualification.json').write_text(json.dumps(channel,indent=2)+'\n')
 receipt=dict(pack=str(a.pack),sha256=hashlib.sha256(written).hexdigest(),identity=pack['identity']['content_hash'],maps=len(pack['data']['maps']),changed_cells=16,unchanged_other_maps=len(pack['data']['maps'])-1,old_four_bank_metatile_pixels_exact=169*1024*4,new_native_glyph_copies=len(native['glyph_copies']),source_day_whole_city_pixel_match=True,all_5120_city_collision_quadrants_exact=True,all_doors_objects_programs_rooms_species_recovery_report_audio_exact=True,original_palette_entries_exact=224,owned_runtime_files=sorted(owned),prairie_channel_surf_mount_pairs=channel['all_land_water_mount_pairs'],prairie_reachable_outer_water=channel['reachable_outer_surf_tiles'],note='Native publication/static art proof. Actual GPU frames and animation tests recorded separately; no earned progression claim.')
 (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt,indent=2))
if __name__=='__main__':main()
