#!/usr/bin/env python3
"""Continue the held garden offer and replay native paths/gift persistence."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

from check_mina_battle_mcp import LiveRecorded
from check_north_shore_story_mcp import finish
from check_prairie_story_mcp import road
from check_winona_mcp import travel, walk
from render_reference import load_pack

MAP = 'MnR02C00R00'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def pack_berry(client):
    client.press('start')
    for _ in range(32):
        menu = client.observe()['structuredContent']['menu']
        if any(row['text'].strip() == '> PACK' for row in menu):
            client.press('a')
            break
        client.press('down')
    else:
        raise AssertionError('production Start menu did not select PACK')
    menu = client.observe()['structuredContent']['menu']
    rows = [row['text'] for row in menu if 'BERRY' in row['text'].upper()]
    quantity = re.search(r'BERRY\s*[x×]\s*(\d+)\b', rows[0], re.I) if len(rows) == 1 else None
    assert quantity is not None and int(quantity[1]) == 1, menu
    for _ in range(8):
        client.press('b')
    client.at(MAP, 10, 19)


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'client', 'save', 'output']:
        cli.add_argument('--' + name, type=Path, required=True)
    cli.add_argument('--road-save', type=Path, help='Actual empty-party Care-ready road checkpoint')
    args = cli.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    inputs = {str(p): sha(p) for p in [args.pack, args.client, args.save]}
    if args.road_save:
        inputs[str(args.road_save)] = sha(args.road_save)
    pack = load_pack(args.pack)
    checks = []
    cases = ['held-gift-and-garden-paths', 'fresh-process-repeat']
    if args.road_save:
        cases += ['both-town-seams', 'fresh-process-road-continue']
    for case in cases:
        out = args.output / case
        out.mkdir()
        source = (args.save if case == 'held-gift-and-garden-paths' else args.road_save
                  if case == 'both-town-seams' else args.output /
                  ('both-town-seams/actual-autosave.crystalsave' if case == 'fresh-process-road-continue'
                   else 'held-gift-and-garden-paths/actual-autosave.crystalsave'))
        records, error, process = [], None, None
        try:
            with (out/'stderr.log').open('x') as stderr:
                process = subprocess.Popen([str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                                            '--load', str(source.resolve()), '--save', str((out/'actual-autosave.crystalsave').resolve())],
                                           stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                                           text=True, bufsize=1)
                c = LiveRecorded(process, records)
                c.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                         clientInfo=dict(name='r02-working-garden', version='1')))
                c.at(MAP, *( (58,14) if case == 'both-town-seams' else (10,19) ))
                before = c.status()
                garden_case = case in ['held-gift-and-garden-paths', 'fresh-process-repeat']
                assert before['badges'] == (0 if garden_case else 1)
                if case == 'held-gift-and-garden-paths':
                    assert before['dialogue']
                    c.move('right'); c.at(MAP, 10, 19)
                    pages = finish(c)
                    assert 'A BERRY for the' in pages and 'Take care!' in pages, pages
                    for x in [12, 13]:
                        for target in [(x, 12), (x, 19), (x, 12)]:
                            walk(c, pack, target)
                    for x in [10, 14]:
                        walk(c, pack, (x, 16)); c.move('down'); c.at(MAP, x, 16)
                    walk(c, pack, (10, 19))
                if garden_case:
                    pack_berry(c)
                    c.press('left'); c.press('a')
                    repeat = finish(c)
                    assert 'restores' in repeat and 'little HP' in repeat and 'Take care!' not in repeat, repeat
                    pack_berry(c)
                elif case == 'both-town-seams':
                    for y in [14,15]:
                        road(c,pack,(59,y)); travel(c,'right','MnRochester',(0,y+4))
                        travel(c,'left',MAP,(59,y))
                    for y in [30,31]:
                        road(c,pack,(0,y)); travel(c,'left','MnMankato',(47,y-12))
                        travel(c,'right',MAP,(0,y))
                    road(c,pack,(10,19))
                c.press('start'); assert c.observe()['structuredContent']['menu']
                c.press('b'); c.at(MAP,10,19)
                after = c.status()
                assert after['badges'] == before['badges']
                assert after['money'] == before['money']
                process.stdin.close(); process.wait(timeout=30)
                assert process.returncode == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if process is not None and process.poll() is None:
                process.terminate(); process.wait(timeout=30)
            (out/'transcript.json').write_text(json.dumps(records, indent=2)+'\n')
            stderr_empty = (out/'stderr.log').exists() and not (out/'stderr.log').read_bytes()
            check = dict(case=case, calls=len(records), error=error, stderr_empty=stderr_empty)
            checks.append(check)
            print(check, flush=True)
            (args.output/'receipt.json').write_text(json.dumps(dict(inputs=inputs, checks=checks), indent=2)+'\n')
        assert error is None and stderr_empty, check
    assert all(sha(Path(path)) == digest for path, digest in inputs.items())


if __name__ == '__main__':
    main()
