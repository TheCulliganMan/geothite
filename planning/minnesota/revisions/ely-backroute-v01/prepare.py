#!/usr/bin/env python3
"""Explicit source proposal and read-only native-art review; never writes packs."""
import argparse,copy,hashlib,json,sys,cbor2
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import SourceArt,load_pack
from render_archive import save_render
from review_north_shore import oriented_sprite

def sha(b):return hashlib.sha256(b).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--epoch',required=True);args=ap.parse_args()
 out=ROOT/'planning/minnesota/renders/ely-backroute-v01'/args.epoch;out.mkdir(parents=True,exist_ok=False)
 src=ROOT/'planning/minnesota/revisions/ely-backroute-v01'/args.epoch;src.mkdir(exist_ok=False)
 parent=ROOT/'content-packs/minnesota-superior-compact-v03.crystalpack'
 assert sha(parent.read_bytes())=='032304d0fd1d51863a2c3bf2562b20728e22c7535d1c194555621bc8e85b4899','exact compact parent required'
 cache=ROOT/'.local/ely-backroute-base.cbor'
 p=cbor2.load(cache.open('rb'))if cache.exists()else load_pack(parent);q=copy.deepcopy(p)
 # Preserve original Kanto buildings; append the existing snow-covered pine art.
 from snow_art import snow_art
 d,files=snow_art(p);q['data']['tilesets']['mn_ely_snow']=d;q['runtime_files'].update(files)
 # Source rows retain every Ely door, actor, service and north port.
 ely=q['data']['maps']['MnEly'];ely['attributes']['tileset_name']='mn_ely_snow';ely['attributes']['border_block']=0x83
 patches={}
 def patch(n,x,y,b):
  m=q['data']['maps'][n];i=y*m['attributes']['width']+x;old=p['data']['maps'][n]['blocks'][i]
  if old!=b:patches.setdefault(n,[]).append({'position':[x,y],'before':old,'after':b});m['blocks'][i]=b
 # Full ordinary pine forest replaces rounded canopy, including two incidental
 # decorative Cut quadrants; no town reward/shortcut was sealed by those trees.
 for i,b in enumerate(p['data']['maps']['MnEly']['blocks']):
  if b in [0x0f,0x32,0x33]:patch('MnEly',i%20,i//20,0x83)
 # Eastern town lane leaves from the existing League apron without crossing a door.
 for x in range(16,20):patch('MnEly',x,7,0x31)
 # Southern Split Rock rest stop: whole authentic two-column Johto clinic.
 shore='MnR11C00R02'
 for y in range(15,19):
  for x in range(15,19):patch(shore,x,y,1)
 for x,y,b in [(16,15,0x18),(17,15,0x19),(16,16,0x1a),(17,16,0x1b),(15,17,0x45)]:patch(shore,x,y,b)
 # West approach bends at a small grassy knoll, joins the current through trail.
 for x in range(0,16):patch(shore,x,15,1)
 for x,y in [(5,15),(6,15),(7,15),(9,15),(10,15)]:patch(shore,x,y,3)
 # A deliberate open moss shoulder and irregular attached pine edges.
 for x,y,b in [(3,14,2),(4,14,2),(5,14,3),(6,14,3),(7,14,3),(8,14,5),(9,14,3),(10,14,3),(11,14,2),(12,14,2),(13,14,1),(14,14,1)]:patch(shore,x,y,b)
 # Seven-metatile pass exactly fills the real gap. West snow, east warm coast.
 rows=[
 '8b 8b 8b 8b 05 05 05',
 '8b 8b 8b 8b 05 05 05',
 '8b 8b 8b 8b 05 05 05',
 '8b 8b 8b 8b 05 05 05',
 '8b 8b 83 83 02 05 05',
 '8b 8b 83 8b 03 01 01',
 '8b 83 83 8b 03 05 05',
 '83 83 8b 8b 02 05 05',
 '8b 8b 8b 8b 05 05 05',
 '8b 8b 8b 8b 05 05 05',
 '8b 8b 8b 8b 05 05 05',
 '8b 8b 8b 8b 05 05 05']
 # Link upper snow dogleg to lower coastal grass bend through explicit row4/5.
 rows[4]='8b 8b 83 83 02 05 05'
 blocks=[int(v,16)for row in rows for v in row.split()]
 module=copy.deepcopy(p['data']['maps']['MnR11C00R00']);module['attributes'].update(width=7,height=12,tileset_name='mn_winter_north_shore');module['blocks']=blocks;module['objects']=[]
 q['data']['maps']['MnElyShorePass']=module
 # Publish one immutable-parent edit per cell after all authored layers.
 patches={n:[{'position':[i%m['attributes']['width'],i//m['attributes']['width']],
              'before':p['data']['maps'][n]['blocks'][i],'after':b}
             for i,b in enumerate(m['blocks'])if b!=p['data']['maps'][n]['blocks'][i]]
          for n,m in q['data']['maps'].items()if n in ['MnEly',shore]}
 spec={'schema_version':1,'id':'MN_ELY_SHORE_PASS_V1','base_content_hash':p['identity']['content_hash'],'base_file_sha256':'032304d0fd1d51863a2c3bf2562b20728e22c7535d1c194555621bc8e85b4899',
 'module_preimages':{n:sha(json.dumps(p['data']['maps'][n],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())for n in ['MnEly',shore]},
 'patches':patches,'pass':{'id':'MnElyShorePass','constant':'MN_ELY_SHORE_PASS','tileset':'mn_winter_north_shore','dimensions_metatiles':[7,12],'origin_metatiles':[183,-342],'rows':rows,'west_port_core':[[0,14],[0,15]],'east_port_core':[[13,10],[13,11]],'landmark':'LANDMARK_MN_LEAGUE_APPROACH'},
 'clinic':{'id':'MnSplitRockPokecenter1F','constant':'MN_SPLIT_ROCK_POKECENTER_1F','exterior':shore,'entrance':[33,33],'return':[33,34],'arrival':[3,5],'nurse':[3,1],'pc':[8,0]},
 'connections':[
 {'map':'MnEly','direction':'east','target':'MnElyShorePass','offset':0},
 {'map':'MnElyShorePass','direction':'west','target':'MnEly','offset':0},
 {'map':'MnElyShorePass','direction':'east','target':shore,'offset':-10},
 {'map':shore,'direction':'west','target':'MnElyShorePass','offset':10}],
 'weather':{'map':'MnEly','kind':'snow','outdoors_only':True,'private_tileset':'mn_ely_snow'},
 'story':'Public rest stop and two-way wooded back route. No League registration, badge, survey or existing Cut gate is changed.'}
 (src/'native.contract.json').write_text(json.dumps(spec,indent=2)+'\n')
 # Static actor-aware ordinary footing, including connected seams and clinic approach.
 def kind(n,xy):
  m=q['data']['maps'][n];x,y=xy;w=m['attributes']['width'];h=m['attributes']['height']
  if not(0<=x<2*w and 0<=y<2*h):return 'OUTSIDE'
  b=m['blocks'][y//2*w+x//2];return q['data']['tilesets'][m['attributes']['tileset_name']]['collision'][f'{b:02x}'][y%2*2+x%2]
 from collections import deque
 def path(n,start,end):
  assert kind(n,start) in ['FLOOR','TALL_GRASS'] and kind(n,end) in ['FLOOR','TALL_GRASS'],('illegal walking anchor',n,start,end,kind(n,start),kind(n,end))
  occupied={(o['x'],o['y'])for o in q['data']['maps'][n]['objects']};todo=deque([start]);prev={start:None}
  while todo:
   a=todo.popleft()
   if a==end:
    result=[]
    while a is not None:result.append(list(a));a=prev[a]
    return result[::-1]
   for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]:
    b=(a[0]+dx,a[1]+dy)
    if b not in prev and b not in occupied and kind(n,b) in ['FLOOR','TALL_GRASS']:prev[b]=a;todo.append(b)
  raise AssertionError((n,start,end,'disconnected'))
 checks={
 'ely_apron_to_east':path('MnEly',(27,14),(39,14)),
 'ely_apron_to_clinic':path('MnEly',(27,14),(11,22)),
 'ely_apron_to_mart':path('MnEly',(27,14),(27,28)),
 'ely_apron_to_north':path('MnEly',(27,14),(24,0)),
 'pass_west_to_east':path('MnElyShorePass',(0,14),(13,10)),
 'shore_backroad_to_clinic':path(shore,(0,30),(33,34)),
 'clinic_to_current_trail':path(shore,(33,34),(36,32))}
 for v in patches[shore]:
  tokens=p['data']['tilesets'][p['data']['maps'][shore]['attributes']['tileset_name']]['collision'][f"{v['before']:02x}"]
  assert set(tokens)<=set(['FLOOR','TALL_GRASS','WALL']),('special terrain would change',v,tokens)
 checks['shore_bridge_to_current_trail']=path(shore,(34,28),(36,32))
 checks['clinic_to_existing_southern_lane']=path(shore,(33,34),(36,47))
 for n in ['MnEly',shore]:
  for o in p['data']['maps'][n]['objects']:assert kind(n,(o['x'],o['y']))==p['data']['tilesets'][p['data']['maps'][n]['attributes']['tileset_name']]['collision'][f"{p['data']['maps'][n]['blocks'][o['y']//2*p['data']['maps'][n]['attributes']['width']+o['x']//2]:02x}"][o['y']%2*2+o['x']%2]
 (src/'input-plan.json').write_text(json.dumps(checks,indent=2)+'\n')
 art=SourceArt(q);scenes={}
 for n in ['MnEly','MnElyShorePass',shore]:
  m=q['data']['maps'][n];a=m['attributes'];im=art.render(a['tileset_name'],a['width'],a['height'],m['blocks'],'day').convert('RGBA')
  for o in m['objects']:im.alpha_composite(oriented_sprite(q,o['sprite'],'south'),(o['x']*16,o['y']*16))
  scenes[n]=im;save_render(im,out/(n+'-whole.png'))
 # Actual equal-scale adjacency, keeping true map origins and exposed gaps visible.
 from connected import connected
 atlas=connected(q,art)
 save_render(atlas,out/'connected-whole.png')
 save_render(atlas.resize((1024,544),Image.Resampling.NEAREST),out/'connected-review.png')
 (src/'prepare.py.snapshot').write_bytes(Path(__file__).read_bytes())
 print(json.dumps({'output':str(out),'source':str(src),'checks':{k:len(v)-1 for k,v in checks.items()},'patches':{k:len(v)for k,v in patches.items()}},indent=2))
if __name__=='__main__':main()
