#!/usr/bin/env python3
"""Read-only whole-parent, native terrain and shelter-program preservation audit."""
import argparse
import base64
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import re
import sys

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import SourceArt, load_pack, npc_palette_pack
from review_passage_native import actors


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def command(name, *args):
    return dict(command=name,args=list(args))


def text(lines):
    rows = []
    for line in lines:
        row = ''
        for word in line.split():
            if row and len(row)+1+len(word)>18:
                rows.append(row); row=''
            row += (' ' if row else '')+word
        if row:
            rows.append(row)
    return [command('text' if i==0 else 'para' if i%2==0 else 'line','"'+row+'"')
            for i,row in enumerate(rows)]+[command('done')]


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=False)
    source=Path(__file__).parent;contract=json.loads((source/'contract.json').read_text())
    parent=ROOT/contract['parent_pack'];assert sha(parent)==contract['parent_pack_sha256']
    old,new=load_pack(parent),load_pack(args.pack);name='MnR12Cave1F';a,b=[p['data']['maps'][name]for p in [old,new]]
    assert hashlib.sha256(json.dumps(a,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()==contract['native_module_sha256']
    after=json.loads((source/'after.json').read_text());before=json.loads((source/'before.json').read_text())
    assert a['blocks']==[int(n,16)for row in before['rows']for n in row.split()]
    assert b['blocks']==[int(n,16)for row in after['rows']for n in row.split()]
    rest=after['rest'];root='MN_R12_OTTA'
    programs={
        root:[command('faceplayer'),command('opentext'),command('writetext','MN_R12_OTTAText'),command('waitbutton'),
              command('writetext','MnGunflintRestOfferText'),command('yesorno'),command('iffalse','MnGunflintRestDeclined'),
              command('writetext','MnGunflintRestAcceptedText'),command('waitbutton'),command('special','HealParty'),
              command('writetext','MnGunflintRestDoneText'),command('waitbutton'),command('closetext'),command('end')],
        'MnGunflintRestDeclined':[command('writetext','MnGunflintRestDeclinedText'),command('waitbutton'),command('closetext'),command('end')],
    }
    for suffix,key in [('Offer','offer_lines'),('Accepted','accepted_lines'),('Done','done_lines'),('Declined','declined_lines')]:
        programs['MnGunflintRest'+suffix+'Text']=text(rest[key])
    expected=deepcopy(old);expected['identity']=new['identity'];m=expected['data']['maps'][name];m['blocks']=b['blocks'];m['scripts'].update(programs)
    assert m['scripts']==b['scripts']
    allowed={'script_object_commands','script_text_commands','script_text_bodies','script_control_commands','script_runtime_commands'}
    for field in allowed:
        if field=='script_text_bodies':
            assert all(b[field][k]==v for k,v in a[field].items())
            assert set(b[field])-set(a[field])=={k for k in programs if k.endswith('Text')}
            for label in set(b[field])-set(a[field]):
                assert [dict(command=row['command'],args=row['args'])for row in b[field][label]['commands']]==programs[label]
        else:
            assert [r for r in a[field]if r['source_script']not in programs]==[r for r in b[field]if r['source_script']not in programs],field
            for row in b[field]:
                if row['source_script']in programs:
                    assert programs[row['source_script']][row['command_index']]['command']==row['command'],field
        m[field]=b[field]
    assert m==b,'unexpected complete shelter module change'
    assert b['script_runtime_commands']==[dict(command='special',args=['HealParty'],source_script=root,command_index=9)]
    for label,body in programs.items():expected['data']['map_scripts'][label]=body
    label=a['attributes']['blocks_label'];expected['data']['map_blocks'][label]=base64.b64encode(bytes(b['blocks'])).decode()
    expected['report']['manifests'].append('minnesota-gunflint-spring-shelter-v1')
    assert expected==new,'unowned whole-pack content changed'
    assert old['runtime_files']==new['runtime_files'],'native pixels/palettes/audio files changed'
    collision=new['data']['tilesets']['cave']['collision']
    kind=lambda p:collision[f'{b["blocks"][p[1]//2*16+p[0]//2]:02x}'][p[1]%2*2+p[0]%2]
    bodies={(o['x'],o['y'])for o in b['objects']};floor={(x,y)for y in range(24)for x in range(32)if kind((x,y))=='FLOOR'and(x,y)not in bodies}
    def path(start,goal,closed=()):
        queue=deque([start]);previous={start:None}
        while queue and goal not in previous:
            x,y=queue.popleft()
            for at in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if at in floor and at not in closed and at not in previous:previous[at]=(x,y);queue.append(at)
        if goal is None:return set(previous)
        assert goal in previous,(start,goal)
        result=[];at=goal
        while at is not None:result.append(at);at=previous[at]
        return result[::-1]
    assert path((17,22),None)==floor
    west=path((13,18),(13,10),{(x,y)for x in range(16,22)for y in range(12,18)})
    east=path((13,18),(13,10),{(x,y)for x in range(2,10)for y in range(12,18)})
    water={(x,y)for y in range(24)for x in range(32)if kind((x,y))=='WATER'}
    assert water and not any(x in [0,31]or y in[0,23]for x,y in water)
    assert {(x,y)for y in range(24)for x in range(32)if kind((x,y))=='WARP_CARPET_DOWN'}=={(17,23)}
    assert b['events']==a['events']and b['objects']==a['objects']
    roster=json.loads((ROOT/'planning/minnesota/pokemon/roster.json').read_text())
    curation=json.loads((ROOT/'planning/minnesota/pokemon/roster-curation.json').read_text())
    retired={family['id']for family in curation['retired_families']}
    normalize=lambda name:re.sub('[^A-Z0-9]','',name.upper())
    allowed={normalize(member['name'])for family in roster['families']if family['id']not in retired for member in family['members']}
    fish=new['data']['fishing'];group=fish['groups'][b['attributes']['fishing_group']];species=set()
    for table in group['rod_tables'].values():
        for slot in table['slots']:
            if slot['species']:species.add(slot['species'])
            if slot['time_group']:
                timed=fish['time_groups'][slot['time_group']]
                species.update([timed['day_species'],timed['night_species']])
    assert all(normalize(name)in allowed for name in species|{'CINDORE','FORTIGARD'})
    assert name not in new['data']['wild_encounters']and name not in new['data']['field_encounters']
    for label,at in [('rest',(6,5)),('cache',(26,5)),('spring',(20,16))]:assert at in floor
    for p,module,label in [(old,a,'before'),(new,b,'after')]:
        scene=actors(npc_palette_pack(p,'nite'),SourceArt(p).render('cave',16,12,module['blocks'],'nite'),module).convert('RGB')
        scene.save(args.output/(label+'-original.png'));scene.resize((1024,768),Image.Resampling.NEAREST).save(args.output/(label+'-2x.png'))
    sheet=Image.new('RGB',(1048,416),'#171d24');draw=ImageDraw.Draw(sheet)
    for i,label in enumerate(['before','after']):draw.text((8+i*520,6),label,fill='white');sheet.paste(Image.open(args.output/(label+'-original.png')),(8+i*520,24))
    sheet.save(args.output/'whole-before-after.png')
    receipt=dict(status='PASS',parent_pack=contract['parent_pack'],parent_pack_sha256=sha(parent),pack=str(args.pack),pack_sha256=sha(args.pack),content_identity=new['identity']['content_hash'],unchanged_other_maps=len(new['data']['maps'])-1,all_native_files_and_audio_exact=True,all_original_actors_warp_cache_and_text_preserved=True,reachable_dry_tiles=len(floor),water_tiles=len(water),dry_west_path=west,dry_east_path=east,original_cache='MAX_ETHER once through EVENT_MN_R12_CAVE_ETHER',rest='Native HealParty after acknowledged acceptance; no payment, flag or blackout command.',active_minnesota_fishing_species=sorted(species))
    (args.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print('PASS whole parent, native art/audio, dry branches, cache and rest programs')


if __name__=='__main__':main()
