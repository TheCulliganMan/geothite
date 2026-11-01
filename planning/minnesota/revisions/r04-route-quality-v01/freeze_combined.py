"""Freeze the approved source-only 57-cell park proposal; never write a pack."""
from copy import deepcopy
from pathlib import Path
import hashlib,json,sys,cbor2
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import layout_blocks,room_sheet
from review_major_city import ReviewArt,render_map,player_camera
from render_archive import save_render
sha=lambda b:hashlib.sha256(b).hexdigest()
canonical=lambda v:json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
write=lambda p,v:p.write_text(json.dumps(v,indent=2,ensure_ascii=False)+'\n')
base_path=ROOT/'content-packs/minnesota-river-long-v02.crystalpack'
raw=base_path.read_bytes();assert sha(raw)=='3ddbad16c6538e52f219c12497e4fbe6e41a7f18af5c42589b7b92357cfad243'
base=cbor2.loads(raw[22:]);assert base['identity']['content_hash']=='3ead916d12c705f2c0559c98b983f5af4760c9a8f557d8e31f62b36ea197e29a'
MAP='MnR04C00R00';TS='mn_river_park';TEXT='MN_R04_BRIDGE_SIGNText'
m=base['data']['maps'][MAP];assert sha(canonical(m))=='ca7e08ba7ec44571661298e28ab8d21560f5287eccfee5d5797983f68d47d6ca'
folder=ROOT/'planning/minnesota/revisions/r04-route-quality-v01/source-v05'
old=ROOT/'planning/minnesota/connections/river-continuity-r04-v01/source-v04'
oldcontract=json.loads((old/'candidate.contract.json').read_bytes())
layout=json.loads((old/'MN_R04.candidate.layout.json').read_bytes())
dossier=json.loads((old/'MN_R04.candidate.json').read_bytes())
proposal=json.loads((ROOT/'planning/minnesota/renders/r04-route-quality-v01/audit-v08/proposal-only/proposal.json').read_bytes())
changes=list(oldcontract['changes'])
for patch in proposal['patches']:
 x,y=patch['metatile'];layout['layers'].append(dict(kind='block_rect',name='Approved elm shoulder: '+patch['purpose'],bounds=[x,y,x+1,y+1],block=patch['after']))
 changes.append(dict(at_meta=[x,y],before=int(patch['before'],16),after=int(patch['after'],16)))
blocks=layout_blocks(layout);assert len(changes)==57 and len({tuple(c['at_meta'])for c in changes})==57
assert sum(a!=b for a,b in zip(m['blocks'],blocks))==57
for change in changes:
 x,y=change['at_meta'];assert m['blocks'][y*28+x]==change['before'] and blocks[y*28+x]==change['after']
dossier['status']='Source composition accepted for bounded native integration; implementation, GPU and traversal acceptance pending.'
dossier['anchor_status']='Combined source-v05: 57 native cells (45 pond closure, 6 bridge rails, 6 elm shoulders), four old-glyph recipes, existing bridge sign first line only. Exact long-river native parent; source composition approved, native acceptance pending.'
write(folder/'MN_R04.candidate.layout.json',layout);write(folder/'MN_R04.candidate.json',dossier)
write(folder/'compiled-candidate.blocks.json',dict(map=MAP,dimensions_metatiles=[28,18],tileset=TS,blocks=blocks))
preview=deepcopy(base);rpath=f'data/tilesets/{TS}_metatiles.bin';oldraw=bytes(base['runtime_files'][rpath]);assert len(oldraw)==68*16
newraw=bytearray(oldraw);recipes=[]
for recipe in oldcontract['native_art_recipes']:
 target=int(recipe['id'],16);assert len(newraw)==target*16
 glyphs=[];sources=[]
 for row in recipe['tile_row_sources']:
  b=int(row['source_block'],16);r=row['source_row'];glyphs.extend(oldraw[b*16+r*4:b*16+r*4+4]);sources.append(dict(block=b,row=r))
 newraw.extend(glyphs)
 recipes.append(dict(target=target,glyphs=glyphs,collision=recipe['collision'],row_sources=sources))
 preview['data']['tilesets'][TS]['collision'][recipe['id']]=recipe['collision']
preview['runtime_files'][rpath]=bytes(newraw)
label=m['attributes']['blocks_label'];signraw=base['data']['map_scripts'][TEXT]
assert signraw==m['scripts'][TEXT]
assert signraw[0]==dict(args=['"MISSISSIPPI BRIDGE"'],command='text')
assert m['script_text_bodies'][TEXT]['commands'][0]==dict(command='text',args=['"MISSISSIPPI BRIDGE"'],command_index=0)
files={k:sha(bytes(v))for k,v in base['runtime_files'].items()if TS in k}
contract=dict(schema_version=1,base_pack_sha256=sha(raw),base_identity=base['identity']['content_hash'],base_map_count=len(base['data']['maps']),map=MAP,tileset=TS,expected_blocks=68,base_module_sha256=sha(canonical(m)),base_definition_sha256=sha(canonical(base['data']['tilesets'][TS])),source_files_sha256=files,blocks=recipes,cells=[dict(metatile=c['at_meta'],before=c['before'],after=c['after'])for c in changes],sign_text_label=TEXT,sign_before='"MISSISSIPPI BRIDGE"',sign_after='"PARK POND BRIDGE"',base_sign_raw_sha256=sha(canonical(signraw)),base_sign_body_sha256=sha(canonical(m['script_text_bodies'][TEXT])))
write(folder/'native.contract.json',contract)
preimage=dict(status='ACTUAL_LONG_PARENT_VERIFIED_SOURCE_ONLY',base_pack=str(base_path.relative_to(ROOT)),base_pack_sha256=sha(raw),base_identity=base['identity']['content_hash'],module_sha256=sha(canonical(m)),tileset_definition_sha256=sha(canonical(base['data']['tilesets'][TS])),native_blocks_sha256=sha(bytes(m['blocks'])),block_label=label,block_mirror_exact=True,alias_file_sha256=files,old_pond_contract_sha256=sha((old/'candidate.contract.json').read_bytes()),approved_shoulder_proof_sha256=sha((ROOT/'planning/minnesota/renders/r04-route-quality-v01/audit-v08/proposal-only/proposal.json').read_bytes()),frozen_source_hashes={p.name:sha(p.read_bytes())for p in [folder/n for n in ['MN_R04.candidate.layout.json','MN_R04.candidate.json','compiled-candidate.blocks.json','native.contract.json']]},source_or_pack_writes=0)
import base64
assert base64.b64decode(base['data']['map_blocks'][label])==bytes(m['blocks'])
write(folder/'native-preimage.json',preimage)
out=ROOT/'planning/minnesota/renders/r04-route-quality-v01/combined-source-v05-v02';out.mkdir(exist_ok=False)
objects=[dict(id=o.get('object_identifier')or o['script'],sprite=o['sprite'],tile=[o['x'],o['y']],facing={'UP':'north','DOWN':'south','LEFT':'west','RIGHT':'east'}.get(o['spritemovedata'].rsplit('_',1)[-1],'south'),native_palette_override=o.get('pal',0))for o in m['objects']]
frames=[];wholes=[]
for name,g in [('actual-long-parent',m['blocks']),('combined-source-v05',blocks)]:
 art=ReviewArt(preview);local=dict(m,blocks=g);ground,scene,_=render_map(art,local,objects,'day',out/name);wholes.append((name,scene))
 for view,core in [('east-apron',[40,18]),('east-bend',[46,20]),('east-return',[50,18]),('west-choice',[15,11]),('picnic-cache',[48,27]),('pond-bank',[23,10])]:
  frame,_=player_camera(art,local,objects,ground,scene,'day',dict(core=core),out/name/'views'/view);frames.append((name+' / '+view,frame))
save_render(room_sheet(wholes,2),out/'whole-before-combined.png');save_render(room_sheet(frames,3),out/'players-before-combined.png')
write(out/'source-receipt.json',dict(native_parent_sha256=sha(raw),native_contract_sha256=sha((folder/'native.contract.json').read_bytes()),changed_metatiles=57,new_recipes=4,new_glyphs=0,source_only=True,pack_written=False,production_gpu=False,source_contract_hashes=preimage['frozen_source_hashes']))
print('native contract',sha((folder/'native.contract.json').read_bytes()));print('source',folder);print('render',out)
