#!/usr/bin/env python3
"""Select explicit collision-safe grove sites; writes authoring JSON, never packs."""
import argparse,hashlib,json,collections,pickle
from pathlib import Path
from render_reference import load_pack,layout_blocks
from audit_outdoor_connections import collision
from review_river_fun import flood

from river_content_lineage import selected_parent,validate

BASE=selected_parent()
SOURCE=Path('planning/minnesota/fruit-groves/groves.compact-v01.json')
RIVER=Path('planning/minnesota/connections/mississippi-fun-v01')
APRICORNS=['RED_APRICORN','BLU_APRICORN','YLW_APRICORN','GRN_APRICORN','WHT_APRICORN','BLK_APRICORN','PNK_APRICORN']
BERRIES=['BERRY','PSNCUREBERRY','PRZCUREBERRY','BITTER_BERRY','MINT_BERRY','ICE_BERRY','BURNT_BERRY','MYSTERYBERRY','GOLD_BERRY']

def author():
    cache=Path('.local')/('fruit-review-'+hashlib.sha256(BASE.read_bytes()).hexdigest()+'.pickle');p=pickle.loads(cache.read_bytes()) if cache.exists() else load_pack(BASE);validate(p);maps=p['data']['maps'];activities=json.loads((RIVER/'revision.json').read_bytes());rivers=[];bank_regions=[]
    for reach in ['Lower','Upstream']:
        prefix=f'MnMississippi{reach}Woods';layout=json.loads((RIVER/f'{prefix}.layout.json').read_bytes());blocks=layout_blocks(layout)
        for row in range(3):
            for col in range(2):
                name=f'{prefix}C{col:02}R{row:02}';m=maps[name];before=bytes(m['blocks']);m['blocks']=[blocks[(row*24+y)*64+col*32+x] for y in range(m['attributes']['height']) for x in range(32)]
                rivers.append(dict(map=name,expected_blocks_sha256=hashlib.sha256(before).hexdigest(),blocks=m['blocks']))
                if not any(o['script'].startswith('MN_RIVER_FUN_') for o in m['objects']):
                    for a in next(v for v in activities['maps'] if v['id']==name)['activities']:
                        m['objects'].append(dict(x=a['tile'][0],y=a['tile'][1],sprite=a.get('sprite','SPRITE_POKE_BALL'),script=a['id'],radius=a.get('sight_range',0),object_type='OBJECTTYPE_TRAINER' if a['kind']=='trainer' else 'OBJECTTYPE_SCRIPT'))
        for i,bank in enumerate(layout['banks']):bank_regions.append((prefix,i,bank))
    trees=[];skipped=[];used=collections.defaultdict(list)
    def eligible(name,bounds=None):
        m=maps[name];a=m['attributes'];k=collision(p,m);w,h=a['width']*2,a['height']*2;bodies={(o['x'],o['y']) for o in m['objects']}
        events=[(v['x'],v['y']) for typ in ['warps','coord_events','bg_events'] for v in m['events'][typ]]
        starts={v for v,kind in k.items() if kind in ['FLOOR','TALL_GRASS'] and (v[0] in [0,w-1] or v[1] in [0,h-1])}
        for event in m['events']['warps']:
            x,y=event['x'],event['y'];starts.update(q for q in [(x,y),(x-1,y),(x+1,y),(x,y-1),(x,y+1)] if k.get(q) in ['FLOOR','TALL_GRASS'])
        for v,kind in k.items():
            if kind=='FLOOR' and any(k.get(q) in ['WATER','CUT_TREE','WHIRLPOOL','WATERFALL'] for q in [(v[0]-1,v[1]),(v[0]+1,v[1]),(v[0],v[1]-1),(v[0],v[1]+1)]):starts.add(v)
        reachable=flood(k,starts,{'FLOOR','TALL_GRASS'},bodies|set(used[name]))
        ranked=[]
        for y in range(3,h-3):
            for x in range(3,w-3):
                if bounds and not(bounds[0]<=x<bounds[2] and bounds[1]<=y<bounds[3]):continue
                if k[x,y]!='FLOOR' or (x,y) not in reachable:continue
                if any(k.get((x+dx,y+dy)) not in ['FLOOR','TALL_GRASS'] or (x+dx,y+dy) in bodies for dx in [-1,0,1] for dy in [-1,0,1]):continue
                if any(abs(x-qx)+abs(y-qy)<3 for qx,qy in events):continue
                if any(abs(x-o['x'])+abs(y-o['y'])<max(3,(o.get('radius') or 0)+2) for o in m['objects']):continue
                if any(abs(x-qx)+abs(y-qy)<9 for qx,qy in used[name]):continue
                walls=sum(k.get((x+dx,y+dy)) in ['WALL','HEADBUTT_TREE','TALL_GRASS'] for dx in range(-3,4) for dy in range(-3,4) if abs(dx)+abs(dy)<=3)
                if not walls:continue
                approach=next(((x+dx,y+dy) for dx,dy in [(0,1),(1,0),(-1,0),(0,-1)] if k.get((x+dx,y+dy))=='FLOOR' and (x+dx,y+dy) not in bodies),None)
                if approach is None:continue
                ranked.append((-walls,y,x,approach))
        return sorted(ranked)
    def place(name,item,bounds=None,feature=None):
        candidates=eligible(name,bounds)
        if not candidates:return False
        _,y,x,approach=candidates[0];used[name].append((x,y))
        trees.append(dict(id=f'FRUITTREE_MN_GROVE_{len(trees)+1:03}',map=name,tile=[x,y],approach=list(approach),item=item,feature=feature or 'Woodland edge harvest'))
        return True
    # One useful berry stop at each named river bank, subject to real collision.
    for prefix,i,bank in bank_regions:
        x0,y0,x1,y1=[v*2 for v in bank['bounds']];done=False
        for row in range(3):
            for col in range(2):
                name=f'{prefix}C{col:02}R{row:02}';bounds=[x0-col*64,y0-row*48,x1-col*64,y1-row*48]
                if place(name,BERRIES[i%len(BERRIES)],bounds,bank['id']):done=True;break
            if done:break
        if not done:skipped.append(dict(map=prefix,feature=bank['id'],reason='No Floor site with clear ring and approach'))
    exteriors=[name for name,m in maps.items() if name.startswith('Mn') and m['attributes']['environment'] in ['TOWN','ROUTE','OUTDOOR']]
    for i,name in enumerate(sorted(exteriors)):
        if 'Mississippi' in name and 'WoodsC' in name:continue
        m=maps[name];area=m['attributes']['width']*m['attributes']['height'];count=3 if area>500 else 2
        if area>1000:count=4
        items=[BERRIES[i%len(BERRIES)],APRICORNS[i%7],BERRIES[(i+3)%len(BERRIES)],APRICORNS[(i+2)%7]]
        for item in items[:count]:place(name,item)
        if not used[name]:skipped.append(dict(map=name,reason='No unoccupied grove site with clear walking ring'))
    design=dict(schema_version=1,base_pack=str(BASE),base_pack_sha256=hashlib.sha256(BASE.read_bytes()).hexdigest(),base_content_hash=p['identity']['content_hash'],rivers=rivers,river_activities=activities,trees=trees,skipped=skipped)
    SOURCE.write_text(json.dumps(design,indent=2)+'\n')
    print('trees',len(trees),'maps',len({t['map'] for t in trees}),'exteriors',len(exteriors),'river trees',sum('Mississippi' in t['map'] and 'WoodsC' in t['map'] for t in trees),'items',dict(collections.Counter(t['item'] for t in trees)),'skipped',skipped,flush=True)

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--base',type=Path,default=BASE);parser.add_argument('--output',type=Path,default=SOURCE);args=parser.parse_args();BASE=args.base;SOURCE=args.output
    assert not SOURCE.exists(), 'source epoch must be fresh'
    author()
