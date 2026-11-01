#!/usr/bin/env python3
"""Play rest refusal/retry, native climbs, cache and the actual exterior via MCP."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from check_mina_battle_mcp import LiveRecorded
from check_north_shore_story_mcp import choose,finish
from render_reference import load_pack

ROOM='MnR12Cave1F'


def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for key in ['pack','save','client','verifier','output']:parser.add_argument('--'+key,type=Path,required=True)
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=False)
    pack=load_pack(args.pack);identity=pack['identity'];idfile=args.output/'modpack-id.txt';idfile.write_text(identity['runtime_modpack_id'])
    bindings={k:sha(getattr(args,k))for k in ['pack','save','client','verifier']};calls=[];native=[];pages=[];live=args.output/'live.crystalsave';process=None
    report=dict(status='IN_PROGRESS',bindings=bindings,calls=calls,fixture='One declared initial injured local Minnesota party at Otta, no Surf, money or badges. Every later choice, heal, walk, cache, exterior transition and fresh-process Continue uses production input. This is local traversal/service verification, not earned regional access.')
    def verify(path=live):
        p=subprocess.run([str(args.verifier.resolve()),str(idfile.resolve()),bindings['pack'],identity['content_hash'],str(path.resolve())],capture_output=True,text=True)
        assert p.returncode==0,(p.stdout,p.stderr)
        assert 'MONEY=0 BADGES=0'in p.stdout and 'PARTY_SURF_MOVES=0'in p.stdout and 'MODE=Normal'in p.stdout
        return p.stdout
    def health(text):return [line for line in text.splitlines()if line.startswith('PARTNER=')]
    def rested(text):
        assert len(health(text))==2
        for line in health(text):
            hp=re.search(r'HP=(\d+)/(\d+)',line);assert hp and hp[1]==hp[2] and 'STATUS=None'in line,line
            for move,pp in re.findall(r'\("([A-Z_0-9]+)", (\d+)\)',line):assert int(pp)==pack['data']['moves'][move]['pp'],line
    def checkpoint(label):
        text=verify();native.append(dict(label=label,state=text));shutil.copy2(live,args.output/(label+'.crystalsave'));return text
    def walk(c,target):
        for _ in range(256):
            status=c.status();assert status['mode']=='overworld'and not status['dialogue'],status
            start=status['position']['x'],status['position']['y']
            if start==target:return
            m=pack['data']['maps'][status['map']];a=m['attributes'];coll=pack['data']['tilesets'][a['tileset_name']]['collision'];taken='CACHE=true'in verify()
            bodies={(o['x'],o['y'])for o in m['objects']if not(taken and o['script']=='MN_R12_CAVE_CACHE')}
            queue=deque([start]);paths={start:[]}
            while queue and target not in paths:
                x,y=queue.popleft()
                for dx,dy,key in [(1,0,'right'),(-1,0,'left'),(0,1,'down'),(0,-1,'up')]:
                    at=x+dx,y+dy
                    if at in paths or at in bodies or not(0<=at[0]<a['width']*2 and 0<=at[1]<a['height']*2):continue
                    block=m['blocks'][at[1]//2*a['width']+at[0]//2]
                    if coll[f'{block:02x}'][at[1]%2*2+at[0]%2]=='FLOOR':paths[at]=paths[x,y]+[key];queue.append(at)
            assert target in paths,(status['map'],start,target)
            c.move(paths[target][0])
        raise AssertionError('native walk did not arrive')
    before=health(verify(args.save));assert any('HP=0/'in line for line in before)and any('STATUS=Some("POISON")'in line for line in before)
    try:
        for phase in range(3):
            with(args.output/f'stderr-{phase}.log').open('x')as err:
                process=subprocess.Popen([str(args.client.resolve()),'mcp',str(args.pack.resolve()),'--load',str((args.save if phase==0 else live).resolve()),'--save',str(live.resolve())],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=err,text=True,bufsize=1)
                c=LiveRecorded(process,calls);c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='gunflint-spring',version='1')))
                if phase==0:
                    c.at(ROOM,6,5);c.press('left');c.press('a');choose(c,False);pages.append(finish(c))
                    assert health(checkpoint('rest-declined'))==before
                elif phase==1:
                    c.at(ROOM,6,5);assert health(verify())==before
                    c.press('left');c.press('a');choose(c,True)
                    for _ in range(48):
                        view=c.observe()
                        if 'Close your eyes.'in json.dumps(view):break
                        c.press('a')
                    else:raise AssertionError('accepted rest waitbutton absent')
                    pages.append(view);c.press('down');c.at(ROOM,6,5)
                    assert health(checkpoint('rest-before-acknowledgement'))==before
                    pages.append(finish(c));rested(checkpoint('rest-acknowledged'))
                    for y in range(6,11):c.move('down');c.at(ROOM,6,y)
                    for at in [(8,14),(13,18),(20,18),(20,14),(17,10),(13,10)]:walk(c,at)
                    walk(c,(26,10))
                    for y in range(9,4,-1):c.move('up');c.at(ROOM,26,y)
                    c.press('right');c.press('a');pages.append(finish(c))
                    text=checkpoint('actual-cache-collected');assert 'CACHE=true'in text and 'MAX_ETHER=1'in text
                    for y in range(6,10):c.move('down');c.at(ROOM,26,y)
                    c.move('right');c.at(ROOM,26,9)
                    c.move('down');c.at(ROOM,26,10);walk(c,(20,16));checkpoint('actual-spring-bank')
                    walk(c,(17,22))
                    for _ in range(3):
                        c.move('down')
                        if c.status()['map']!=ROOM:break
                    c.at('MnR12C00R00',40,15);c.move('down');c.at('MnR12C00R00',40,16)
                    checkpoint('safe-exterior-return')
                else:
                    c.at('MnR12C00R00',40,16);rested(verify());c.move('up');c.at(ROOM,17,23);c.move('up');c.at(ROOM,17,22)
                    walk(c,(27,5));c.press('a');pages.append(finish(c))
                    text=checkpoint('continued-cache-stays-gone');assert 'CACHE=true'in text and 'MAX_ETHER=1'in text
                    walk(c,(6,5));c.press('left');c.press('a');choose(c,True);pages.append(finish(c));rested(checkpoint('repeat-free-rest'))
                    c.press('start');assert any('PACK'in row['text']for row in c.observe()['structuredContent']['menu']);c.press('b')
                process.stdin.close();assert process.wait(timeout=30)==0;process=None
            assert(args.output/f'stderr-{phase}.log').stat().st_size==0
        assert all(sha(getattr(args,k))==value for k,value in bindings.items())
        report.update(status='PASS',rpc_calls=len(calls),native_checkpoints=native,pages=pages,fresh_process_continue=True)
    except BaseException as error:
        report.update(status='FAIL',error=repr(error),native_checkpoints=native,pages=pages);raise
    finally:
        if process is not None and process.poll()is None:process.terminate();process.wait(timeout=30)
        (args.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n')
    print('PASS native camp rest, two paths/climbs, cache, exterior and Continue;',len(calls),'RPC')


if __name__=='__main__':main()
