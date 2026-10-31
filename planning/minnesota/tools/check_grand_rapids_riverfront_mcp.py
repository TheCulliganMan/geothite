#!/usr/bin/env python3
"""Replay the town loop, optional refusal and registered fishing over stdio.

Inputs are actual identity-bound saves emitted by the controller regressions.
This tool never edits those saves, game state, party, flags, money or RNG.
The controller tests separately prove battle PP/prize, services and story gates.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

from check_lake_country_mcp import escape_wild
from check_mina_battle_mcp import LiveRecorded
from check_north_shore_story_mcp import choose, finish
from check_winona_mcp import walk
from render_reference import load_pack

TOWN = 'MnGrandRapids'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'client', 'saves', 'inspector', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    pack = load_pack(args.pack)
    identity = pack['identity']
    idfile = args.output / 'modpack-id.txt'
    idfile.write_text(identity['runtime_modpack_id'])
    results = []
    for case, source in [
        ('town-refusal', args.saves / 'fisher/initial-local-town.crystalsave'),
        ('registered-fishing', args.saves / 'water/actual-fishing-bank.crystalsave'),
    ]:
        directory = args.output / case
        directory.mkdir()
        saved = directory / 'actual-autosave.crystalsave'
        inputs = {str(p): digest(p) for p in [args.pack, args.client, args.inspector, source]}
        records, inspected, error, process = [], [], None, None

        def inspect(path):
            command = [str(args.inspector.resolve()), str(idfile), digest(args.pack),
                       identity['content_hash'], str(path)]
            result = subprocess.run(command, capture_output=True, text=True)
            assert result.returncode == 0, result.stderr
            inspected.append(result.stdout)
            return result.stdout

        try:
            with (directory / 'stderr.log').open('x') as stderr:
                process = subprocess.Popen(
                    [str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                     '--load', str(source.resolve()), '--save', str(saved.resolve())],
                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                    text=True, bufsize=1)
                client = LiveRecorded(process, records)
                client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                           clientInfo=dict(name='grand-rapids-riverfront', version='1')))
                if case == 'town-refusal':
                    client.at(TOWN, 6, 24)
                    before = client.status()
                    for point in [(9, 12), (23, 12), (27, 14), (39, 16),
                                  (38, 23), (16, 25), (20, 31), (31, 33), (26, 36)]:
                        walk(client, pack, point)
                    client.press('left')
                    client.press('a')
                    choose(client, False)
                    finish(client)
                    client.at(TOWN, 26, 36)
                    assert client.status()['money'] == before['money']
                    shutil.copy2(saved, directory / 'actual-declined.crystalsave')
                    walk(client, pack, (26, 38))
                    walk(client, pack, (6, 24))
                    client.press('start')
                    assert any('PACK' in line['text'] for line in
                               client.observe()['structuredContent']['menu'])
                    client.press('b')
                    assert 'MODE=Normal' in inspect(saved)
                else:
                    client.at(TOWN, 26, 38)
                    before = client.status()
                    assert before['facing'] == 'down'
                    bite = False
                    for _ in range(64):
                        client.press('select')
                        client.call('hold_button', button='left', frames=120)
                        notice = client.observe()['content'][0]['text']
                        bite = 'A bite!' in notice
                        assert bite or 'Not even a nibble!' in notice, notice
                        after = client.status()
                        for key in ['map', 'position', 'facing', 'money', 'badges', 'pokedex']:
                            assert after[key] == before[key], (key, before, after)
                        client.press('a')
                        client.at(TOWN, 26, 38)
                        if bite:
                            assert client.status()['mode'] == 'wildbattle'
                            native = inspect(saved)
                            slots = pack['data']['fishing']['groups']['FISHGROUP_MN_SHORE']['rod_tables']['OLD_ROD']['slots']
                            assert any('WILD=' + slot['species'] + ' ' in native for slot in slots)
                            shutil.copy2(saved, directory / 'actual-native-fish.crystalsave')
                            escape_wild(client)
                            break
                        assert client.status()['mode'] == 'overworld'
                    assert bite, 'native registered Old Rod never produced a bite'
                    client.at(TOWN, 26, 38)
                    assert 'MODE=Normal' in inspect(saved)
                process.stdin.close()
                assert process.wait(timeout=30) == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if process is not None and process.poll() is None:
                process.terminate()
                process.wait(timeout=30)
        stderr_text = (directory / 'stderr.log').read_text()
        unchanged = all(digest(Path(p)) == sha for p, sha in inputs.items())
        receipt = dict(status='PASS' if error is None and unchanged and not stderr_text else 'FAIL',
                       error=error, inputs=inputs, inputs_unchanged=unchanged,
                       stderr=stderr_text, native_save_inspection=inspected, calls=records,
                       scope=__doc__)
        (directory / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
        results.append(dict(case=case, status=receipt['status'], rpc_calls=len(records)))
        (args.output / 'receipt.json').write_text(json.dumps(results, indent=2) + '\n')
        assert receipt['status'] == 'PASS', error
        print(case, 'PASS', len(records), 'RPC calls', flush=True)


if __name__ == '__main__':
    main()
