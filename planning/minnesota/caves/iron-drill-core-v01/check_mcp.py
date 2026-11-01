#!/usr/bin/env python3
"""Replay native drill frames, choices and Continue from actual chapter saves."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT/'planning/minnesota/tools'))
from check_mina_battle_mcp import LiveRecorded
from check_north_shore_story_mcp import choose, finish
from render_reference import load_pack

CORE = 'MnIronDrillCore'
SEEN = 'EVENT_MN_DRILL_TEST_WITNESSED'
WON = 'EVENT_MN_REED_MINE_DEFEATED'
OFF = 'EVENT_MN_DRILL_STOPPED'
RECOVERED = 'EVENT_MN_MINE_INSTRUMENT_RECOVERED'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['pack', 'feed-save', 'rival-save', 'client', 'verifier', 'output']:
        parser.add_argument('--'+key, type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    pack = load_pack(args.pack)
    identity = pack['identity']
    idfile = args.output/'native-modpack-id.txt'
    idfile.write_text(identity['runtime_modpack_id'])
    bindings = {key: sha(getattr(args, key)) for key in
                ['pack', 'feed_save', 'rival_save', 'client', 'verifier']}
    live = args.output/'live.crystalsave'
    records, pages, native = [], [], []
    report = dict(status='IN_PROGRESS', bindings=bindings, calls=records,
                  fixture='Two separately reached checkpoints from the actual full controller chapter: isolated feed and defeated rival. Prior regional access and party setup are declared in that chapter test. No state mutation after input; this is not a continuous MCP campaign.')

    def verify(path=live):
        result = subprocess.run([str(args.verifier.resolve()), str(idfile.resolve()),
                                 bindings['pack'], identity['content_hash'], str(path.resolve())],
                                capture_output=True, text=True)
        assert result.returncode == 0, (result.stdout, result.stderr)
        assert 'PARTY_SURF_MOVES=0' in result.stdout
        assert 'MODE=Normal' in result.stdout
        return result.stdout

    def flags(text, **expected):
        names = dict(seen=SEEN, won=WON, off=OFF, recovered=RECOVERED)
        for key, value in expected.items():
            assert names[key]+'='+str(value).lower() in text, (key, text)

    def checkpoint(label):
        text = verify()
        native.append(dict(label=label, state=text))
        shutil.copy2(live, args.output/(label+'.crystalsave'))
        return text

    def walk(client, target):
        for _ in range(256):
            status = client.status()
            assert status['mode'] == 'overworld' and not status['dialogue'], status
            start = status['position']['x'], status['position']['y']
            if start == target:
                return
            module = pack['data']['maps'][status['map']]
            attrs = module['attributes']
            collision = pack['data']['tilesets'][attrs['tileset_name']]['collision']
            bodies = {(o['x'], o['y']) for o in module['objects']
                      if o['script'] != 'MN_NDRILL_FINN'}
            if status['map'] == CORE and OFF+'=true' in verify():
                bodies.add((8, 7))
            queue, paths = deque([start]), {start: []}
            while queue and target not in paths:
                x, y = queue.popleft()
                for dx, dy, key in [(1,0,'right'), (-1,0,'left'), (0,1,'down'), (0,-1,'up')]:
                    at = x+dx, y+dy
                    if at in paths or at in bodies or not (0 <= at[0] < attrs['width']*2 and 0 <= at[1] < attrs['height']*2):
                        continue
                    block = module['blocks'][at[1]//2*attrs['width']+at[0]//2]
                    if collision[f'{block:02x}'][at[1]%2*2+at[0]%2] == 'FLOOR':
                        paths[at] = paths[(x,y)]+[key]
                        queue.append(at)
            assert target in paths, (status['map'], start, target)
            client.move(paths[target][0])
        raise AssertionError('native walking did not arrive')

    # Expected geometry in the read-only planner reflects the already-earned
    # feed callback. The actual save and game obtain these openings themselves.
    for x, y in [(12,3), (7,8)]:
        pack['data']['maps']['MnIronPumpGallery']['blocks'][y*16+x] = 13

    process = None
    try:
        flags(verify(args.feed_save), seen=False, won=False, off=False, recovered=False)
        flags(verify(args.rival_save), seen=True, won=True, off=False, recovered=False)
        for phase in range(4):
            source = args.feed_save if phase == 0 else args.rival_save if phase == 1 else live
            with (args.output/f'stderr-{phase}.log').open('x') as stderr:
                process = subprocess.Popen([str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                                            '--load', str(source.resolve()), '--save', str(live.resolve())],
                                           stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                                           text=True, bufsize=1)
                client = LiveRecorded(process, records)
                client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                             clientInfo=dict(name='native-drill-core', version='1')))
                if phase == 0:
                    client.at('MnIronPumpGallery', 25, 18)
                    walk(client, (25,5)); client.move('up'); client.at(CORE,4,15)
                    walk(client, (4,13)); client.move('up'); client.at(CORE,9,9)
                    for frame in [22,18]:
                        for _ in range(48):
                            text = verify()
                            if f'(5, 4): {frame}' in text:
                                break
                            client.press('a')
                        else:
                            raise AssertionError('native witness frame absent')
                        flags(text, seen=False, won=False, off=False, recovered=False)
                        assert client.status()['dialogue']
                        client.press('left'); client.at(CORE,9,9)
                        checkpoint(f'played-piston-{frame:02x}')
                        pages.append(client.observe())
                        client.press('a')
                    pages.append(finish(client))
                    flags(checkpoint('witnessed-only'), seen=True, won=False, off=False, recovered=False)
                    walk(client,(16,3)); client.press('up'); client.press('a')
                    pages.append(finish(client))
                    flags(checkpoint('power-denied-before-rival'), off=False, recovered=False)
                elif phase == 1:
                    client.at(CORE,8,9)
                    walk(client,(16,3)); client.press('up'); client.press('a')
                    choose(client,False); pages.append(finish(client))
                    flags(checkpoint('shutdown-declined'), seen=True, won=True, off=False, recovered=False)
                elif phase == 2:
                    client.at(CORE,16,3)
                    client.press('up'); client.press('a'); choose(client,True)
                    pages.append(finish(client)); client.at(CORE,9,9)
                    text = checkpoint('shutdown-crew-at-west')
                    flags(text, off=True, recovered=False)
                    assert 'FINN_LIVE_X=8 Y=7 VISIBLE=true' in text and 'HIDDEN=false' in text
                    walk(client,(8,6)); client.press('down'); client.press('a')
                    choose(client,False); pages.append(finish(client))
                    flags(checkpoint('crew-handoff-declined'), off=True, recovered=False)
                else:
                    client.at(CORE,8,6)
                    text = checkpoint('continued-crew-restored')
                    flags(text, off=True, recovered=False)
                    assert 'FINN_LIVE_X=8 Y=7 VISIBLE=true' in text and 'HIDDEN=false' in text
                    client.press('down'); client.press('a'); choose(client,True)
                    for _ in range(48):
                        view = client.observe()
                        if 'ELLIS built' in json.dumps(view):
                            break
                        client.press('a')
                    else:
                        raise AssertionError('crew quest waitbutton absent')
                    pages.append(view)
                    flags(checkpoint('handoff-before-acknowledgement'), recovered=False)
                    pages.append(finish(client))
                    flags(checkpoint('handoff-acknowledged'), recovered=True)
                    client.press('down'); client.press('a'); pages.append(finish(client))
                    client.press('start')
                    assert any('PACK' in row['text'] for row in client.observe()['structuredContent']['menu'])
                    client.press('b')
                process.stdin.close()
                assert process.wait(timeout=30) == 0
                process = None
            assert (args.output/f'stderr-{phase}.log').stat().st_size == 0
        assert all(sha(getattr(args,key)) == value for key,value in bindings.items())
        report.update(status='PASS', rpc_calls=len(records), native_checkpoints=native,
                      observed_pages=pages, fresh_process_continue=True)
    except BaseException as error:
        report.update(status='FAIL', error=repr(error), native_checkpoints=native, observed_pages=pages)
        raise
    finally:
        if process is not None and process.poll() is None:
            process.terminate(); process.wait(timeout=30)
        (args.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n')
    print('PASS native witness, denial/refusal/retry, crew acknowledgement and Continue;',len(records),'RPC')


if __name__ == '__main__':
    main()
