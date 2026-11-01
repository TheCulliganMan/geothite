#!/usr/bin/env python3
"""Static physical proof from frozen source or exact written native terrain."""
import argparse
import base64
from collections import deque
from copy import deepcopy
from functools import lru_cache
import hashlib
import json
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
sys.path.insert(0,str(ROOT/'planning/minnesota/connections/river-continuity-v01'))
from render_reference import load_pack,layout_blocks
from audit_city_expansions import module_hash
from audit_outdoor_connections import collision
from audit_water_edges import production_permissions,numeric,blocked

DIRS=[('east',1,0),('west',-1,0),('south',0,1),('north',0,-1)]
TOKENS,WATER,WALL,ENGINE_HASHES=production_permissions()
PREFIXES=['MnMississippiUpstreamWoods','MnMississippiLowerWoods']


@lru_cache(maxsize=None)
def number(token):return numeric(token,TOKENS)


def sha(raw):return hashlib.sha256(raw).hexdigest()


def source_rows(pack,prefix,manifest):
    if prefix=='R05':
        return [dict(id=f'MnR05C00R{i:02}',source_origin_metatiles=[0,i*24],dimensions_metatiles=[32,pack['data']['maps'][f'MnR05C00R{i:02}']['attributes']['height']])for i in range(3)]
    return [r for r in manifest['new_maps']if r['id'].startswith(prefix)]


def kinds(pack,rows):
    grid={};bodies=set()
    for row in rows:
        x0,y0=row['source_origin_metatiles'];m=pack['data']['maps'][row['id']]
        grid.update({(x+x0*2,y+y0*2):v for (x,y),v in collision(pack,m).items()})
        bodies.update((o['x']+x0*2,o['y']+y0*2)for o in m['objects'])
    return grid,bodies


def wet(token):
    n=number(token)
    return n in WATER or 0xe0<=n<=0xe3


def water_step(a,b,d):
    if not wet(a)or not wet(b):return False
    for value,leaving in [(number(a),True),(number(b),False)]:
        if value in {0xe0,0xe1}and d in {'east','west'}:return False
        if value in {0xe2,0xe3}and d in {'north','south'}:return False
        if blocked(value,d,leaving):return False
    return True


def search(grid,start,target=None,bodies=frozenset(),foot=False):
    if start not in grid or (foot and grid[start]not in {'FLOOR','TALL_GRASS'})or(not foot and not wet(grid[start])):return {}
    previous={start:None};todo=deque([start])
    while todo:
        p=todo.popleft()
        if p==target:break
        for d,dx,dy in DIRS:
            q=p[0]+dx,p[1]+dy
            if q not in grid or q in previous or q in bodies:continue
            if (grid[q]in {'FLOOR','TALL_GRASS'}if foot else water_step(grid[p],grid[q],d)):
                previous[q]=p;todo.append(q)
    return previous


def course(previous,target):
    if target not in previous:return None
    points=[]
    while target is not None:points.append(list(target));target=previous[target]
    return list(reversed(points))


def clone_map(pack,name):
    result=dict(pack);result['data']=dict(pack['data']);result['data']['maps']=dict(pack['data']['maps']);result['data']['maps'][name]=deepcopy(pack['data']['maps'][name]);return result


def set_global(pack,rows,rectangle,block):
    result=dict(pack);result['data']=dict(pack['data']);result['data']['maps']=dict(pack['data']['maps'])
    x0,y0,x1,y1=rectangle
    for row in rows:
        name=row['id'];m=deepcopy(pack['data']['maps'][name]);ox,oy=row['source_origin_metatiles'];w,h=row['dimensions_metatiles']
        for y in range(h):
            for x in range(w):
                if x0<=x+ox<x1 and y0<=y+oy<y1:m['blocks'][y*w+x]=block
        result['data']['maps'][name]=m
    return result


def validate(before,after,manifest,owned):
    residual=dict(after['data']);residual['maps']=dict(after['data']['maps'])
    for name in owned:
        a=before['data']['maps'][name];b=after['data']['maps'][name];expected=deepcopy(a);expected['blocks']=b['blocks'];assert b==expected,('Nonterrain native drift',name)
        residual['maps'][name]=a
    assert residual==before['data'],'Unrelated game catalog/program drift'
    lengths=[];landings=[];port_counts={};bridge_cells=0;river_components=[]
    for prefix,start,end in [(PREFIXES[0],(0,40),(0,132)),(PREFIXES[1],(0,72),(36,135))]:
        rows=source_rows(after,prefix,manifest);old,_=kinds(before,rows);new,bodies=kinds(after,rows)
        sight=set()
        for row in rows:
            ox,oy=row['source_origin_metatiles']
            for obj in after['data']['maps'][row['id']]['objects']:
                direction=obj['sightline_direction_override']or obj['spritemovedata'].removeprefix('SPRITEMOVEDATA_STANDING_')
                dx,dy={'DOWN':(0,1),'UP':(0,-1),'LEFT':(-1,0),'RIGHT':(1,0),'south':(0,1),'north':(0,-1),'west':(-1,0),'east':(1,0)}[direction]
                sight.update((obj['x']+ox*2+dx*d,obj['y']+oy*2+dy*d)for d in range(1,obj['radius']+1))
        width=max(x for x,y in old)+1;height=max(y for x,y in old)+1
        ports=0
        for p in old:
            if p[0]in {0,width-1}or p[1]in {0,height-1}:
                assert wet(old[p])==wet(new[p]),('Unexpected river perimeter opening/closure',prefix,p)
                if wet(old[p]):assert old[p]==new[p],('Original water mouth quadrant changed',prefix,p);ports+=1
        port_counts[prefix]=ports
        old_path=course(search(old,start,end),end);new_search=search(new,start);new_path=course(new_search,end)
        assert old_path and new_path,('Required river through course disconnected',prefix)
        assert len(new_path)>=len(old_path)*0.9,('Long river replaced by short bypass',prefix,len(old_path),len(new_path))
        reverse_search=search(new,end);reverse_path=course(reverse_search,start)
        assert reverse_path,('Required reverse river course disconnected',prefix)
        wet_cells={p for p,t in new.items()if wet(t)}
        assert set(new_search)==wet_cells,('New river contains stranded water or a misleading disconnected cove',prefix,sorted(wet_cells-set(new_search)))
        assert set(reverse_search)==wet_cells,('River has a one-way water or bridge trap',prefix,sorted(wet_cells-set(reverse_search)))
        river_components.append(dict(region=prefix,all_wet_quadrants=len(wet_cells),reachable_from_both_ends=len(new_search),isolated_water_quadrants=0,reverse_steps=len(reverse_path)-1))
        lengths.append(dict(region=prefix,before_steps=len(old_path)-1,candidate_steps=len(new_path)-1,path=new_path,reverse_path=reverse_path))
        for row in rows:
            m=after['data']['maps'][row['id']];ox,oy=row['source_origin_metatiles']
            for obj in m['objects']:
                body=(obj['x']+ox*2,obj['y']+oy*2);approach=(body[0]-1,body[1]);assert new[body]==old[body]
                for _,dx,dy in DIRS:
                    q=body[0]+dx,body[1]+dy;assert new[q]==old[q],('Original actor approach changed',obj['script'],q)
                assert approach not in sight,('Manual interaction approach is in native trainer sight',obj['script'],approach)
                dry=search(new,approach,bodies=bodies|sight,foot=True);launches=[]
                for p in dry:
                    for d,dx,dy in DIRS:
                        q=p[0]+dx,p[1]+dy
                        if q in new_search and wet(new[q]):
                            launches.append((len(course(dry,p)),p,q,d))
                assert launches,('Original bank actor has no main-river landing',obj['script'])
                _,p,q,d=min(launches)
                landings.append(dict(script=obj['script'],body_core=list(body),approach_core=list(approach),native_standing_movement=obj['spritemovedata'],native_sight_radius=obj['radius'],native_sight_cells_avoided=sorted(list(p)for p in sight),land_core=list(p),water_core=list(q),land_to_water=d,foot_course_from_approach=course(dry,p),surf_course_from_entry=course(new_search,q)))
    r05rows=source_rows(after,'R05',manifest);old,_=kinds(before,r05rows);new,_=kinds(after,r05rows)
    for p,token in old.items():
        n=number(token)
        if n not in WATER|WALL:assert new[p]==token,('Original R05 foot/ledge/door/bridge permission changed',p,token,new[p])
        if 'CUT_TREE'in token:assert new[p]==token
        if 0xe0<=n<=0xe3:assert new[p]==token;bridge_cells+=1
    north=(45,0);north_join=(63,4);south_join=(63,96)
    assert course(search(new,north,north_join),north_join),'St Cloud to real upper river mouth disconnected'
    assert course(search(new,south_join,(16,111)),(16,111)),'Southern river mouth to actual Minneapolis boundary disconnected'
    assert south_join not in search(new,north),'Old parallel R05 river now bypasses the required woods'
    # Check old shore access directly rather than assuming a vanished water
    # strip cannot affect a reward. All unchanged qualified mounts/dismounts
    # retain the exact old ground and water quadrants and directional masks.
    def ground_water_edges(grid):
        mounts=set();dismounts=set();reverse={'north':'south','south':'north','east':'west','west':'east'}
        blocked_mounts=set(before['data']['field_moves']['surf']['blocked_collisions'])
        for p,t in grid.items():
            n=number(t)
            if n in WATER|WALL or 0xe0<=n<=0xe3:continue
            for d,dx,dy in DIRS:
                q=p[0]+dx,p[1]+dy
                if q not in grid or number(grid[q])not in WATER:continue
                if grid[q]not in blocked_mounts and not blocked(n,d,True)and not blocked(number(grid[q]),d):mounts.add((p,q,d))
                r=reverse[d]
                if not blocked(number(grid[q]),r,True)and not blocked(n,r):dismounts.add((q,p,r))
        return mounts,dismounts
    old_mounts,old_dismounts=ground_water_edges(old);new_mounts,new_dismounts=ground_water_edges(new)
    assert old_mounts==new_mounts,('Old R05 shore mount lost or changed',old_mounts-new_mounts,new_mounts-old_mounts)
    assert old_dismounts==new_dismounts,('Old R05 shore dismount lost or changed',old_dismounts-new_dismounts,new_dismounts-old_dismounts)
    old_main=set(search(old,north))|set(search(old,south_join));new_main=set(search(new,north))|set(search(new,south_join))
    for _,q,_ in old_mounts:
        if q in old_main:assert q in new_main,('Retained old shore launch stranded from main river',q)
    for q,_,_ in old_dismounts:
        if q in old_main:assert q in new_main,('Retained old shore dismount stranded from main river',q)
    cut_before=set();cut_after=set();reward_approaches=[]
    for name in owned:
        a=before['data']['maps'][name];b=after['data']['maps'][name];ak=collision(before,a);bk=collision(after,b)
        cut_before.update((name,p)for p,v in ak.items()if v=='CUT_TREE');cut_after.update((name,p)for p,v in bk.items()if v=='CUT_TREE')
        for group,events in [('objects',a['objects']),*a['events'].items()]:
            for e in events:
                assert ak[e['x'],e['y']]==bk[e['x'],e['y']],('Native anchor changed',name,group,e)
                if name.startswith('MnR05')and group=='objects'and ('ITEM'in e['script']or e['script'].endswith(('FISHER','LOOKOUT'))):
                    approaches=[]
                    for d,dx,dy in DIRS:
                        p=e['x']+dx,e['y']+dy;assert ak[p]==bk[p],('Old reward/NPC approach changed',e['script'],p)
                        approaches.append(dict(core=list(p),permission=ak[p]))
                    reward_approaches.append(dict(map=name,script=e['script'],body_core=[e['x'],e['y']],approaches=approaches))
    assert cut_before==cut_after,'Incidental Cut collision added or intended Cut gate removed'
    return dict(source_only=True,all_river_water_components=river_components,river_courses=lengths,all_four_bank_landings=landings,exact_external_water_quadrants=port_counts,r05_original_bridge_quadrants_retained=bridge_cells,r05_north_to_woods=course(search(new,north,north_join),north_join),r05_south_to_city=course(search(new,south_join,(16,111)),(16,111)),old_parallel_r05_through_bypass=False,r05_native_ground_water_access=dict(before_mounts=len(old_mounts),after_mounts=len(new_mounts),before_dismounts=len(old_dismounts),after_dismounts=len(new_dismounts),lost_mounts=0,lost_dismounts=0,meaning='Closed water had no removed native ground-water transition. All original land/bridge/ledge/warp permissions and reward approaches remain exact; no old Surf-dependent shore access is cut off.'),r05_old_rewards_and_cove_npcs=reward_approaches,cut_quadrants_retained=len(cut_before))


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--pack',type=Path,required=True);ap.add_argument('--candidate',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);ap.add_argument('--expected-parent-sha256');ap.add_argument('--written-pack',type=Path);ap.add_argument('--expected-written-sha256');args=ap.parse_args();assert bool(args.written_pack)==bool(args.expected_written_sha256);assert not args.output.exists();args.output.mkdir(parents=True)
    contract=json.loads((args.candidate/'candidate.contract.json').read_text());actual_parent_sha=sha(args.pack.read_bytes());assert actual_parent_sha==(args.expected_parent_sha256 or contract['parent_pack_sha256']);before=load_pack(args.pack)
    after=dict(before);after['data']=dict(before['data']);after['data']['maps']=dict(before['data']['maps']);owned=[]
    for row in contract['changes']:
        name=row['map'];old=before['data']['maps'][name];assert module_hash(old)==row['native_module_sha256'];m=deepcopy(old);w=old['attributes']['width']
        for cell in row['cells']:
            x,y=cell['metatile'];i=y*w+x;assert old['blocks'][i]==int(cell['before'],16);m['blocks'][i]=int(cell['after'],16)
        source_file=args.candidate/'source'/f'{name}.candidate.layout.json';source_file=source_file if source_file.exists()else args.candidate/f'{name}.layout.json';assert layout_blocks(json.loads(source_file.read_text()))==m['blocks'];after['data']['maps'][name]=m;owned.append(name)
    manifest=json.loads((ROOT/'planning/minnesota/connections/mississippi-v02/manifest.json').read_text())
    report=validate(before,after,manifest,owned);negatives=[]
    def reject(label,changed):
        try:validate(before,changed,manifest,owned)
        except AssertionError as e:negatives.append(dict(id=label,rejected=True,reason=str(e)));return
        raise AssertionError(('Counterexample incorrectly accepted',label))
    upper=source_rows(after,PREFIXES[0],manifest);lower=source_rows(after,PREFIXES[1],manifest)
    reject('Close actual southern upper-river throat',set_global(after,upper,[5,64,6,68],5))
    reject('Close actual lower-river middle channel',set_global(after,lower,[52,29,60,30],5))
    reject('Open unconnected forest boundary',set_global(after,upper,[0,5,1,6],0x35))
    reject('Add a stranded interior water pocket',set_global(after,upper,[5,5,6,6],0x35))
    reject('Bypass the long upper reach with an interior water slash',set_global(after,upper,[14,23,16,64],0x35))
    reject('Replace original fisher body tile by forest',set_global(after,upper,[16,65,17,66],5))
    reject('Replace original keeper approach by water',set_global(after,lower,[51,41,52,42],0x35))
    actual_cut_blocks=[int(k,16)for k,v in after['data']['tilesets']['johto']['collision'].items()if 'CUT_TREE'in v]
    if actual_cut_blocks:
        reject('Add an actual native Cut-tree quadrant to an ungated bank',set_global(after,upper,[15,60,16,61],actual_cut_blocks[0]))
    else:
        bad=dict(after);bad['data']=dict(after['data']);bad['data']['tilesets']=dict(after['data']['tilesets']);bad['data']['tilesets']['johto']=deepcopy(after['data']['tilesets']['johto']);bad['data']['tilesets']['johto']['collision']['05'][0]='CUT_TREE'
        reject('Inject a forbidden Cut collision into the native canopy definition',bad)
    r05rows=source_rows(after,'R05',manifest)
    reject('Reopen old parallel R05 through channel',set_global(after,r05rows,[29,13,31,45],0x35))
    reject('Erase original R05 city-hook foot bridge',set_global(after,r05rows,[11,54,12,55],0x35))
    n='MnR05C00R00';bad=clone_map(after,n);key=next(iter(bad['data']['maps'][n]['scripts']));bad['data']['maps'][n]['scripts'][key]=[];reject('Change original story raw program',bad)
    bad=clone_map(after,n);bad['data']['maps'][n]['objects'][0]['script']='UNRELATED_CHANGED_ACTOR';reject('Change original actor binding',bad)
    bad=clone_map(after,n);bad['data']['maps'][n]['events']['warps'][0]['target_map']='WRONG_CAVE';reject('Change original paired cave mouth',bad)
    bad=dict(after);bad['data']=dict(after['data']);bad['data']['moves']=deepcopy(after['data']['moves']);bad['data']['moves']['TACKLE']['power']+=1;reject('Change unrelated native move catalog',bad)
    report.update(proof_tool_sha256=sha(Path(__file__).read_bytes()),status='STATIC_PHYSICAL_SOURCE_PASS_NATIVE_INPUT_PENDING',parent_pack_sha256=actual_parent_sha,parent_identity=before['identity']['content_hash'],source_preimage_baseline_sha256=contract['parent_pack_sha256'],source_preimage_baseline_identity=contract['parent_identity'],candidate_contract_sha256=sha((args.candidate/'candidate.contract.json').read_bytes()),unchanged_native_map_metadata_and_unrelated_game_data=True,meaningful_negative_cases=negatives,engine_source_hashes=ENGINE_HASHES,limits=['No Rust pack is written and no native controller/GPU executes here.','Normal Surf/HM/badge/chapter prerequisites, trainer sight/battles/PP/rewards, Cut persistence and Save/Continue require native production regressions after publication.','Bank source paths avoid current actor bodies; runtime wandering/facing and menu ownership remain production concerns.'])
    if args.written_pack:
        assert sha((args.candidate/'candidate.contract.json').read_bytes())=='67c451f315baf3939f92ae619aae44c50963c069633dab177076ee18b99cd3cd'
        assert sha(args.written_pack.read_bytes())==args.expected_written_sha256
        written=load_pack(args.written_pack)
        expected_data=deepcopy(after['data']);changed_mirrors=0
        for row in contract['changes']:
            if not row['cells']:continue
            module=expected_data['maps'][row['map']];label=module['attributes']['blocks_label']
            expected_data['map_blocks'][label]=base64.b64encode(bytes(module['blocks'])).decode()
            changed_mirrors+=1
        assert changed_mirrors==14 and sum(len(r['cells'])for r in contract['changes'])==1678
        assert written['data']==expected_data,'Written native data differs from exact actual parent plus 1,678 cells and 14 mirrors'
        residual=dict(written);residual['data']=before['data'];residual['identity']=before['identity']
        assert residual==before,'Written non-game-data payload changed'
        assert len(written['data']['maps'])==683
        report.update(source_only=False,status='WRITTEN_NATIVE_STATIC_PHYSICAL_PASS_CONTROLLER_PENDING',written_pack_sha256=args.expected_written_sha256,written_identity=written['identity']['content_hash'],written_native_data_exactly_matches_checked_geometry=True,all_other_written_payloads_exact=True,changed_native_block_mirrors=changed_mirrors,native_controller_executed=False,parent_pack_unchanged=sha(args.pack.read_bytes())==actual_parent_sha)
        report['limits']=['The graph and 14 corruption cases describe actual written native terrain; no production controller or GPU executes here.','Surf/Waterfall prerequisites, current bridge-plane behavior, trainer battles/PP/once-only flags, old reward/cove access, Cut persistence and Save/Continue still require native production regressions.','All unrelated approved actual-parent maps/art/data and written top-level payloads remain exact. The source contract baseline is historical; it is not restored over the actual parent.']
    raw=(json.dumps(report,indent=2)+'\n').encode();(args.output/'proof.json').write_bytes(raw);(args.output/f'proof-{sha(raw)}.json').write_bytes(raw);print(json.dumps({k:report[k]for k in ['status','exact_external_water_quadrants','r05_original_bridge_quadrants_retained']},indent=2));print('negatives',len(negatives),'proof',sha(raw))


if __name__=='__main__':main()
