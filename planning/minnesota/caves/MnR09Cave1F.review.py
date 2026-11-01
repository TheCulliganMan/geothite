import argparse,hashlib,json,sys,time
from copy import deepcopy
from pathlib import Path
from PIL import Image
sys.path.insert(0,'planning/minnesota/tools')
from render_reference import load_pack,npc_palette_pack,room_sheet
from render_archive import save_render
from render_stages import render_stage
from review_cave_candidate import review,path
from review_north_shore import oriented_sprite
def validate(candidate, original, pack):
    """Check immutable Q06 source fields and actual quadrant-level geometry."""
    assert {k:v for k,v in candidate.items() if k not in ['rows','review_contract']} == {k:v for k,v in original.items() if k!='rows'}, 'Q06 source contract changed'
    width,height=candidate['dimensions_metatiles'];assert [width,height]==[12,11]
    rows=[row.split() for row in candidate['rows']]
    assert len(rows)==height and all(len(row)==width for row in rows)
    blocks=[int(value,16) for row in rows for value in row]
    collision=pack['data']['tilesets']['cave']['collision']
    kind=lambda q:collision[f'{blocks[q[1]//2*width+q[0]//2]:02x}'][q[1]%2*2+q[0]%2]
    actors=candidate['residents']+candidate['story_trainers']+candidate['story_objects']
    native=pack['data']['maps'][candidate['id']]
    assert sorted((a['id'],*a['tile'],a['sprite']) for a in actors)==sorted((a['object_identifier'],a['x'],a['y'],a['sprite']) for a in native['objects'])
    bodies={tuple(a['tile']) for a in actors}
    assert all(kind(at)=='FLOOR' for at in bodies)
    floor={(x,y) for y in range(height*2) for x in range(width*2) if kind((x,y))=='FLOOR' and (x,y) not in bodies}
    start=tuple(candidate['clear_arrival']);assert start in floor
    for actor in candidate['story_trainers']+candidate['story_objects']:
        approach=tuple(actor['approach']);assert approach in floor and path(start,approach,floor),actor['id']
        assert sum(abs(x-y) for x,y in zip(approach,actor['tile']))==1
    for at in floor:assert path(start,at,floor),f'Stranded Floor {at}'
    assert all(kind((x,y))=='WALL' for y in range(height*2) for x in range(width*2) if x in [0,width*2-1] or y in [0,height*2-1])
    ladders={(x,y) for y in range(height*2) for x in range(width*2) if kind((x,y))=='LADDER'}
    assert ladders=={tuple(p) for p in candidate['exits']}=={(13,19)}
    assert blocks[9*width+6]==0x07, 'Keep all original mouth quadrants'
    declared=set()
    for ledge in candidate['review_contract']['one_way_ledges']:
        x0,y0,x1,y1=ledge['bounds_core'];assert y1==y0+2
        for x in range(x0,x1):
            assert kind((x,y0))=='HOP_DOWN' and kind((x,y0+1))=='WALL'
            assert kind((x,y0-1))==kind((x,y1))=='FLOOR'
            ordinary=path((x,y1),(x,y0-1),floor)
            assert ordinary and len(ordinary)-1>=ledge['minimum_return_steps']
            declared.add((x,y0))
        assert all(kind(tuple(p))=='WALL' for p in ledge['side_barriers'])
    hops={(x,y) for y in range(height*2) for x in range(width*2) if kind((x,y)).startswith('HOP')}
    assert hops==declared, 'Every native hop needs its actual return contract'
    allowed={'FLOOR','WALL','UP_WALL','LADDER','HOP_DOWN'}
    assert all(kind((x,y)) in allowed for y in range(height*2) for x in range(width*2)), 'No new water, grass, holes or unpaired entrances'


def negative_checks(candidate, original, pack):
    rejected=[]
    for case in ['moved foreman','changed party','changed cache flag','moved exit','blocked foreman approach','blocked cache','unpaired ladder','missing exit ladder','removed lip','opened ledge side','opened map edge','added actor']:
        bad=deepcopy(candidate)
        if case=='moved foreman':bad['story_trainers'][0]['tile'][0]+=1
        elif case=='changed party':bad['story_trainers'][0]['party'][0]['level']+=1
        elif case=='changed cache flag':bad['story_objects'][0]['recovered_flag']='EVENT_MN_DEPOT_CLEARED'
        elif case=='moved exit':bad['exits'][0][0]-=1
        elif case=='added actor':bad['residents'].append(deepcopy(bad['residents'][0]))
        else:
            rows=[row.split() for row in bad['rows']]
            x,y,value={'blocked foreman approach':(4,3,'01'),'blocked cache':(3,2,'01'),'unpaired ladder':(5,8,'1b'),'missing exit ladder':(6,9,'02'),'removed lip':(5,4,'02'),'opened ledge side':(4,4,'02'),'opened map edge':(0,0,'02')}[case]
            rows[y][x]=value;bad['rows']=[' '.join(row) for row in rows]
        try:validate(bad,original,pack)
        except AssertionError:rejected.append(case)
        else:raise AssertionError('Corruption accepted: '+case)
    return rejected


parser=argparse.ArgumentParser();parser.add_argument('--room',type=Path,default=Path('planning/minnesota/caves/MnR09Cave1F.candidate.json'));parser.add_argument('--pack',type=Path,default=Path('content-packs/minnesota-northstar-map-refinements.crystalpack'));parser.add_argument('--self-test',action='store_true');args=parser.parse_args();started=time.perf_counter()
base_path=Path('planning/minnesota/caves/MnR09Cave1F.previous.json');base=json.loads(base_path.read_bytes());raw=args.room.read_bytes();room=json.loads(raw);root=Path('planning/minnesota/renders/caves/r09-candidate');out=root/hashlib.sha256(raw).hexdigest()[:16];out.mkdir(parents=True,exist_ok=True)
(out/'MnR09Cave1F.candidate.json').write_bytes(raw);(out/'MnR09Cave1F.previous.json').write_bytes(base_path.read_bytes())
assert {k:v for k,v in room.items() if k not in ['rows','review_contract']}=={k:v for k,v in base.items() if k!='rows'}, 'Only terrain rows and nonruntime review metadata may differ'
pack_path=args.pack;pack=load_pack(pack_path);native=pack['data']['maps'][room['id']];assert native['blocks']==[int(x,16) for row in base['rows'] for x in row.split()]
validate(room,base,pack)
rejected=negative_checks(room,base,pack) if args.self_test else []
# Adapt only the physical display roster for the existing generic source renderer.
# Authoritative story actors remain story actors; no item/event lowering is run.
preview=deepcopy(room);preview['residents']+=preview['story_trainers'];preview['items']+=preview['story_objects'];review(pack,preview,out/'generic-preview')
for name in ['terrain-original.png','whole-source-rgba.png','whole-source.png']:(out/name).write_bytes((out/'generic-preview'/name).read_bytes())
w,h=room['dimensions_metatiles'];blocks=[int(x,16) for row in room['rows'] for x in row.split()];collisions=pack['data']['tilesets']['cave']['collision'];kind=lambda q:collisions[f'{blocks[q[1]//2*w+q[0]//2]:02x}'][q[1]%2*2+q[0]%2]
actors=room['residents']+room['story_trainers']+room['story_objects'];occupied={tuple(a['tile']) for a in actors};floor={(x,y) for y in range(h*2) for x in range(w*2) if kind((x,y))=='FLOOR' and (x,y) not in occupied};start=tuple(room['clear_arrival'])
for a in room['story_trainers']+room['story_objects']:
 assert kind(tuple(a['tile']))=='FLOOR' and path(start,tuple(a['approach']),floor),a['id']
ledge_reports=[]
for ledge in room['review_contract']['one_way_ledges']:
 x0,y0,x1,y1=ledge['bounds_core'];assert all(kind((x,y0))=='HOP_DOWN' and kind((x,y0+1))=='WALL' for x in range(x0,x1));assert all(kind(tuple(q))=='WALL' for q in ledge['side_barriers']);assert kind(tuple(ledge['reverse_blocked']))=='WALL';ret=path(tuple(ledge['landing']),tuple(ledge['upper_approach']),floor);assert ret and len(ret)-1>=ledge['minimum_return_steps'];ledge_reports.append(dict(id=ledge['id'],ordinary_return_steps=len(ret)-1,return_core=ret))
whole=Image.open(out/'whole-source.png');player=oriented_sprite(npc_palette_pack(pack,'nite'),'SPRITE_CHRIS','south');cameras=[]
# Largest genuinely empty all-Floor core square, excluding all three story bodies.
sizes={};worst=(0,0,0)
for y in range(h*2):
 for x in range(w*2):
  n=1+min(sizes.get((x-1,y),0),sizes.get((x,y-1),0),sizes.get((x-1,y-1),0)) if (x,y) in floor else 0;sizes[x,y]=n
  if n>worst[0]:worst=n,x-n+1,y-n+1
n,x,y=worst;at=[x+n//2,y+n//2]
views=room['review_contract']['decision_views']+[dict(id='ACTUAL_WEAKEST_FLOOR',tile=at)]
for v in views:
 x,y=v['tile'];assert (x,y) in floor,v['id'];left=max(0,min(x*16-80,whole.width-160));top=max(0,min(y*16-72,whole.height-144));crop=render_stage(whole,'crop',out/'camera-stages',v['id'],(left,top,left+160,top+144));save_render(crop,out/(v['id'].lower()+'-terrain160.png'));rgba=render_stage(crop,'convert',out/'camera-stages',v['id']+'-rgba','RGBA');anchor=(x*16-left,y*16-top);rgba.alpha_composite(player,anchor);save_render(rgba,out/(v['id'].lower()+'-player-rgba.png'));rgb=render_stage(rgba,'convert',out/'camera-stages',v['id']+'-rgb','RGB');save_render(rgb,out/(v['id'].lower()+'-player160.png'));save_render(render_stage(rgb,'resize',out/'camera-stages',v['id']+'-4x',(640,576),Image.Resampling.NEAREST),out/(v['id'].lower()+'-4x.png'));cameras.append((v['id']+' '+str(v['tile']),rgb))
save_render(room_sheet(cameras,columns=3),out/'decision-cameras.png');save_render(render_stage(whole,'resize',out/'whole-stages','whole-2x',(whole.width*2,whole.height*2),Image.Resampling.NEAREST),out/'whole-source-2x.png')
report=dict(status='SOURCE ONLY; no native publication, controller run, story script mutation or guard weakening',pack_sha256=hashlib.sha256(pack_path.read_bytes()).hexdigest(),previous_source_sha256=hashlib.sha256(base_path.read_bytes()).hexdigest(),candidate_source_sha256=hashlib.sha256(raw).hexdigest(),dimensions_metatiles=[w,h],all_nonterrain_story_actor_party_reward_warp_flag_fields_equal=True,native_previous_all132_blocks_equal=True,native_previous_all_nonterrain_field_hashes='baseline-c16-1793069600708277000/native-field-hashes.json',native_programs_never_lowered=True,physical_display_adapter_only=True,actors=[dict(id=a['id'],tile=a['tile']) for a in actors],ordinary_ledge_returns=ledge_reports,widest_empty_floor_square_core=n,weakest_camera_core=at,source_cameras_clamped_to_actual_room_extent=True,source_preview_not_native_capture=True,errors=[])
report.update(negative_cases_rejected=rejected,total_review_seconds=round(time.perf_counter()-started,3))
(out/'source-review.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(output=str(out),**report),indent=2))
