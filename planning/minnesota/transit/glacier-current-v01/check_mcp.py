#!/usr/bin/env python3
"""Replay real reached passenger saves through the production stdio client."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from check_mina_battle_mcp import LiveRecorded
from check_north_shore_story_mcp import finish
from check_winona_mcp import walk
from render_reference import load_pack


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def ticket(client, pack, stop):
    walk(client, pack, tuple(stop['approach_core']))
    client.press('up')
    client.at(stop['map'], *stop['approach_core'])
    client.press('a')
    for _ in range(128):
        view = client.observe()['structuredContent']
        rows = [r['text'].strip() for r in view['menu']]
        if any(r.lstrip('>').strip() == 'CANCEL' for r in rows):
            return rows
        client.press('a')
    raise AssertionError('native ticket menu did not open')


def select(client, label):
    for _ in range(8):
        rows = client.observe()['structuredContent']['menu']
        selected = next((r['text'].strip().lstrip('>').strip() for r in rows
                         if r['text'].strip().startswith('>')), None)
        assert selected is not None, rows
        if selected == label:
            client.press('a')
            return
        client.press('down')
    raise AssertionError(('missing native ticket choice', label))


def ride(client, destination):
    for _ in range(128):
        status = client.status()
        if (status['map'] == destination['map'] and not status['dialogue']
                and status['mode'] == 'overworld'):
            client.at(destination['map'], *destination['ride_arrival_core'])
            return
        client.press('a')
    raise AssertionError(('train did not arrive', destination['map']))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for field in ['pack', 'client', 'save', 'output']:
        parser.add_argument('--'+field, type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    pack = load_pack(args.pack)
    source = json.loads(Path(__file__).with_name('service.json').read_text())
    stops = source['stations']
    cloud, paul = stops[2], stops[3]
    bindings = {str(p): sha(p) for p in [args.pack, args.client, args.save]}
    sessions, checks = [], []
    process, error = None, None
    save = args.output / 'actual-autosave.crystalsave'
    try:
        for phase in range(2):
            records = []
            sessions.append(records)
            initial = args.save if phase == 0 else args.output / 'actual-returned.crystalsave'
            with (args.output / f'stderr-{phase}.log').open('x') as stderr:
                process = subprocess.Popen(
                    [str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                     '--load', str(initial.resolve()), '--save', str(save.resolve())],
                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                    text=True, bufsize=1)
                client = LiveRecorded(process, records)
                client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                             clientInfo=dict(name='glacier-current', version='1')))
                client.at(cloud['map'], *cloud['ride_arrival_core'])
                initial_money = client.status()['money']
                if phase == 0:
                    rows = ticket(client, pack, cloud)
                    assert any('DETROIT LAKES X' in row for row in rows), rows
                    select(client, 'DETROIT LAKES X')
                    assert 'station' in finish(client).lower()
                    client.at(cloud['map'], *cloud['approach_core'])
                    checks.append('unknown-destination-marked-and-refused')
                    ticket(client, pack, cloud)
                    client.press('b')
                    finish(client)
                    checks.append('ticket-B-cancel-releases-input')
                    for origin, destination in [(cloud, paul), (paul, cloud)]:
                        ticket(client, pack, origin)
                        select(client, destination['menu_label'])
                        ride(client, destination)
                        assert client.status()['money'] == initial_money
                        checks.append(origin['id']+'-to-'+destination['id'])
                        shutil.copy2(save, args.output / ('actual-arrival-'+destination['id']+'.crystalsave'))
                    shutil.copy2(save, args.output / 'actual-returned.crystalsave')
                client.press('start')
                assert any('PACK' in row['text'] for row in client.observe()['structuredContent']['menu'])
                client.press('b')
                client.at(cloud['map'], *cloud['ride_arrival_core'])
                process.stdin.close()
                assert process.wait(timeout=30) == 0
        checks.append('fresh-process-Continue-and-production-Start')
    except BaseException as exc:
        error = repr(exc)
    finally:
        if process is not None and process.poll() is None:
            process.terminate()
            process.wait(timeout=30)
    stderr = ''.join(p.read_text() for p in args.output.glob('stderr-*.log'))
    unchanged = all(sha(p) == digest for p, digest in bindings.items())
    result = dict(status='PASS' if error is None and not stderr and unchanged else 'FAIL',
                  error=error, bindings=bindings, inputs_unchanged=unchanged,
                  stderr=stderr, checks=checks, sessions=sessions,
                  rpc_calls=sum(len(s) for s in sessions),
                  scope='Starts from an actual courier-win/train-arrival checkpoint. Upstream local party/badges/visit fixtures remain declared; this is not an earned campaign.')
    (args.output / 'receipt.json').write_text(json.dumps(result, indent=2)+'\n')
    assert result['status'] == 'PASS', error
    print('PASS', result['rpc_calls'], 'production RPC calls')


if __name__ == '__main__':
    main()
