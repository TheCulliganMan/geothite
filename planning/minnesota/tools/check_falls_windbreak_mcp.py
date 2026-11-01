#!/usr/bin/env python3
"""Replay both winter-town loops and native Skier conversations over stdio.

Consumes actual controller checkpoints without editing saves or game state.
Battle PP, prize, flag isolation and services are separately controller-tested.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

from check_mina_battle_mcp import LiveRecorded
from check_north_shore_story_mcp import choose, finish
from check_winona_mcp import walk
from render_reference import load_pack

TOWN = 'MnInternationalFalls'


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
    for case, filename in [('town-refusal', 'initial-local-town'), ('won-continue', 'skier-won')]:
        out = args.output/case
        out.mkdir()
        source = args.saves/'skier'/(filename+'.crystalsave')
        save = out/'actual-autosave.crystalsave'
        inputs = {str(p): digest(p) for p in [args.pack, args.client, args.inspector, source]}
        records, error, process, inspected = [], None, None, []
        try:
            with (out/'stderr.log').open('x') as stderr:
                process = subprocess.Popen(
                    [str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                     '--load', str(source.resolve()), '--save', str(save.resolve())],
                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                    text=True, bufsize=1)
                c = LiveRecorded(process, records)
                c.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                      clientInfo=dict(name='falls-windbreak', version='1')))
                c.at(TOWN, *([20, 25] if case == 'town-refusal' else [17, 13]))
                before = c.status()
                for point in [(19, 22), (13, 20), (8, 17), (7, 12), (9, 8),
                              (14, 11), (23, 10), (27, 12), (27, 14),
                              (23, 17), (15, 14), (17, 13)]:
                    walk(c, pack, point)
                c.press('up')
                c.press('a')
                if case == 'town-refusal':
                    choose(c, False)
                finish(c)
                c.at(TOWN, 17, 13)
                assert c.status()['mode'] == 'overworld'
                assert c.status()['money'] == before['money']
                shutil.copy2(save, out/'actual-yard-conversation.crystalsave')
                walk(c, pack, (13, 20))
                for _ in range(3):
                    c.press('up')
                    c.at(TOWN, 13, 20)
                walk(c, pack, (20, 25))
                c.press('start')
                assert any('PACK' in line['text'] for line in c.observe()['structuredContent']['menu'])
                c.press('b')
                result = subprocess.run(
                    [str(args.inspector.resolve()), str(idfile), digest(args.pack),
                     pack['identity']['content_hash'], str(save)], capture_output=True, text=True)
                assert result.returncode == 0, result.stderr
                assert 'MODE=Normal' in result.stdout and 'MAP='+TOWN in result.stdout
                inspected.append(result.stdout)
                process.stdin.close()
                assert process.wait(timeout=30) == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if process is not None and process.poll() is None:
                process.terminate()
                process.wait(timeout=30)
        stderr = (out/'stderr.log').read_text()
        unchanged = all(digest(Path(p)) == sha for p, sha in inputs.items())
        receipt = dict(status='PASS' if error is None and unchanged and not stderr else 'FAIL',
                       error=error, inputs=inputs, inputs_unchanged=unchanged, stderr=stderr,
                       native_save_inspection=inspected, calls=records, scope=__doc__)
        (out/'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
        results.append(dict(case=case, status=receipt['status'], rpc_calls=len(records)))
        (args.output/'receipt.json').write_text(json.dumps(results, indent=2)+'\n')
        assert receipt['status'] == 'PASS', error
        print(case, 'PASS', len(records), 'RPC calls', flush=True)


if __name__ == '__main__':
    main()
