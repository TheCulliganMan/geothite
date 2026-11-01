#!/usr/bin/env python3
"""Replay the reached Cedar Shelter circuits and healing over production stdio.

Consumes real controller checkpoints. Native controller tests separately verify
HP/status/PP, item consumption, fishing battle tables and once-only persistence.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

from check_mina_battle_mcp import LiveRecorded
from check_north_shore_story_mcp import finish
from check_winona_mcp import walk
from render_reference import load_pack

ROOM = 'MnR13Cave1F'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'client', 'saves', 'inspector', 'output']:
        parser.add_argument('--'+name, type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    pack = load_pack(args.pack)
    idfile = args.output/'modpack-id.txt'
    idfile.write_text(pack['identity']['runtime_modpack_id'])
    results = []
    for case in ['injured', 'ether-used']:
        folder = args.output/case
        folder.mkdir()
        source, save = args.saves/(case+'.crystalsave'), folder/'actual-autosave.crystalsave'
        bindings = {str(p): digest(p) for p in [args.pack, args.client, args.inspector, source]}
        records, error, process, inspection = [], None, None, None
        try:
            with (folder/'stderr.log').open('x') as stderr:
                process = subprocess.Popen(
                    [str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                     '--load', str(source.resolve()), '--save', str(save.resolve())],
                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                    text=True, bufsize=1)
                c = LiveRecorded(process, records)
                c.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                      clientInfo=dict(name='cedar-shelter', version='1')))
                c.at(ROOM, *([7, 14] if case == 'injured' else [14, 11]))
                money = c.status()['money']
                for point in [(12, 12), (12, 9), (10, 8), (10, 5), (13, 6)]:
                    walk(c, pack, point)
                c.press('up')
                c.press('a')
                c.press('down')
                c.at(ROOM, 13, 6)
                pages = finish(c)
                assert 'Rest in the rock.' in pages and 'Ely is not far.' in pages, pages
                for point in [(10, 8), (5, 5), (3, 7), (8, 12), (14, 11), (12, 9), (7, 14)]:
                    walk(c, pack, point)
                c.move('down')
                c.at('MnR13C00R01', 14, 6)
                c.move('down')
                c.at('MnR13C00R01', 14, 7)
                c.move('up')
                c.at('MnR13C00R01', 14, 6)
                c.move('up')
                c.at(ROOM, 7, 15)
                c.move('up')
                c.at(ROOM, 7, 14)
                assert c.status()['money'] == money
                c.press('start')
                assert any('PACK' in row['text'] for row in c.observe()['structuredContent']['menu'])
                c.press('b')
                shutil.copy2(save, folder/'actual-circuit-complete.crystalsave')
                checked = subprocess.run(
                    [str(args.inspector.resolve()), str(idfile), digest(args.pack),
                     pack['identity']['content_hash'], str(save)], capture_output=True, text=True)
                assert checked.returncode == 0, checked.stderr
                assert 'PARTY_SURF_MOVES=0' in checked.stdout and 'MODE=Normal' in checked.stdout
                inspection = checked.stdout
                process.stdin.close()
                assert process.wait(timeout=30) == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if process is not None and process.poll() is None:
                process.terminate()
                process.wait(timeout=30)
        stderr = (folder/'stderr.log').read_text()
        unchanged = all(digest(Path(p)) == sha for p, sha in bindings.items())
        receipt = dict(status='PASS' if error is None and not stderr and unchanged else 'FAIL',
                       error=error, bindings=bindings, inputs_unchanged=unchanged,
                       native_save_inspection=inspection, stderr=stderr, calls=records, scope=__doc__)
        (folder/'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
        results.append(dict(case=case, status=receipt['status'], rpc_calls=len(records)))
        (args.output/'receipt.json').write_text(json.dumps(results, indent=2)+'\n')
        assert receipt['status'] == 'PASS', error
        print(case, 'PASS', len(records), 'RPC calls', flush=True)


if __name__ == '__main__':
    main()
