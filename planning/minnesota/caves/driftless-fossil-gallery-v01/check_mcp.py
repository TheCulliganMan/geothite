#!/usr/bin/env python3
"""Replay actual dark/lit gallery checkpoints, cache and connected returns.

Consumes native controller saves; never changes game state except through input.
Controller cases separately prove HM teaching, Flash, held acknowledgment and
full-pocket native Toss/retry. Fresh process reload checks the final save again.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from check_mina_battle_mcp import LiveRecorded
from check_north_shore_story_mcp import finish
from check_winona_mcp import walk
from render_reference import load_pack

ROOM='MnDriftlessLampGallery'
CACHE='MN_LAMP_GALLERY_CACHE'


def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()


def visible(c):
    return {o['id'] for o in c.call('map_info')['structuredContent']['visible_objects']}


def select_pack(c):
    c.press('start')
    for _ in range(24):
        rows=c.observe()['structuredContent']['menu']
        chosen=next((r['text'].strip()[1:].strip() for r in rows if r['text'].strip().startswith('>')),None)
        if chosen and chosen.upper()=='PACK':c.press('a');return
        c.press('down')
    raise AssertionError('production Start menu did not select PACK')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['pack','client','saves','inspector','output']:parser.add_argument('--'+name,type=Path,required=True)
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=False);pack=load_pack(args.pack)
    idfile=args.output/'modpack-id.txt';idfile.write_text(pack['identity']['runtime_modpack_id']);results=[]
    for case in ['dark-fossil','lit-fossil-wall']:
        folder=args.output/case;folder.mkdir();source=args.saves/(case+'.crystalsave');save=folder/'actual-autosave.crystalsave'
        bindings={str(p):sha(p)for p in [args.pack,args.client,args.inspector,source]};sessions=[];error=None;process=None;inspection=None
        try:
            for phase in [0,1]:
                records=[];sessions.append(records);initial=source if phase==0 else folder/'actual-returned.crystalsave'
                with(folder/f'stderr-{phase}.log').open('x')as err:
                    process=subprocess.Popen([str(args.client.resolve()),'mcp',str(args.pack.resolve()),'--load',str(initial.resolve()),'--save',str(save.resolve())],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=err,text=True,bufsize=1)
                    c=LiveRecorded(process,records);c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='driftless-gallery',version='1')))
                    c.at(ROOM,*([5,4]if phase==0 else[18,17]))
                    if phase==0:
                        c.press('right');c.press('a');pages=finish(c)
                        if case=='dark-fossil':assert 'Too dark' in pages and CACHE not in visible(c),pages
                        else:assert 'shallow sea' in pages and CACHE in visible(c),pages
                        for point in [(6,9),(5,4),(9,11),(14,14),(16,13),(18,9)]:walk(c,pack,point)
                        if case=='lit-fossil-wall':
                            c.press('right');c.press('a');finish(c);assert CACHE not in visible(c)
                            select_pack(c);view=c.observe();assert 'RARE CANDY' in json.dumps(view).upper(),view
                            for _ in range(4):c.press('b')
                        walk(c,pack,(19,16));c.move('down');c.at('MnR01Cave1F',10,9)
                        walk(c,pack,(9,10));c.move('down');c.at('MnR01C00R00',18,18)
                        shutil.copy2(save,folder/'actual-world-return.crystalsave')
                        c.move('up');c.at('MnR01Cave1F',9,11);c.move('up');c.at('MnR01Cave1F',9,10)
                        walk(c,pack,(11,8));c.move('down');c.at(ROOM,18,17)
                        assert CACHE not in visible(c)
                        shutil.copy2(save,folder/'actual-returned.crystalsave')
                    else:
                        assert CACHE not in visible(c)
                        c.press('start');assert any('PACK'in r['text']for r in c.observe()['structuredContent']['menu']);c.press('b')
                    checked=subprocess.run([str(args.inspector.resolve()),str(idfile),sha(args.pack),pack['identity']['content_hash'],str(save)],capture_output=True,text=True)
                    assert checked.returncode==0,checked.stderr
                    assert 'MODE=Normal'in checked.stdout and 'MAP='+ROOM in checked.stdout
                    inspection=checked.stdout
                    process.stdin.close();assert process.wait(timeout=30)==0
        except BaseException as exc:error=repr(exc)
        finally:
            if process is not None and process.poll() is None:process.terminate();process.wait(timeout=30)
        stderr=''.join(p.read_text()for p in folder.glob('stderr-*.log'));unchanged=all(sha(Path(p))==h for p,h in bindings.items())
        receipt=dict(status='PASS'if error is None and not stderr and unchanged else'FAIL',error=error,bindings=bindings,inputs_unchanged=unchanged,stderr=stderr,sessions=sessions,native_save_inspection=inspection,fresh_process_continue=True,scope=__doc__)
        (folder/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');results.append(dict(case=case,status=receipt['status'],rpc_calls=sum(len(s)for s in sessions)))
        (args.output/'receipt.json').write_text(json.dumps(results,indent=2)+'\n');assert receipt['status']=='PASS',error
        print(case,'PASS',results[-1]['rpc_calls'],'RPC calls',flush=True)


if __name__=='__main__':main()
