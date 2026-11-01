#!/usr/bin/env python3
"""Author the pine-knoll approach over the immutable compact-coast parent."""
import argparse,copy,hashlib,json,sys,time
from collections import deque
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[4]
sys.path[:0]=[str(ROOT/'planning/minnesota/tools'),str(ROOT/'planning/minnesota/revisions/ely-backroute-v01')]
from render_reference import SourceArt,load_pack
from render_archive import save_render
from snow_art import snow_art
from connected import connected


def edges(pack,name,at):
    """Model native current-cell ledge authorization and its two-tile landing."""
    m=pack['data']['maps'][name];w=m['attributes']['width'];h=m['attributes']['height']
    collision=pack['data']['tilesets'][m['attributes']['tileset_name']]['collision']
    occupied={(o['x'],o['y'])for o in m['objects']}
    def kind(at):
        x,y=at
        if not(0<=x<2*w and 0<=y<2*h):return 'OUTSIDE'
        return collision[f'{m["blocks"][y//2*w+x//2]:02x}'][y%2*2+x%2]
    k=kind(at)
    for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]:
        jump=(k,dx,dy)in [('HOP_LEFT',-1,0),('HOP_RIGHT',1,0),('HOP_DOWN',0,1)]
        target=(at[0]+dx*(2 if jump else 1),at[1]+dy*(2 if jump else 1))
        tk=kind(target)
        if target not in occupied and (tk in ['FLOOR','TALL_GRASS'] or tk=='HOP_LEFT' and dx==-1):
            yield target,tk,('left' if dx<0 else 'right' if dx else 'down' if dy>0 else 'up'),jump


def course(pack,start,end,grass_cost=False):
    import heapq
    costs={start:(0,0)};prev={start:None};queue=[((0,0),start)]
    while queue:
        distance,at=heapq.heappop(queue)
        if distance!=costs[at]:continue
        for target,k,button,jump in edges(pack,'MnR11C00R02',at):
            cost=(distance[0]+int(grass_cost and k=='TALL_GRASS'),distance[1]+1)
            if cost<costs.get(target,(10**9,10**9)):
                costs[target]=cost;prev[target]=(at,button,jump,k)
                heapq.heappush(queue,(cost,target))
    assert end in costs,('disconnected',start,end)
    points=[];at=end
    while at!=start:
        old,button,jump,kind=prev[at];points.append(dict(from_core=list(old),to_core=list(at),button=button,jump=jump,terrain=kind));at=old
    return dict(cost=costs[end][0 if grass_cost else 1],steps=points[::-1])



def main():
    parser=argparse.ArgumentParser();parser.add_argument('--epoch',required=True);args=parser.parse_args();began=time.monotonic()
    source=Path(__file__).parent/args.epoch;source.mkdir(exist_ok=False)
    out=ROOT/'planning/minnesota/renders/ely-trail-quality-v02'/args.epoch;out.mkdir(parents=True,exist_ok=False)
    import cbor2
    parent=ROOT/'content-packs/minnesota-superior-compact-v03.crystalpack'
    assert hashlib.sha256(parent.read_bytes()).hexdigest()=='032304d0fd1d51863a2c3bf2562b20728e22c7535d1c194555621bc8e85b4899'
    cache=ROOT/'.local/ely-backroute-base.cbor'
    base=cbor2.load(cache.open('rb'))if cache.exists()else load_pack(parent)
    spec=json.load(open(ROOT/'planning/minnesota/revisions/ely-backroute-v01/source-v08/native.contract.json'))
    pack=copy.deepcopy(base)
    art,files=snow_art(base);pack['data']['tilesets']['mn_ely_snow']=art;pack['runtime_files'].update(files)
    for name,patches in spec['patches'].items():
        m=pack['data']['maps'][name];w=m['attributes']['width']
        for patch in patches:
            x,y=patch['position'];m['blocks'][y*w+x]=patch['after']
    pack['data']['maps']['MnEly']['attributes'].update(tileset_name='mn_ely_snow',border_block=0x83)
    shore=pack['data']['maps']['MnR11C00R02'];w=shore['attributes']['width']
    # Remove the former straight approach; keep the clinic and stream unchanged.
    for y in range(12,20):
        for x in range(15):shore['blocks'][y*w+x]=base['data']['maps']['MnR11C00R02']['blocks'][y*w+x]
    cells={}
    def put(x,y,b):cells[x,y]=b
    # Two-walking-tile bends through two pine knolls. No empty broad shoulder.
    for x in range(4):put(x,15,1)
    put(2,14,3)
    for x in range(2,6):put(x,13,3 if x in [3,4]else 1)
    for x in range(5,9):put(x,14,3 if x in [5,6,7]else 1)
    for x in range(8,12):put(x,15,3 if x==10 else 1)
    for x in range(11,16):put(x,16,3 if x==12 else 1)
    put(14,17,1)
    # Dry westbound shelf. The right half of 4c is the native HOP_LEFT lip;
    # its left half blocks reversal. Solid pines prevent a lower/east bypass.
    for x in range(3,11):put(x,17,1)
    put(3,16,1);put(11,17,0x4c)
    for x in range(12,16):put(x,17,1)
    # Deliberate basalt knoll, flower verge, grass bays; no incidental Cut tree.
    put(7,16,0x0a);put(9,17,4)
    for (x,y),b in cells.items():shore['blocks'][y*w+x]=b
    # Reuse the reviewed gap geometry and guide, preserving all actual seam lanes.
    m=copy.deepcopy(base['data']['maps']['MnR11C00R00']);m['attributes'].update(width=7,height=12,tileset_name='mn_winter_north_shore');m['blocks']=[int(v,16)for row in spec['pass']['rows']for v in row.split()];m['objects']=[]
    pack['data']['maps']['MnElyShorePass']=m
    patches={}
    for name in ['MnEly','MnR11C00R02']:
        original=base['data']['maps'][name];m=pack['data']['maps'][name];mw=m['attributes']['width']
        patches[name]=[dict(position=[i%mw,i//mw],before=original['blocks'][i],after=b)for i,b in enumerate(m['blocks'])if b!=original['blocks'][i]]
    spec['patches']=patches
    spec['story']='Public pine-knoll back trail: encounter grass on the eastbound climb, native westbound ledge return, snowy Ely and south-bank Split Rock clinic. Existing story and water unchanged.'
    raw=json.dumps(spec,indent=2)+'\n';(source/'native.contract.json').write_text(raw)
    forward=course(pack,(0,30),(33,34));back=course(pack,(33,34),(0,30))
    least_east=course(pack,(0,30),(33,34),True);least_west=course(pack,(33,34),(0,30),True)
    assert least_east['cost']>=8,least_east
    assert least_west['cost']==0,least_west
    assert any(s['jump']for s in back['steps'])
    # Reverse at the lower landing must stay blocked; ordinary uphill remains.
    assert (22,34)not in [v[0]for v in edges(pack,'MnR11C00R02',(21,34))]
    for patch in patches['MnR11C00R02']:
        assert set(base['data']['tilesets']['mn_north_shore']['collision'][f'{patch["before"]:02x}'])<=set(['WALL','FLOOR','TALL_GRASS'])
    c=dict(eastbound=forward,westbound=back,minimum_grass_east=least_east['cost'],minimum_grass_west=least_west['cost'],dry_return=least_west)
    (source/'input-plan.json').write_text(json.dumps(c,indent=2)+'\n')
    art=SourceArt(pack)
    for name in ['MnEly','MnElyShorePass','MnR11C00R02']:
        m=pack['data']['maps'][name];a=m['attributes'];im=art.render(a['tileset_name'],a['width'],a['height'],m['blocks'],'day')
        save_render(im,out/(name+'-whole.png'))
        if name=='MnR11C00R02':save_render(im.crop((0,12*32,19*32,20*32)).resize((1216,512),Image.Resampling.NEAREST),out/'pine-knoll-trail.png')
    save_render(connected(pack,art),out/'connected-whole.png')
    (source/'author.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(dict(source=str(source),renders=str(out),contract_sha256=hashlib.sha256(raw.encode()).hexdigest(),east_inputs=len(forward['steps']),west_inputs=len(back['steps']),minimum_grass_east=least_east['cost'],minimum_grass_west=least_west['cost'],seconds=round(time.monotonic()-began,2)),indent=2))
if __name__=='__main__':main()
