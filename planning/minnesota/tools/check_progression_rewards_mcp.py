#!/usr/bin/env python3
"""Verify earned reward delivery over production stdio from declared chapter saves."""
import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_north_shore_story_mcp import finish


def screen(client):
    return client.observe()['content'][0]['text']


def open_pack(client):
    client.press('start')
    for _ in range(16):
        if '> PACK' in screen(client):
            client.press('a')
            return
        client.press('down')
    raise AssertionError('production PACK was not selectable')


def play(client, scene):
    client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
        clientInfo=dict(name='progression-rewards-check', version='1')))
    before = client.status()
    if scene == 'master':
        client.at('MnWinonaFieldStation', 5, 3)
        client.press('up'); client.press('a')
        assert 'MASTER BALL' in finish(client)
        client.press('up'); client.press('a')
        assert 'received' not in finish(client).lower()
        open_pack(client); client.press('right')
        assert '> MASTER BALL x01' in screen(client), screen(client)
    else:
        client.at('MnWinonaFieldStation', 2, 8)
        client.press('down'); client.press('a')
        first = finish(client)
        if scene == 'dex-denied':
            assert 'received' not in first.lower(), first
            assert 'come back' in first.lower(), first
        else:
            assert '10 Minnesota species' in first, first
            client.press('down'); client.press('a')
            assert '25 Minnesota species' in finish(client)
            client.press('down'); client.press('a')
            assert 'received' not in finish(client).lower()
            open_pack(client); client.press('right'); client.press('right')
            assert '> ITEMFINDER x01' in screen(client), screen(client)
    after = client.status()
    for key in ['money', 'badges', 'pokedex']:
        assert after[key] == before[key], (key, before, after)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['client', 'pack', 'save']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--scene', choices=['master', 'dex', 'dex-denied'], required=True)
    args = parser.parse_args()
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen([str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
            '--load', str(args.save.resolve())], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=stderr, text=True, bufsize=1)
        try:
            play(Client(process, timeout=90), args.scene)
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b'', 'gameplay stderr must remain byte-empty'
    print('PASS: production reward/repeat checks and byte-empty stderr:', args.scene)


if __name__ == '__main__':
    main()
