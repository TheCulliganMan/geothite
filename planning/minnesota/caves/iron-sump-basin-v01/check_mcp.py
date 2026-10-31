#!/usr/bin/env python3
"""Replay actual sump fishing, Surf, Continue and five shaft pairs over stdio."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from check_mina_battle_mcp import LiveRecorded
from check_north_shore_story_mcp import finish
from render_reference import load_pack

ROOM = 'MnIronSump'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['pack','save','client','verifier','output']:
        parser.add_argument('--'+key,type=Path,required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True,exist_ok=False)
    pack = load_pack(args.pack)
    identity = pack['identity']
    idfile = args.output/'native-modpack-id.txt'
    idfile.write_text(identity['runtime_modpack_id'])
    bindings = {k:sha(getattr(args,k)) for k in ['pack','save','client','verifier']}
    constants = {m['attributes']['map_constant']:name for name,m in pack['data']['maps'].items()}
    live = args.output/'live.crystalsave'
    calls,pages = [],[]
    report = dict(status='IN_PROGRESS',bindings=bindings,calls=calls,
                  fixture='Actual controller-reached bank after native guards/valves/shafts and Surf; declared initial local party/access setup.')

    def verify(path=live):
        p = subprocess.run([str(args.verifier.resolve()),str(idfile.resolve()),bindings['pack'],identity['content_hash'],str(path.resolve())],capture_output=True,text=True)
        assert p.returncode == 0,(p.stdout,p.stderr)
        return p.stdout

    initial = verify(args.save)
    assert 'MODE=Normal' in initial
    assert all(flag+'=true' in initial for flag in ['EVENT_MN_IRON_HAUL_PRESSURE_OFF','EVENT_MN_IRON_VENT_PRESSURE_OFF','EVENT_MN_LODE_ARC_SUMP_DEFEATED','EVENT_MN_LODE_ARC_DRAIN_DEFEATED'])
    # This is only the expected graph for planning. The named valves were
    # operated through controller input in the reached save; no pack or game
    # state is written by these local in-memory copies.
    for name,index,block in [('MnIronHaulage',8*18+9,13),('MnIronVentilation',6*18+9,7),(ROOM,3*20+16,13)]:
        pack['data']['maps'][name]['blocks'][index] = block

    def walk(c,target):
        for _ in range(512):
            status = c.status()
            if status['dialogue']:
                pages.append(finish(c));continue
            assert 'battle' not in status['mode'],status
            name = status['map'];start = status['position']['x'],status['position']['y']
            if start == target:return
            m = pack['data']['maps'][name];a = m['attributes']
            collision = pack['data']['tilesets'][a['tileset_name']]['collision']
            bodies = {(o['x'],o['y']) for o in m['objects']}
            queue,paths = deque([start]),{start:[]}
            while queue and target not in paths:
                x,y = queue.popleft()
                for dx,dy,key in [(1,0,'right'),(-1,0,'left'),(0,1,'down'),(0,-1,'up')]:
                    at = x+dx,y+dy
                    if at in paths or at in bodies or not (0 <= at[0] < a['width']*2 and 0 <= at[1] < a['height']*2):continue
                    block = m['blocks'][at[1]//2*a['width']+at[0]//2]
                    if collision[f'{block:02x}'][at[1]%2*2+at[0]%2] == 'FLOOR':
                        paths[at] = paths[(x,y)]+[key];queue.append(at)
            assert target in paths,(name,start,target)
            c.move(paths[target][0])
        raise AssertionError('native walking did not arrive')

    def cross(c,name,index):
        port = next(w for w in pack['data']['maps'][name]['events']['warps'] if w['index'] == index)
        to = constants[port['target_map_constant']]
        arrival = next(w for w in pack['data']['maps'][to]['events']['warps'] if w['index'] == port['target_warp_id'])
        walk(c,(port['x'],port['y']+1));c.move('up');c.at(to,arrival['x'],arrival['y'])
        c.call('hold_button',button='b',frames=40);c.at(to,arrival['x'],arrival['y'])
        return to,arrival['index']-1

    def pair(c,index):
        to,back = cross(c,ROOM,index);cross(c,to,back)

    def release(c):
        for _ in range(96):
            if c.status()['dialogue']:
                pages.append(finish(c));continue
            c.press('start');view=c.observe()['structuredContent']
            if any('SAVE' in x['text'].upper() for x in view.get('menu',[])):
                c.press('b');return
            c.press('a')
        raise AssertionError('ordinary input not released')

    try:
        for continued in [False,True]:
            with (args.output/f'stderr-{int(continued)}.log').open('w+') as stderr:
                process = subprocess.Popen([str(args.client.resolve()),'mcp',str(args.pack.resolve()),'--load',str((live if continued else args.save).resolve()),'--save',str(live.resolve())],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,text=True,bufsize=1)
                try:
                    c = LiveRecorded(process,calls)
                    c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='sump-basin',version='1')))
                    if not continued:
                        c.at(ROOM,12,12);release(c);c.move('left');c.at(ROOM,12,12)
                        hooked = False
                        for cast in range(30):
                            c.press('select');c.call('hold_button',button='up',frames=120)
                            for page in range(30):
                                view=c.observe()['structuredContent'];menu=' '.join(x['text'] for x in view.get('menu',[])).upper()
                                if all(w in menu for w in ['FIGHT','PKMN','PACK','RUN']):hooked=True;break
                                if c.status()['mode'] == 'overworld' and 'NO_WILD_BATTLE' in verify():break
                                c.press('a')
                            if hooked:break
                        assert hooked,'native registered Old Rod never produced a bite'
                        proof=verify();assert 'WILD=' in proof and 'MAP=MnIronSump' in proof
                        shutil.copy2(live,args.output/'hooked-fish.crystalsave')
                        (args.output/'hooked-fish.txt').write_text(proof)
                        c.press('down');c.press('right');c.press('a')
                        for _ in range(128):
                            if c.status()['mode'] == 'overworld' and 'NO_WILD_BATTLE' in verify():break
                            c.press('b')
                        else:raise AssertionError('native Run did not return to the bank')
                        release(c);c.at(ROOM,12,12)
                        c.press('left');c.press('a')
                        for _ in range(96):
                            s=c.status()
                            if s['position'] == dict(x=11,y=12) and 'MODE=Surf' in verify():break
                            c.press('a')
                        else:raise AssertionError('native Surf did not launch')
                        release(c)
                        for y in [13,14]:c.move('down');c.at(ROOM,11,y)
                        c.move('down');c.at(ROOM,11,14)
                        assert 'MODE=Surf' in verify()
                    else:
                        c.at(ROOM,11,14);assert 'MODE=Surf' in verify()
                        c.move('down');c.at(ROOM,11,14)
                        for y in [13,12]:c.move('up');c.at(ROOM,11,y)
                        c.move('right');c.at(ROOM,12,12);assert 'MODE=Normal' in verify()
                        pair(c,1);pair(c,3)
                        cross(c,ROOM,3);cross(c,'MnIronVentilation',7)
                        for index in [5,7,9]:pair(c,index)
                        cross(c,ROOM,5);cross(c,'MnIronHaulage',1)
                        c.at('MnIronDrillApproach',24,5);release(c)
                finally:
                    process.stdin.close();assert process.wait(timeout=30) == 0
                    stderr.flush();assert stderr.tell() == 0,'gameplay stderr must stay empty'
        assert all(sha(getattr(args,k)) == v for k,v in bindings.items())
        final=verify();assert 'MODE=Normal' in final
        for flag in ['EVENT_MN_DRILL_FEED_ISOLATED','EVENT_MN_DRILL_TEST_WITNESSED','EVENT_MN_DRILL_STOPPED','EVENT_MN_MINE_INSTRUMENT_RECOVERED']:assert flag+'=false' in final
        report.update(status='PASS',rpc_calls=len(calls),auxiliary_pages=pages,final_native_save=final,fresh_process_continue=True)
        print('PASS',len(calls),'RPC')
    finally:
        (args.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n')


if __name__ == '__main__':main()
