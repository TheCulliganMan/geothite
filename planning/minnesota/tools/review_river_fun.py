#!/usr/bin/env python3
"""Review the two explicit river revisions; never publish or edit a game pack."""
import argparse,collections,copy,hashlib,json,sys
from pathlib import Path
from PIL import Image,ImageDraw
from render_reference import load_pack,layout_blocks
from render_progress_atlas import native_room
from audit_outdoor_connections import collision


def flood(kinds, starts, allowed, blocked=frozenset()):
    parents={p:None for p in starts if p not in blocked};q=collections.deque(parents)
    while q:
        x,y=q.popleft()
        for v in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
            if v not in parents and v not in blocked and kinds.get(v) in allowed:
                parents[v]=(x,y);q.append(v)
    return parents


def review(base, source, output, written=None):
    output.mkdir(parents=True,exist_ok=False)
    plan=json.loads(source.read_bytes());p=load_pack(base);before=copy.deepcopy(p)
    assert p['identity']['content_hash']==plan['base_content_hash']
    for row in plan['maps']:
        m=p['data']['maps'][row['id']];assert hashlib.sha256(bytes(m['blocks'])).hexdigest()==row['expected_blocks_sha256']
        for cell in row['cells']:
            x,y=cell['metatile'];i=y*m['attributes']['width']+x;assert m['blocks'][i]==cell['before'];m['blocks'][i]=cell['after']
    for launch in plan.get('preserved_launches',[]):
        m=p['data']['maps'][launch['map']];v=tuple(launch['core'])
        assert collision(p,m)[v]==collision(before,before['data']['maps'][launch['map']])[v]=='FLOOR','original dry launch changed'
    if written:
        actual=load_pack(written)
        for row in plan['maps']:assert actual['data']['maps'][row['id']]['blocks']==p['data']['maps'][row['id']]['blocks']
        residual=copy.deepcopy(actual['data'])
        for row in plan['maps']:
            n=row['id'];m=before['data']['maps'][n]
            residual['maps'][n]=copy.deepcopy(m)
            residual['map_blocks'][m['attributes']['blocks_label']]=before['data']['map_blocks'][m['attributes']['blocks_label']]
            residual['npcs'][n]=before['data']['npcs'][n]
            for field in ['map_scripts_label','map_events_label']:
                label=m['attributes'][field];residual['map_scripts'][label]=before['data']['map_scripts'][label]
        for table in [residual['map_scripts'],residual['trainers']['trainers'],residual['story_event_script_constants']['global']]:
            for key in list(table):
                if key.startswith(('MN_RIVER_FUN_','EVENT_MN_RIVER_FUN_')):del table[key]
        assert residual==before['data'],'unrelated game payload changed'
        for field in ['runtime_files','audio_manifest','compiled_audio','audio_compression','format_version']:
            assert actual[field]==before[field],field+' changed'
        expected_report=copy.deepcopy(before['report']);expected_report['manifests'].append('minnesota-mississippi-fun-v1')
        assert actual['report']==expected_report,'unrelated report changed'
        p=actual
    receipt={'base':str(base),'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'pack':str(written) if written else None,'pack_sha256':hashlib.sha256(written.read_bytes()).hexdigest() if written else None,'reaches':{},'cases':[]}
    for reach in ['Lower','Upstream']:
        prefix=f'MnMississippi{reach}Woods';kinds={};old_kinds={};bodies=set();activities=[]
        layout=json.loads((source.parent/f'{prefix}.layout.json').read_bytes());raster=layout_blocks(layout)
        for row in plan['maps']:
            n=row['id']
            if not n.startswith(prefix):continue
            col=int(n[-5:-3]);r=int(n[-2:]);m=p['data']['maps'][n];ox,oy=col*64,r*48
            for (x,y),k in collision(p,m).items():kinds[x+ox,y+oy]=k
            for (x,y),k in collision(before,before['data']['maps'][n]).items():old_kinds[x+ox,y+oy]=k
            for y in range(m['attributes']['height']):
                for x in range(32):assert m['blocks'][y*32+x]==raster[(r*24+y)*64+col*32+x]
            bodies.update((o['x']+ox,o['y']+oy) for o in before['data']['maps'][n]['objects'])
            for a in row['activities']:
                a=copy.deepcopy(a);a.update(map=n,global_tile=[a['tile'][0]+ox,a['tile'][1]+oy]);activities.append(a);bodies.add(tuple(a['global_tile']))
        # Every original water quadrant and every outside boundary remains exact.
        assert all(kinds[v]=='WATER' for v,k in old_kinds.items() if k=='WATER')
        assert all(kinds[v]==k for v,k in old_kinds.items() if v[0] in [0,127] or v[1] in [0,135])
        water={v for v,k in kinds.items() if k=='WATER'}
        all_water=flood(kinds,[min(water)],{'WATER'})
        assert water <= all_water.keys(),'stranded water'
        landings=[(v,q) for v,k in kinds.items() if k=='FLOOR' and v not in bodies for q in [(v[0]-1,v[1]),(v[0]+1,v[1]),(v[0],v[1]-1),(v[0],v[1]+1)] if kinds.get(q)=='WATER']
        assert landings
        sight=set()
        for a in activities:
            if a['kind']=='trainer':
                x,y=a['global_tile'];dx,dy={'north':(0,-1),'south':(0,1),'east':(1,0),'west':(-1,0)}[a['facing']];sight.update((x+dx*i,y+dy*i) for i in [1,2]);assert not any(kinds.get(v)=='WATER' for v in sight),'trainer sees main river'
        land=flood(kinds,[v for v,_ in landings if v not in sight],{'FLOOR','TALL_GRASS'},bodies|sight)
        dry=flood(kinds,[v for v,_ in landings],{'FLOOR'},bodies)
        for a in activities:
            x,y=a['global_tile'];neighbors=[(x-1,y),(x+1,y),(x,y-1),(x,y+1)]
            goals=[v for v in neighbors if v in land];assert goals,(a['id'],'unreachable')
            goal=min(goals,key=lambda v:(kinds[v]=='TALL_GRASS',v))
            # Find a native Foot path from a real launch, then interaction input.
            path=[goal]
            while land[path[-1]] is not None:path.append(land[path[-1]])
            path.reverse();landing=path[0];water_at=next(q for v,q in landings if v==landing)
            if a['kind']=='item':assert not any(v in dry for v in neighbors),(a['id'],'reward has free dry bypass')
            if a['kind']=='trainer':assert kinds[tuple(a['global_tile'])]=='FLOOR'
            def location(v):
                x,y=v
                return {'map':f'{prefix}C{x//64:02}R{y//48:02}','point':[x%64,y%48]}
            case={**a,'path':[location(v) for v in path],'water':location(water_at)}
            receipt['cases'].append(case)
        grass={v for v,k in kinds.items() if k=='TALL_GRASS'}
        unreachable=grass-bodies-flood(kinds,[v for v,_ in landings],{'FLOOR','TALL_GRASS'},bodies).keys()
        assert not unreachable,('unreachable grass patch',reach,sorted(unreachable))
        components=0;unseen=set(grass)
        while unseen:
            component=flood(kinds,[min(unseen)],{'TALL_GRASS'});unseen-=component.keys();components+=1
        receipt['reaches'][reach]={'new_trainers':sum(a['kind']=='trainer' for a in activities),'total_trainers':20,'grass_gameplay_tiles':len(grass),'grass_patches':components,'launches':len(landings),'all_activities_reachable':True,'supplies_require_grass':True,'outer_boundary_preserved':True,'original_water_preserved':True}
        im=Image.new('RGB',(2048,2176))
        for r in range(3):
            for c in range(2):im.paste(native_room(p['data']['maps'][f'{prefix}C{c:02}R{r:02}'],p,native_origin=True),(c*1024,r*768))
        im.save(output/f'{reach}-whole-original.png');im.resize((768,816)).save(output/f'{reach}-preview.png')
    (output/'proof.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(receipt['reaches'],indent=2))

if __name__=='__main__':
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--base',type=Path,default=Path('content-packs/minnesota-park-pond-v05.crystalpack'));ap.add_argument('--source',type=Path,default=Path('planning/minnesota/connections/mississippi-fun-v01/revision.json'));ap.add_argument('--output',type=Path,required=True);ap.add_argument('--written',type=Path);a=ap.parse_args();review(a.base,a.source,a.output,a.written)
