#!/usr/bin/env python3
"""Native-glyph waterfront frontage study; no pack writes or canonical source edits."""
import argparse,copy,hashlib,json,sys,time
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[4];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import SourceArt
from render_archive import save_render
from review_north_shore import oriented_sprite
BASE=ROOT/'content-packs/minnesota-ely-pine-knolls-v02.crystalpack'
SHA='1f5604a6c743904e5b0c8fbfe36b32ef056f50dab309a42795aa639457ac0bf3'
CITY='MnMinneapolis';TS='mn_minneapolis_city'

def sha(x):return hashlib.sha256(x).hexdigest()
def canonical(x):return json.dumps(x,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
def glyph(ts,b,x,y):return dict(tileset=ts,block=b,x=x,y=y)
def block(ts,b):return[glyph(ts,b,x,y)for y in range(4)for x in range(4)]
def quadrant(target,donor,tx,ty,sx,sy):
 for y in range(2):
  for x in range(2):target[(ty*2+y)*4+tx*2+x]=donor[(sy*2+y)*4+sx*2+x]

def design():
 # Keep each existing footprint and its exact door quadrant. This is three
 # coherent waterfront forms and one small brick home with a pitched roof.
 boat_roof=[block(TS,b)for b in[0x80,0x82,0x82,0x81]]
 # Complete authentic Pallet family house, whose native door exactly matches
 # the dispatcher's fixed lower-right quadrant in the left wall block.
 dispatcher=[[block('kanto',0x38),block('kanto',0x39)],
             [block('kanto',0x3c),block('kanto',0x3d)]]
 # The angler retains its accepted brick lower facade; only the roof slopes.
 angler=[[block('kanto',0x38),block('kanto',0x39)],None]
 cabin_ground=[block('johto',0x26),block('johto',0x27),block('johto',0x2f)]
 # Remove the original middle doorway before placing the only real entrance.
 quadrant(cabin_ground[1],block('johto',0x2f),0,1,0,1)
 quadrant(cabin_ground[1],block('johto',0x2e),1,1,1,1)
 cabin=[[block('johto',b)for b in[0x2c,0x2a,0x2d]],cabin_ground]
 return [dict(id='BoatHouse',origin=[2,12],rows=[boat_roof,None,None],door=[9,29],camera=[9,30],role='Slate-roof waterfront meeting house; existing joined brick front and host, no rental service'),dict(id='AnglerHome',origin=[8,13],rows=angler,door=[19,29],camera=[19,30],role='Small inhabited brick angler home; existing fishing advice'),dict(id='DispatcherHouse',origin=[11,19],rows=dispatcher,door=[23,41],camera=[23,42],role='Compact weatherboard family home; rescue witness remains'),dict(id='LakeCabin',origin=[2,21],rows=cabin,door=[7,45],camera=[7,46],role='Low timber lake cabin; existing host, no new inn service')]

def render(pack,source,out):
 began=time.monotonic();out.mkdir(exist_ok=False,parents=True);source.mkdir(exist_ok=False,parents=True)
 art=SourceArt(pack);m=pack['data']['maps'][CITY];old=list(m['blocks']);definition=pack['data']['tilesets'][TS];assert len(definition['collision'])==169
 recipes=[];cells=[];fronts=design()
 def add(glyphs,collision):
  donor=glyphs[0]['block']
  if glyphs==block(TS,donor)and definition['collision'][f'{donor:02x}']==collision:return donor
  for r in recipes:
   if r['glyphs']==glyphs and r['collision']==collision:return r['target']
  n=169+len(recipes);recipes.append(dict(target=n,glyphs=glyphs,collision=collision));return n
 for house in fronts:
  ox,oy=house['origin']
  for y,row in enumerate(house['rows']):
   if row is None:continue
   for x,g in enumerate(row):
    before=old[(oy+y)*40+ox+x];n=add(g,definition['collision'][f'{before:02x}'])
    cells.append(dict(metatile=[ox+x,oy+y],before=before,after=n))
 assert len({tuple(c['metatile'])for c in cells})==len(cells)
 # Work directly from original native pixels in every palette bank.
 def candidate(group):
  images={}
  for r in recipes:
   im=Image.new('RGB',(32,32))
   for i,g in enumerate(r['glyphs']):im.paste(art.blocks(g['tileset'],group)[g['block']].crop((g['x']*8,g['y']*8,g['x']*8+8,g['y']*8+8)),(i%4*8,i//4*8))
   images[r['target']]=im
  im=art.render(TS,40,32,old,group)
  for c in cells:x,y=c['metatile'];im.paste(images[c['after']]if c['after']in images else art.blocks(TS,group)[c['after']],(x*32,y*32))
  return im
 before=art.render(TS,40,32,old);after=candidate('day');save_render(before,out/'whole-city-before.png');save_render(after,out/'whole-city-candidate.png')
 # One complete waterfront composition, without hiding the unchanged streets,
 # river, lake, rails, homes or the remaining city's older architecture.
 bounds=(0,11*32,17*32,25*32)
 paired=Image.new('RGB',(1088,472),'#17232c');draw=ImageDraw.Draw(paired)
 for i,(label,im)in enumerate([('Published native before',before),('Waterfront candidate',after)]):paired.paste(im.crop(bounds),(i*544,24));draw.text((i*544+8,6),label,fill='white')
 save_render(paired,out/'waterfront-before-candidate.png')
 banks=Image.new('RGB',(4*240,4*228),'#17232c');d=ImageDraw.Draw(banks)
 for col,group in enumerate(['morn','day','nite','dark']):
  whole=candidate(group)
  for row,h in enumerate(fronts):
   ox,oy=h['origin'];width=4 if h['id']=='BoatHouse'else 3 if h['id']=='LakeCabin'else 2;height=3 if h['id']=='BoatHouse'else 2
   crop=whole.crop((ox*32,oy*32,(ox+width)*32,(oy+height)*32));save_render(crop,out/f'{h["id"]}-{group}.png');banks.paste(crop.resize((crop.width*2,crop.height*2),Image.Resampling.NEAREST),(col*240+8,row*228+24));d.text((col*240+8,row*228+7),group+': '+h['id'],fill='white')
 save_render(banks,out/'four-native-palette-banks.png')
 # Actor-aware whole and actual passive-return player cameras.
 for tag,im in [('before',before),('candidate',after)]:
  scene=im.convert('RGBA')
  for o in m['objects']:
   face={'UP':'north','LEFT':'west','RIGHT':'east'}.get(o['spritemovedata'].rsplit('_',1)[-1],'south');scene.alpha_composite(oriented_sprite(pack,o['sprite'],face),(o['x']*16,o['y']*16))
  save_render(scene,out/('whole-city-'+tag+'-actors.png'))
  for h in fronts:
   x,y=h['camera'];v=scene.copy();v.alpha_composite(oriented_sprite(pack,'SPRITE_CHRIS','north'),(x*16,y*16));left=min(max(x*16-80,0),1280-160);top=min(max(y*16-72,0),1024-144);save_render(v.crop((left,top,left+160,top+144)),out/(h['id']+'-'+tag+'-native160.png'))
 spec=dict(schema_version=1,id='MINNEAPOLIS_WATERFRONT_V1',base_pack_sha256=SHA,base_identity=pack['identity']['content_hash'],base_module_sha256=sha(canonical(m)),base_definition_sha256=sha(canonical(definition)),map=CITY,tileset=TS,expected_blocks=169,new_metatiles=recipes,cells=cells,fronts=[{k:v for k,v in h.items()if k!='rows'}for h in fronts])
 (source/'art.contract.json').write_text(json.dumps(spec,indent=2)+'\n');(source/'author.py.snapshot').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(source=str(source),renders=str(out),recipes=len(recipes),cells=len(cells),warm_seconds=round(time.monotonic()-began,3)),indent=2))

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--epoch',required=True);a=ap.parse_args();raw=BASE.read_bytes();assert sha(raw)==SHA
 import cbor2
 p=cbor2.loads(raw[22:]);render(p,Path(__file__).parent/a.epoch,ROOT/'planning/minnesota/renders/minneapolis-waterfront-v01'/a.epoch)
if __name__=='__main__':main()
