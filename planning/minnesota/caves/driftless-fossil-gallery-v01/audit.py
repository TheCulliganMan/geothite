#!/usr/bin/env python3
"""Independently bound the gallery, paired stair, hint and native Flash delta.

No game content is written. Native controller, stdio and visual acceptance are
separate; this checks complete preservation and the declared static contracts.
"""
import argparse
import base64
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from audit_regional_species_policy import audit
from render_reference import load_pack

HOLLOW='MnR01Cave1F';GALLERY='MnDriftlessLampGallery';CONSTANT='MN_DRIFTLESS_LAMP_GALLERY'
GUIDE='MN_R01_HOLLOW_GUIDEText'
FLAGS=['EVENT_MN_LAMP_GALLERY_READ','EVENT_MN_LAMP_GALLERY_CACHE','EVENT_MN_LAMP_GALLERY_HIDDEN']


def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()


def cmd(name,*args):return dict(command=name,args=list(args))


def wire(rows):return [dict(command=r['command'],args=r['args'])for r in rows]


def reachable(allowed,start):
    seen,queue={start},deque([start])
    while queue:
        x,y=queue.popleft()
        for at in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
            if at in allowed and at not in seen:seen.add(at);queue.append(at)
    return seen


def native_flash(old,new):
    expected=deepcopy(old)
    names={'Script_UseFlash','UseFlashTextScript','_BlindingFlashText'}
    assert set(new['scripts'])-set(old['scripts'])==names
    for name in names:expected['scripts'][name]=old['definitions'][name]
    for name in ['UseFlashTextScript','_BlindingFlashText']:
        body=old['definitions'][name]
        end=next((i+1 for i,r in enumerate(body)if r['command']=='text_asm'),len(body))
        expected['script_text_bodies'][name]=dict(label=name,commands=[dict(r,command_index=i)for i,r in enumerate(body[:end])])
    additions={
        'script_map_commands':[dict(command='refreshmap',target_map=None,x=None,y=None,facing=None,map_setup=None,source_script='Script_UseFlash',command_index=0)],
        'script_text_commands':[dict(command='writetext',text_label='UseFlashTextScript',source_script='Script_UseFlash',command_index=2),dict(command='closetext',text_label=None,source_script='Script_UseFlash',command_index=4)],
        'script_control_commands':[dict(command='end',compare_value=None,target_label=None,resolved_target_script=None,source_script='Script_UseFlash',command_index=5)],
        'script_runtime_commands':[dict(command='special',args=['UpdateTimePals'],source_script='Script_UseFlash',command_index=1),dict(command='callasm',args=['BlindingFlash'],source_script='Script_UseFlash',command_index=3)]}
    for field,rows in additions.items():
        # Native parsing interleaves the new script by label. Preserve the exact
        # order of every original row and every new command/index independently.
        assert [r for r in new[field]if r['source_script']!='Script_UseFlash']==old[field]
        assert [r for r in new[field]if r['source_script']=='Script_UseFlash']==rows
        expected[field]=new[field]
    assert expected==new,'native Flash closure must be the only global program addition'


def check(parent,child,source):
    old,new=parent['data'],child['data'];hollow=old['maps'][HOLLOW];current=new['maps'][HOLLOW];gallery=new['maps'][GALLERY]
    assert set(new['maps'])-set(old['maps'])=={GALLERY}
    expected_hollow=deepcopy(hollow);expected_hollow['blocks'][4*8+5]=0x0b
    assert hollow['blocks'][4*8+5]==2
    expected_hollow['events']['warps'] += [dict(index=2,x=11,y=9,target_map_constant=CONSTANT,target_map=CONSTANT,target_warp_id=2),dict(index=3,x=10,y=9,target_map_constant=hollow['attributes']['map_constant'],target_map=hollow['attributes']['map_constant'],target_warp_id=3)]
    text=hollow['scripts'][GUIDE];assert text[-1]==cmd('done')
    hint=[cmd('para'if i%2==0 else'line','"'+line+'"')for i,line in enumerate(source['guide_hint'])]
    expected_hollow['scripts'][GUIDE]=text[:-1]+hint+[cmd('done')]
    expected_hollow['script_text_bodies'][GUIDE]=dict(label=GUIDE,commands=[dict(r,command_index=i)for i,r in enumerate(expected_hollow['scripts'][GUIDE])])
    event_rows=deepcopy(hollow['map_event_section_commands']);cut=next(i for i,r in enumerate(event_rows)if r['command']=='def_coord_events')
    event_rows[cut:cut]=[dict(command='warp_event',args=['11','9',CONSTANT,'2']),dict(command='warp_event',args=['10','9',hollow['attributes']['map_constant'],'3'])]
    for i,r in enumerate(event_rows):r['command_index']=i
    expected_hollow['map_event_section_commands']=event_rows
    assert expected_hollow==current,'original HM dispatch, Antidote, actor or world exit changed'
    a=gallery['attributes'];width,height=source['dimensions_metatiles'];blocks=[int(v,16)for row in source['rows']for v in row.split()]
    assert [a['width'],a['height']]==[width,height]and gallery['blocks']==blocks
    assert a['tileset_name']=='cave'and a['time_of_day']=='dark'and a['phone_service']&15==4 and a['fishing_group']=='FISHGROUP_NONE'
    assert gallery['events']['warps']==[dict(index=1,x=19,y=17,target_map_constant=hollow['attributes']['map_constant'],target_map=hollow['attributes']['map_constant'],target_warp_id=3),dict(index=2,x=18,y=17,target_map_constant=CONSTANT,target_map=CONSTANT,target_warp_id=2)]
    assert gallery['events']['coord_events']==[]and gallery['events']['bg_events']==[dict(x=6,y=4,event_type='BGEVENT_READ',script='MnLampFossils')]
    assert len(gallery['objects'])==1
    actor=gallery['objects'][0];assert (actor['x'],actor['y'],actor['sprite'],actor['script'],actor['event_flag'])==(19,9,'SPRITE_POKE_BALL','MN_LAMP_GALLERY_CACHE',FLAGS[2])
    body=gallery['scripts']['MnLampFossils'];assert body[:2]==[cmd('checkflag','STATUSFLAGS_FLASH'),cmd('iffalse','MnLampTooDark')]
    assert body[4]==cmd('waitbutton')and body[5]==cmd('setevent',FLAGS[0])
    lines=[line for pair in source['read_pages']for line in pair]
    fossil_text=[cmd('text'if i==0 else'line'if i%2 else'para','"'+line+'"')for i,line in enumerate(lines)]+[cmd('done')]
    assert gallery['scripts']['MnLampFossilsText']==fossil_text
    reward=gallery['scripts']['MN_LAMP_GALLERY_CACHE'];grant=reward.index(cmd('verbosegiveitem','RARE_CANDY','1'));claim=reward.index(cmd('setevent',FLAGS[1]));assert reward[grant+1]==cmd('iffalse','MnLampCacheClose')and claim>grant+1
    collision=new['tilesets']['cave']['collision'];kinds={(x,y):collision[f'{blocks[y//2*width+x//2]:02x}'][y%2*2+x%2]for y in range(height*2)for x in range(width*2)}
    floor={p for p,k in kinds.items()if k=='FLOOR'}-{tuple(source['cache'])};seen=reachable(floor,tuple(source['passive_arrival']));assert floor==seen
    assert len(floor)==123 and kinds[6,4]=='WALL'and kinds[5,4]=='FLOOR'and kinds[19,17]=='LADDER'
    assert all(k=='WALL'for(x,y),k in kinds.items()if x in [0,width*2-1]or y in [0,height*2-1])
    assert all(tuple(v['tile'])in seen for v in source['review_contract']['viewpoints'])
    for stair in source['review_contract']['stairs']:
        assert all(tuple(p)in seen for p in [stair['before'],*stair['lane'],stair['after']]);assert kinds[tuple(stair['blocked_target'])]=='WALL'
    expected=deepcopy(parent);expected['identity']=child['identity'];data=expected['data'];data['maps'][HOLLOW]=current;data['maps'][GALLERY]=gallery
    data['map_blocks'][a['blocks_label']]=base64.b64encode(bytes(blocks)).decode();data['map_blocks'][current['attributes']['blocks_label']]=base64.b64encode(bytes(current['blocks'])).decode()
    data['map_attributes'][GALLERY]=a;data['map_dimensions'][CONSTANT]=dict(width=width,height=height);data['npcs'][GALLERY]=gallery['objects']
    labels=dict(gallery['scripts']);labels[a['map_scripts_label']]=wire(gallery['map_script_section_commands']);labels[a['map_events_label']]=wire(gallery['map_event_section_commands'])
    assert not(set(labels)&set(old['map_scripts']))
    data['map_scripts'].update(labels);data['map_scripts'][GUIDE]=expected_hollow['scripts'][GUIDE];data['map_scripts'][current['attributes']['map_events_label']]=wire(event_rows)
    constant=hollow['attributes']['map_constant'];metadata=deepcopy(old['runtime_map_metadata'][constant]);metadata.update(constant=CONSTANT,name=GALLERY,mapId=max(v['mapId']for v in old['runtime_map_metadata'].values()if v['groupId']==metadata['groupId'])+1,width=width,height=height,phoneService=(metadata['phoneService']&240)|4);data['runtime_map_metadata'][CONSTANT]=metadata
    data['pokegear_landmarks']['map_to_landmark'][GALLERY]=old['pokegear_landmarks']['map_to_landmark'][HOLLOW]
    maximum=max(v for k,v in old['story_event_script_constants']['global'].items()if k.startswith('EVENT_'))
    for i,flag in enumerate(FLAGS):data['story_event_script_constants']['global'][flag]=maximum+i+1
    standard=next(e['StandardScripts']for e in data['story_events']if'StandardScripts'in e);standard['GlobalScriptRoots'].append('Script_UseFlash')
    native_flash(old['global_scripts'],new['global_scripts']);data['global_scripts']=new['global_scripts']
    expected['report']['maps']=len(new['maps'])
    expected['report']['manifests'] += ['northstar-lair-mndriftlesslampgallery','minnesota-driftless-fossil-gallery-v1']
    for field in ['reachable_maps','solvable_maps']:
        expected['report'][field].append(GALLERY)
    assert expected==child,[k for k in data if data[k]!=new[k]]
    roster=json.loads((ROOT/'planning/minnesota/pokemon/roster.json').read_text());curation=json.loads((ROOT/'planning/minnesota/pokemon/roster-curation.json').read_text());retired={f['id']for f in curation['retired_families']};active={m['name'].upper()for f in roster['families']if f['id']not in retired for m in f['members']}
    assert 'BEETYNAMO'in active
    regional=audit(dict(child,data=dict(new,maps={HOLLOW:current,GALLERY:gallery})),active)
    return dict(unchanged_existing_maps=len(old['maps'])-1,new_maps=1,reachable_gallery_floor=len(floor),original_programs_except_appended_hint_exact=True,original_world_exit_and_antidote_exact=True,all_native_art_audio_exact=True,no_new_blackout_or_fly_destination=True,regional_policy=regional)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['parent','child','output']:p.add_argument('--'+n,type=Path,required=True)
    args=p.parse_args();assert not args.output.exists();source=Path(__file__).parent;parent=load_pack(args.parent);child=load_pack(args.child);spec=json.loads((source/'gallery.json').read_text());result=check(parent,child,spec)
    result.update(status='PASS',parent=str(args.parent),parent_sha256=sha(args.parent),pack=str(args.child),pack_sha256=sha(args.child),content_identity=child['identity']['content_hash'],source_sha256=sha(source/'gallery.json'))
    args.output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))


if __name__=='__main__':main()
