#!/usr/bin/env python3
"""Play both lip lanes, the bench and paired ladders through public stdio MCP.

The native initial fixture declares cleared guards, party and public-shaft access.
All later actions are production buttons, including fresh-process Continue.
"""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'tools'))
from check_mina_battle_mcp import LiveRecorded
from check_lake_country_mcp import escape_wild
from check_north_shore_story_mcp import finish
from render_reference import load_pack

cli = argparse.ArgumentParser(description=__doc__)
for name in ['pack','save','client','output']:
    cli.add_argument('--'+name, type=Path, required=True)
a = cli.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
pack = load_pack(a.pack)
calls, checkpoints = [], []
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
inputs = {str(p): sha(p) for p in [a.pack,a.save,a.client]}
live = a.output/'actual-played.crystalsave'
process, error = None, None
room = 'MnIronDrillApproach'

def position(c):
    s = c.status()
    return s['position']['x'], s['position']['y']

def step(c, direction, target):
    for _ in range(12):
        if c.status()['mode'] in {'battle','wildbattle'}:
            escape_wild(c)
        if position(c) == target:
            return
        c.move(direction)
    assert position(c) == target, (direction,target,c.status())

def go(c, target):
    m = pack['data']['maps'][room]
    collision = pack['data']['tilesets']['cave']['collision']
    bodies = {(o['x'],o['y']) for o in m['objects']}
    start = position(c)
    queue, previous = deque([start]), {start:None}
    while queue and target not in previous:
        x,y = queue.popleft()
        for q,button in [((x-1,y),'left'),((x+1,y),'right'),
                         ((x,y-1),'up'),((x,y+1),'down')]:
            if q in previous or q in bodies or not (0<=q[0]<32 and 0<=q[1]<26):
                continue
            block = m['blocks'][q[1]//2*16+q[0]//2]
            if collision[f'{block:02x}'][q[1]%2*2+q[0]%2] != 'FLOOR':
                continue
            previous[q] = ((x,y),button)
            queue.append(q)
    assert target in previous, (start,target)
    route, q = [], target
    while previous[q] is not None:
        p,button = previous[q]
        route.append((q,button))
        q = p
    for q,button in reversed(route):
        step(c,button,q)
        c.at(room,*q)

def checkpoint(c, label):
    s = c.status()
    p = a.output/(label+'.crystalsave')
    assert not p.exists()
    shutil.copy2(live,p)
    checkpoints.append(dict(label=label,map=s['map'],position=position(c),save=str(p),sha256=sha(p)))

try:
    for phase in range(2):
        with (a.output/f'stderr-{phase}.log').open('x') as err:
            process = subprocess.Popen([str(a.client.resolve()),'mcp',str(a.pack.resolve()),
                '--load',str((a.save if phase==0 else live).resolve()),'--save',str(live.resolve())],
                stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=err,text=True,bufsize=1)
            c = LiveRecorded(process,calls)
            c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='fitted-iron-crosscut',version='1')))
            if phase == 0:
                c.at(room,6,21)
                for target in [(9,13),(4,5),(6,10),(15,10),(19,13),(25,18),(14,18)]:
                    go(c,target)
                step(c,'left',(13,18)); checkpoint(c,'actual-upper-lip')
                step(c,'left',(11,18));c.move('right');c.at(room,11,18)
                checkpoint(c,'actual-upper-landing')
            else:
                c.at(room,11,18);c.move('right');c.at(room,11,18)
                go(c,(14,19));step(c,'left',(13,19));checkpoint(c,'actual-lower-lip')
                step(c,'left',(11,19));c.move('right');c.at(room,11,19)
                checkpoint(c,'actual-lower-landing')
                go(c,(6,10))
                for y in [9,8,7]:step(c,'up',(6,y))
                for y in [8,9,10]:step(c,'down',(6,y))
                go(c,(24,5));c.move('right');c.at('MnIronHaulage',4,23)
                checkpoint(c,'real-haulage-arrival')
                c.move('right');c.at('MnIronHaulage',5,23)
                c.move('up');c.at(room,24,5)
                go(c,(6,21));c.move('right');c.at('MnR09Cave1F',19,9)
                checkpoint(c,'real-public-mine-arrival')
                c.move('down');c.at('MnR09Cave1F',19,10)
                c.move('down');c.at(room,6,21)
                c.press('start');assert any('PACK' in row['text'] for row in c.observe()['structuredContent']['menu']);c.press('b')
                checkpoint(c,'actual-reentry-and-start')
            process.stdin.close();assert process.wait(timeout=30)==0;process=None
        assert (a.output/f'stderr-{phase}.log').stat().st_size==0
except BaseException as exc:
    error = repr(exc)
    raise
finally:
    if process is not None and process.poll() is None:
        process.terminate();process.wait(timeout=30)
    unchanged = all(sha(Path(p)) == h for p,h in inputs.items())
    report = dict(status='PASS' if error is None and unchanged else 'FAIL',error=error,
                  inputs=inputs,inputs_unchanged=unchanged,calls=calls,rpc_calls=len(calls),
                  checkpoints=checkpoints,fresh_process_continue=True,
                  scope='Declared cleared guards, native party and prior public-shaft access before input. Both one-way lanes/reverse blocks, long floor returns, bench stair, real ladder pairs and Continue use production input. No earned campaign claim.')
    (a.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS public crosscut trip;',len(calls),'RPC')
