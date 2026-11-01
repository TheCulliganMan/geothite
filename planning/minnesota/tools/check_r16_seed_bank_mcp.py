#!/usr/bin/env python3
"""Replay actual seed-bank saves through the production stdio controller."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import subprocess

from check_mina_battle_mcp import LiveRecorded
from check_north_shore_story_mcp import finish
from render_reference import load_pack
from review_r16_seed_bank import graph

MAP = 'MnR16C00R00'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def walk(client, terrain, edges, target):
    status = client.status()
    start = status['position']['x'], status['position']['y']
    pending, previous = deque([start]), {start: None}
    while pending:
        at = pending.popleft()
        if at == target:
            break
        for q in edges.get(at, []):
            if q not in previous and terrain[q] in {'FLOOR', 'TALL_GRASS'}:
                previous[q] = at
                pending.append(q)
    assert target in previous, (start, target)
    route, q = [], target
    while q != start:
        p = previous[q]
        direction = {(1, 0): 'right', (-1, 0): 'left', (0, 1): 'down', (0, -1): 'up'}[q[0]-p[0], q[1]-p[1]]
        route.append((q, direction))
        q = p
    grass = 0
    for q, direction in reversed(route):
        client.move(direction)
        client.at(MAP, *q)
        grass += terrain[q] == 'TALL_GRASS'
    return grass


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'client', 'save', 'output']:
        cli.add_argument('--' + name, type=Path, required=True)
    args = cli.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    inputs = {str(p): sha(p) for p in [args.pack, args.client, args.save]}
    pack = load_pack(args.pack)
    terrain, edges = graph(pack, [pack['data']['maps'][MAP]], hops=False)
    results = []
    for case in ['two-hops-and-grassy-returns', 'fresh-process-continue']:
        out = args.output / case
        out.mkdir()
        source = args.save if case == 'two-hops-and-grassy-returns' else args.output / 'two-hops-and-grassy-returns/actual-autosave.crystalsave'
        records, process, error = [], None, None
        try:
            with (out / 'stderr.log').open('x') as stderr:
                process = subprocess.Popen([str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                                            '--load', str(source.resolve()), '--save', str((out / 'actual-autosave.crystalsave').resolve())],
                                           stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                                           text=True, bufsize=1)
                c = LiveRecorded(process, records)
                c.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                         clientInfo=dict(name='r16-seed-bank', version='1')))
                c.at(MAP, 47, 30)
                before = c.status()
                assert before['badges'] == 0
                if case == 'two-hops-and-grassy-returns':
                    for x in [42, 43]:
                        walk(c, terrain, edges, (x, 25))
                        c.move('down'); c.at(MAP, x, 26)
                        c.move('down'); c.at(MAP, x, 28)
                        c.move('up'); c.at(MAP, x, 28)
                        assert walk(c, terrain, edges, (x, 25)) >= 4
                    for x in [41, 44]:
                        walk(c, terrain, edges, (x, 28))
                        c.move('up'); c.at(MAP, x, 28)
                    walk(c, terrain, edges, (47, 30))
                    c.press('up'); c.press('a')
                    assert c.status()['dialogue']
                    c.move('down'); c.at(MAP, 47, 30)
                    finish(c)
                c.press('start')
                assert c.observe()['structuredContent']['menu']
                c.press('b'); c.at(MAP, 47, 30)
                after = c.status()
                assert after['badges'] == before['badges'] == 0
                assert after['money'] == before['money']
                process.stdin.close(); process.wait(timeout=30)
                assert process.returncode == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if process is not None and process.poll() is None:
                process.terminate(); process.wait(timeout=30)
            (out / 'transcript.json').write_text(json.dumps(records, indent=2) + '\n')
            stderr_empty = (out / 'stderr.log').exists() and not (out / 'stderr.log').read_bytes()
            result = dict(case=case, calls=len(records), error=error, stderr_empty=stderr_empty)
            results.append(result)
            print(result, flush=True)
            (args.output / 'receipt.json').write_text(json.dumps(dict(inputs=inputs, checks=results), indent=2) + '\n')
        assert error is None and stderr_empty, result
    assert all(sha(Path(p)) == value for p, value in inputs.items())


if __name__ == '__main__':
    main()
