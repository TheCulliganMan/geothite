#!/usr/bin/env python3
"""Replay actual ventilation valve, climbs, four shaft pairs and Continue over stdio."""
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
from check_north_shore_story_mcp import finish, choose
from render_reference import load_pack

ROOM = 'MnIronVentilation'


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
                  fixture='Actual reached declined ventilation valve after both real local guards, the real haulage control and shaft travel; declared initial local party/access setup.')

    def verify(path=live):
        p = subprocess.run([str(args.verifier.resolve()),str(idfile.resolve()),bindings['pack'],identity['content_hash'],str(path.resolve())],capture_output=True,text=True)
        assert p.returncode == 0,(p.stdout,p.stderr)
        return p.stdout

    initial = verify(args.save)
    assert 'MODE=Normal' in initial
    assert all(flag+'=true' in initial for flag in ['EVENT_MN_IRON_HAUL_PRESSURE_OFF','EVENT_MN_LODE_ARC_VENT_DEFEATED','EVENT_MN_LODE_ARC_SHIFT_DEFEATED'])
    assert 'EVENT_MN_IRON_VENT_PRESSURE_OFF=false' in initial
    # In-memory expected geometry only: the actual reached haulage valve
    # is already off. No game state or pack is changed by this planner.
    pack['data']['maps']['MnIronHaulage']['blocks'][8*18+9] = 13

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
                    c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='ventilation-cleft',version='1')))
                    if not continued:
                        c.at(ROOM,13,16);release(c)
                        walk(c,(17,13));c.move('right');c.at(ROOM,17,13)
                        walk(c,(13,16));c.press('up');c.press('a')
                        choose(c,False);pages.append(finish(c))
                        assert 'EVENT_MN_IRON_VENT_PRESSURE_OFF=false' in verify()
                        walk(c,(17,13));c.move('right');c.at(ROOM,17,13)
                        walk(c,(13,16));c.press('up');c.press('a')
                        choose(c,True);pages.append(finish(c))
                        assert 'EVENT_MN_IRON_VENT_PRESSURE_OFF=true' in verify()
                        pack['data']['maps'][ROOM]['blocks'][6*18+9] = 7
                        pack['data']['maps']['MnIronSump']['blocks'][3*20+16] = 13
                        walk(c,(17,13))
                        for x in [18,19,20]:c.move('right');c.at(ROOM,x,13)
                        assert 'MODE=Normal' in verify()
                    else:
                        c.at(ROOM,20,13);assert 'EVENT_MN_IRON_VENT_PRESSURE_OFF=true' in verify()
                        for x,bottom,top in [(8,10,7),(28,8,5)]:
                            walk(c,(x,bottom))
                            for y in range(bottom-1,top-1,-1):c.move('up');c.at(ROOM,x,y)
                            for y in range(top+1,bottom+1):c.move('down');c.at(ROOM,x,y)
                        for at in [(5,13),(5,17),(9,17),(9,13)]:walk(c,at)
                        for index in [1,3,5,7]:pair(c,index)
                        cross(c,ROOM,7);walk(c,(34,5))
                        cross(c,'MnIronSump',7);cross(c,'MnIronPumpGallery',1)
                        cross(c,'MnIronSump',5);cross(c,'MnIronHaulage',1)
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
