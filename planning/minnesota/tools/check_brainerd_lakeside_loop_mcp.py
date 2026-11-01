#!/usr/bin/env python3
"""Replay Brainerd's lake street and rod bank from actual controller saves.

Local companions, rod and upstream permission are explicit producer fixtures.
No save, party, progress, money, position or RNG is rewritten by this tool.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

from check_lake_country_mcp import escape_wild
from check_mina_battle_mcp import LiveRecorded
from check_north_shore_story_mcp import finish
from check_winona_mcp import walk
from render_reference import load_pack

CITY = 'MnBrainerd'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def frontage(pack, room):
    town = pack['data']['maps'][CITY]
    inside = pack['data']['maps'][room]
    door, = [w for w in town['events']['warps']
             if w['index'] % 2 == 1 and w['target_map_constant'] == inside['attributes']['map_constant']]
    outside, = [w for w in town['events']['warps'] if w['index'] == door['index'] + 1]
    assert (outside['x'], outside['y']) == (door['x'], door['y'] + 1)
    return outside['x'], outside['y']


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    for key in ['pack', 'client', 'doors-save', 'bank-save', 'output']:
        cli.add_argument('--' + key, type=Path, required=True)
    args = cli.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    pack = load_pack(args.pack)
    inputs = {str(p): sha(p) for p in [args.pack, args.client, args.doors_save, args.bank_save]}
    results = []
    for case in ['town-loop', 'registered-fishing', 'fresh-process-continue']:
        directory = args.output / case
        directory.mkdir()
        source = (args.doors_save if case == 'town-loop' else args.bank_save
                  if case == 'registered-fishing'
                  else args.output / 'registered-fishing/after-native-run.crystalsave')
        saved = directory / 'actual-autosave.crystalsave'
        records, error, process = [], None, None
        try:
            with (directory / 'stderr.log').open('x') as stderr:
                process = subprocess.Popen(
                    [str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                     '--load', str(source.resolve()), '--save', str(saved.resolve())],
                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                    text=True, bufsize=1)
                client = LiveRecorded(process, records)
                client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                           clientInfo=dict(name='brainerd-lakeside-loop', version='1')))
                if case == 'town-loop':
                    client.at(CITY, *frontage(pack, 'MnBrainerdPokecenter1F'))
                    before = client.status()
                    for room in ['MnBrainerdGym', 'MnBrainerdRangerStation',
                                 'MnBrainerdOutfitter', 'MnBrainerdMart',
                                 'MnBrainerdPokecenter1F']:
                        walk(client, pack, frontage(pack, room))
                    for post in pack['data']['maps'][CITY]['events']['bg_events']:
                        stand = post['x'] + 1, post['y']
                        walk(client, pack, stand)
                        client.press('left')
                        client.at(CITY, *stand)
                        client.press('a')
                        assert client.status()['dialogue']
                        client.move('down')
                        client.at(CITY, *stand)
                        finish(client)
                    walk(client, pack, (32, 4))
                    client.move('up')
                    assert client.status()['dialogue']
                    finish(client)
                    client.at(CITY, 32, 4)
                    for point in [(40, 25), (47, 20), (44, 20), (33, 32)]:
                        walk(client, pack, point)
                    walk(client, pack, frontage(pack, 'MnBrainerdPokecenter1F'))
                    after = client.status()
                    assert after['badges'] == before['badges'] == 0
                    assert after['money'] == before['money']
                elif case == 'registered-fishing':
                    client.at(CITY, 44, 20)
                    client.press('up')
                    client.at(CITY, 44, 20)
                    before = client.status()
                    assert before['badges'] == 0 and before['facing'] == 'up'
                    bite = False
                    for _ in range(64):
                        client.press('select')
                        client.call('hold_button', button='left', frames=120)
                        notice = client.observe()['content'][0]['text']
                        bite = 'A bite!' in notice
                        assert bite or 'Not even a nibble!' in notice, notice
                        after = client.status()
                        for key in ['map', 'position', 'facing', 'money', 'badges']:
                            assert after[key] == before[key], (key, before, after)
                        client.press('a')
                        if bite:
                            assert client.status()['mode'] == 'wildbattle'
                            view = client.observe()
                            assert any(species in json.dumps(view) for species in ['SCUTLET', 'CLINKLAW'])
                            (directory / 'actual-fish-observe.json').write_text(json.dumps(view, indent=2) + '\n')
                            escape_wild(client)
                            break
                    assert bite, 'native Old Rod must produce a shore encounter'
                    client.at(CITY, 44, 20)
                    assert client.status()['mode'] == 'overworld'
                    shutil.copy2(saved, directory / 'after-native-run.crystalsave')
                else:
                    client.at(CITY, 44, 20)
                    assert client.status()['mode'] == 'overworld'
                    client.move('down')
                    client.at(CITY, 44, 21)
                    client.move('up')
                    client.at(CITY, 44, 20)
                client.press('start')
                assert any('PACK' in row['text'] for row in client.observe()['structuredContent']['menu'])
                client.press('b')
                process.stdin.close()
                assert process.wait(timeout=30) == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if process is not None and process.poll() is None:
                process.terminate()
                process.wait(timeout=30)
        stderr = (directory / 'stderr.log').read_bytes()
        result = dict(case=case, status='PASS' if error is None and not stderr else 'FAIL',
                      error=error, stderr_bytes=len(stderr), rpc_calls=len(records), records=records)
        (directory / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
        results.append({k: v for k, v in result.items() if k != 'records'})
        unchanged = all(sha(Path(path)) == value for path, value in inputs.items())
        (args.output / 'receipt.json').write_text(json.dumps(dict(
            inputs=inputs, inputs_unchanged=unchanged, results=results, scope=__doc__), indent=2) + '\n')
        assert result['status'] == 'PASS' and unchanged, error
        print(case, 'PASS', len(records), 'calls', flush=True)


if __name__ == '__main__':
    main()
