#!/usr/bin/env python3
"""Render a hash-bound frozen candidate; never mutate source or a native pack."""
from pathlib import Path
import hashlib,json,sys
ROOT=Path(__file__).resolve().parents[4];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack,layout_blocks,room_sheet
from review_major_city import ReviewArt,render_map,player_camera
src=Path(sys.argv[1]);out=Path(sys.argv[2]);assert not out.exists();out.mkdir(parents=True)
c=json.loads((src/'native.contract.json').read_text())
for name in ['MN_R12.candidate.layout.json','MN_R12.candidate.json']:assert hashlib.sha256((src/name).read_bytes()).hexdigest()==c['sources_sha256'][name]
p=load_pack(ROOT/'content-packs/minnesota-superior-inland-winter-v01.crystalpack');l=json.loads((src/'MN_R12.candidate.layout.json').read_text());d=json.loads((src/'MN_R12.candidate.json').read_text());a=ReviewArt(p);b=layout_blocks(l)
recipes=[r for r in l['derived_blocks']if int(r['id'],16)>=len(a.art.blocks(l['compiled_tileset']))]
for r in recipes:r.setdefault('collision',p['data']['tilesets'][r['source_tileset']]['collision'][r['source_block']])
m=dict(id='MnR12CompactReview',attributes=dict(p['data']['maps']['MnR12C00R00']['attributes'],width=32,height=32),blocks=b)
objects=[dict(id=o['id'],tile=o['tile'],sprite=o.get('sprite','SPRITE_POKE_BALL'),facing=o.get('facing','south'))for f in ['residents','trainers','items']for o in d[f]]
ground,scene,_=render_map(a,m,objects,'day',out/'whole',recipes);scene.save(out/'whole-display.png')
views=[('east-arrival',[59,54]),('ranger-shore',[49,49]),('loons-east-bank',[50,28]),('lake-west-bank',[23,27]),('frost-entry',[38,17]),('frost-rim',[55,19]),('raven-shelter',[40,16]),('revive-forecourt',[36,17]),('cut-approach',[14,18]),('bog-neck',[22,49]),('hammock',[23,54]),('west-exit',[3,28])]
frames=[]
for name,at in views:
 frame,_=player_camera(a,m,objects,ground,scene,'day',dict(core=at),out/'cameras'/name,recipes);frames.append((name,frame))
room_sheet(frames,3).save(out/'player-contact.png')
(out/'receipt.json').write_text(json.dumps(dict(status='FROZEN_SOURCE_RENDER_NOT_NATIVE_GPU',source_contract_sha256=hashlib.sha256((src/'native.contract.json').read_bytes()).hexdigest(),views=views,period='day',facing='south'),indent=2)+'\n')
print(out)
