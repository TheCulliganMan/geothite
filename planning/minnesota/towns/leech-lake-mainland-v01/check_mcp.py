#!/usr/bin/env python3
"""Public stdio cabin trail, declined fly-tier, reward, ordinary return and Continue.

Only the supplied no-Surf initial native fixture provides prior party/badges/position.
Every later state change uses public MCP buttons. No save or pack mutation.
"""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import traceback
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'tools'))
from render_reference import load_pack
from check_mina_battle_mcp import LiveRecorded
from check_lake_country_mcp import escape_wild
from check_north_shore_story_mcp import choose, finish

cli = argparse.ArgumentParser(description=__doc__)
for key in ['pack', 'save', 'client', 'output']:
    cli.add_argument('--' + key, type=Path, required=True)
a = cli.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
pack = load_pack(a.pack)
calls, checkpoints = [], []
error, process = None, None
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
frozen = {str(p): sha(p) for p in [a.pack, a.save, a.client]}
autosave = a.output / 'actual-played.crystalsave'

def pos(c):
    s = c.status()
    assert s['map'] == 'MnLeechLake', s
    return s['position']['x'], s['position']['y']

def step(c, direction, target):
    for _ in range(12):
        if c.status()['mode'] in {'battle', 'wildbattle'}:
            escape_wild(c)
        if pos(c) == target:
            return
        c.move(direction)
    assert pos(c) == target, (direction, target, c.status())

def go(c, target, grass=True):
    module = pack['data']['maps']['MnLeechLake']
    collision = pack['data']['tilesets']['mn_headwaters']['collision']
    start = pos(c)
    todo, previous = deque([start]), {start: None}
    bodies = {(o['x'], o['y']) for o in module['objects']}
    while todo and target not in previous:
        x, y = todo.popleft()
        for q, direction in [((x-1,y),'left'), ((x+1,y),'right'),
                             ((x,y-1),'up'), ((x,y+1),'down')]:
            if q in previous or q in bodies or not (0 <= q[0] < 64 and 0 <= q[1] < 48):
                continue
            block = module['blocks'][q[1]//2*32+q[0]//2]
            token = collision[f'{block:02x}'][q[1]%2*2+q[0]%2]
            if token not in ({'FLOOR','TALL_GRASS'} if grass else {'FLOOR'}):
                continue
            previous[q] = ((x,y), direction)
            todo.append(q)
    assert target in previous, (start, target, grass)
    route, q = [], target
    while previous[q] is not None:
        p, direction = previous[q]
        route.append((q,direction))
        q = p
    for target, direction in reversed(route):
        step(c,direction,target)

def checkpoint(c, name):
    c.at('MnLeechLake', *pos(c))
    p = a.output / (name + '.crystalsave')
    assert not p.exists()
    p.write_bytes(autosave.read_bytes())
    checkpoints.append(dict(name=name, position=pos(c), save=str(p), sha256=sha(p)))

try:
    with (a.output / 'stderr.log').open('x') as err:
        def start(save, output):
            process = subprocess.Popen([str(a.client.resolve()), 'mcp', str(a.pack),
                '--load', str(save), '--save', str(output)], stdin=subprocess.PIPE,
                stdout=subprocess.PIPE, stderr=err, text=True, bufsize=1)
            c = LiveRecorded(process, calls)
            c.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                clientInfo=dict(name='leech-mainland-no-surf-trip', version='1')))
            return process, c
        process, c = start(a.save, autosave)
        c.at('MnLeechLake', 11, 14)
        initial = c.status()
        go(c, (16,22)); checkpoint(c, 'mainland-grass-link')
        go(c, (18,24)); c.press('right'); c.press('a'); choose(c,False); finish(c)
        checkpoint(c, 'mainland-fly-tier-declined')
        go(c, (18,26)); c.press('right'); c.press('a')
        assert 'SUPER REPEL' in finish(c).upper()
        checkpoint(c, 'mainland-reward-claimed')
        go(c, (9,36)); checkpoint(c, 'mainland-fisher-porch')
        go(c, (35,22),False); c.press('up'); c.press('a')
        assert 'empty this week.' in finish(c)
        go(c, (28,29),False); c.press('left'); c.press('a')
        assert 'joins both cabins.' in finish(c)
        go(c, (11,14),False); checkpoint(c, 'ordinary-public-return')
        assert c.status()['money'] == initial['money'] and c.status()['badges'] == initial['badges']
        c.press('start')
        assert any('PACK' in row['text'] for row in c.observe()['structuredContent']['menu'])
        c.press('b'); process.stdin.close(); assert process.wait(timeout=30) == 0
        process, c = start(autosave, a.output / 'continued.crystalsave')
        c.at('MnLeechLake',11,14)
        c.move('right'); c.at('MnLeechLake',12,14)
        process.stdin.close(); assert process.wait(timeout=30) == 0
except BaseException as e:
    error = repr(e)
    traceback.print_exc()
finally:
    if process and process.poll() is None:
        process.terminate(); process.wait(timeout=30)
    unchanged = all(sha(Path(p)) == digest for p,digest in frozen.items())
    stderr = (a.output / 'stderr.log').read_bytes()
    report = dict(status='PASS' if error is None and unchanged and not stderr else 'FAIL',
        error=error, inputs=frozen, inputs_unchanged=unchanged, rpc_calls=len(calls),
        stderr_bytes=len(stderr), checkpoints=checkpoints, calls=calls, scope=__doc__)
    (a.output / 'receipt.json').write_text(json.dumps(report,indent=2) + '\n')
    print(report['status'],error,len(calls),'RPC',flush=True)
    assert report['status'] == 'PASS',error
