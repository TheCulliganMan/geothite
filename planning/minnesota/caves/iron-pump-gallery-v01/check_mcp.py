#!/usr/bin/env python3
"""Replay the feed console, both physical openings, climbs and two shaft pairs."""
import argparse
from collections import deque
import hashlib
import json
import re
import shutil
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from check_mina_battle_mcp import LiveRecorded
from check_north_shore_story_mcp import finish, choose
from render_reference import load_pack

ROOM = 'MnIronPumpGallery'


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
                  fixture='Actual reached declined feed after both real pump guards and both real pressure controls; initial prior foreman chapter/local party/access setup is explicit.')

    def verify(path=live):
        p = subprocess.run([str(args.verifier.resolve()),str(idfile.resolve()),bindings['pack'],identity['content_hash'],str(path.resolve())],capture_output=True,text=True)
        assert p.returncode == 0,(p.stdout,p.stderr)
        return p.stdout

    initial = verify(args.save)
    assert 'MODE=Normal' in initial
    assert all(flag+'=true' in initial for flag in ['EVENT_MN_DEPOT_FOREMAN_DEFEATED','EVENT_MN_IRON_HAUL_PRESSURE_OFF','EVENT_MN_IRON_VENT_PRESSURE_OFF','EVENT_MN_LODE_ARC_PUMP_DEFEATED','EVENT_MN_LODE_ARC_INTAKE_DEFEATED'])
    assert 'EVENT_MN_DRILL_FEED_ISOLATED=false' in initial
    assert 'PARTY_SURF_MOVES=0' in initial
    # In-memory expected geometry only: the actual reached haulage/ventilation valves
    # are already off. No game state or pack is changed by this planner.
    pack['data']['maps']['MnIronHaulage']['blocks'][8*18+9] = 13
    pack['data']['maps']['MnIronVentilation']['blocks'][6*18+9] = 7
    pack['data']['maps']['MnIronSump']['blocks'][3*20+16] = 13

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

    def battle_finish(c):
        for _ in range(512):
            status = c.status()
            if status['mode'] == 'overworld' and not status['dialogue']:return
            view = c.observe()['structuredContent']
            lines = [row['text'].strip() for row in view['menu']]
            prompt = [row['text'].strip() for row in view['prompt']]
            selected = next((line[2:].strip() for line in lines+prompt if line.startswith('> ')),None)
            if any(line.lstrip('> ').strip() == 'YES' for line in lines+prompt):
                choose(c,False);continue
            if any(line.lstrip('> ').strip() == 'FIGHT' for line in lines):
                if selected != 'FIGHT':c.press('up');continue
                c.press('a');continue
            moves = [(i,re.search(r'PP\s+(\d+)\s*/',line)) for i,line in enumerate(lines)]
            usable = [(i,match) for i,match in moves if match and int(match.group(1)) > 0]
            if usable:
                target = usable[0][0]
                cursor = next((i for i,line in enumerate(lines) if line.startswith('> ')),None)
                assert cursor is not None,'visible move cursor'
                if cursor != target:c.press('down');continue
            c.press('a')
        raise AssertionError('native core guard did not release input')

    def cross(c,name,index):
        port = next(w for w in pack['data']['maps'][name]['events']['warps'] if w['index'] == index)
        to = constants[port['target_map_constant']]
        arrival = next(w for w in pack['data']['maps'][to]['events']['warps'] if w['index'] == port['target_warp_id'])
        walk(c,(port['x'],port['y']+1))
        for _ in range(3):
            c.move('up')
            if c.status()['map'] == to:break
            assert name == 'MnIronDrillCore','unexpected shaft interruption'
            battle_finish(c)
            if c.status()['map'] == to:break
            walk(c,(port['x'],port['y']+1))
        c.at(to,arrival['x'],arrival['y'])
        c.call('hold_button',button='b',frames=40);c.at(to,arrival['x'],arrival['y'])
        return to,arrival['index']-1

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
        for phase in range(3):
            with (args.output/f'stderr-{phase}.log').open('w+') as stderr:
                process = subprocess.Popen([str(args.client.resolve()),'mcp',str(args.pack.resolve()),'--load',str((args.save if phase == 0 else live).resolve()),'--save',str(live.resolve())],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,text=True,bufsize=1)
                try:
                    c = LiveRecorded(process,calls)
                    c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='pump-gallery',version='1')))
                    if phase == 0:
                        c.at(ROOM,25,18);release(c)
                        walk(c,(16,17));c.move('left');c.at(ROOM,16,17)
                        shutil.copy2(live,args.output/'closed-service-shutter.crystalsave')
                        walk(c,(24,8));c.move('up');c.at(ROOM,24,8)
                        walk(c,(25,18));c.press('up');c.press('a')
                        choose(c,False);pages.append(finish(c))
                        assert 'EVENT_MN_DRILL_FEED_ISOLATED=false' in verify()
                    elif phase == 1:
                        c.at(ROOM,25,18);release(c)
                        assert 'EVENT_MN_DRILL_FEED_ISOLATED=false' in verify()
                        c.press('up');c.press('a');choose(c,True);pages.append(finish(c))
                        assert 'EVENT_MN_DRILL_FEED_ISOLATED=true' in verify()
                        for x,y in [(12,3),(7,8)]:pack['data']['maps'][ROOM]['blocks'][y*16+x] = 13
                        walk(c,(16,17))
                        shutil.copy2(live,args.output/'open-service-shutter.crystalsave')
                        for x in [15,14,13]:c.move('left');c.at(ROOM,x,17)
                        for x in [6,16]:
                            walk(c,(x,10))
                            for y in [9,8,7]:c.move('up');c.at(ROOM,x,y)
                            for y in [8,9,10]:c.move('down');c.at(ROOM,x,y)
                        walk(c,(24,8))
                        for y in [7,6,5]:c.move('up');c.at(ROOM,24,y)
                        cross(c,ROOM,3);cross(c,'MnIronDrillCore',1)
                        cross(c,ROOM,1);cross(c,'MnIronSump',7)
                        c.at(ROOM,4,19)
                    else:
                        c.at(ROOM,4,19);release(c)
                        assert 'EVENT_MN_DRILL_FEED_ISOLATED=true' in verify()
                        walk(c,(25,18));c.press('up');c.press('a');pages.append(finish(c))
                        cross(c,ROOM,3);cross(c,'MnIronDrillCore',1)
                        cross(c,ROOM,1);cross(c,'MnIronSump',5);cross(c,'MnIronHaulage',1)
                        c.at('MnIronDrillApproach',24,5);release(c)
                    assert 'PARTY_SURF_MOVES=0' in verify()
                    assert 'MODE=Normal' in verify()
                finally:
                    process.stdin.close();assert process.wait(timeout=30) == 0
                    stderr.flush();assert stderr.tell() == 0,'gameplay stderr must stay empty'
        assert all(sha(getattr(args,k)) == v for k,v in bindings.items())
        final=verify();assert 'EVENT_MN_DRILL_FEED_ISOLATED=true' in final
        for flag in ['EVENT_MN_DRILL_TEST_WITNESSED','EVENT_MN_REED_MINE_DEFEATED','EVENT_MN_DRILL_STOPPED','EVENT_MN_MINE_INSTRUMENT_RECOVERED']:assert flag+'=false' in final
        report.update(status='PASS',rpc_calls=len(calls),auxiliary_pages=pages,final_native_save=final,fresh_process_continue=True)
        print('PASS',len(calls),'RPC')
    except BaseException as error:
        report.update(status='FAIL',error=repr(error))
        raise
    finally:
        (args.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n')


if __name__ == '__main__':main()
