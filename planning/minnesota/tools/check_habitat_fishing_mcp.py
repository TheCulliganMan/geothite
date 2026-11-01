#!/usr/bin/env python3
"""Advance an actual registered-rod cast through production stdio input."""
import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client


def play(client, require_bite):
    client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
        clientInfo=dict(name='habitat-fishing-check', version='1')))
    client.at('MnR03C00R00', 11, 6)
    before = client.status()
    assert before['facing'] == 'right', before
    for _ in range(64):
        client.press('select')
        cast = client.status()
        for key in ['position', 'map', 'facing', 'money', 'badges']:
            assert cast[key] == before[key], (key, before, cast)
        client.call('hold_button', button='left', frames=120)
        notice = client.observe()['content'][0]['text']
        bite = 'A bite!' in notice
        assert bite or 'Not even a nibble!' in notice, notice
        after = client.status()
        for key in ['position', 'map', 'facing', 'money', 'badges', 'pokedex']:
            assert after[key] == cast[key], (key, cast, after)
        # Acknowledgement is a separate input boundary, after the timed cast.
        client.press('a')
        client.at('MnR03C00R00', 11, 6)
        if bite:
            assert client.status()['mode'] == 'wildbattle'
        if bite or not require_bite:
            return 'bite' if bite else 'no bite'
        assert client.status()['mode'] == 'overworld'
    raise AssertionError('native registered fishing never produced a bite')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['client', 'pack', 'save']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--require-bite', action='store_true')
    args = parser.parse_args()
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen([str(args.client.resolve()), 'mcp',
            str(args.pack.resolve()), '--load', str(args.save.resolve())],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
            text=True, bufsize=1)
        try:
            result = play(Client(process, timeout=90), args.require_bite)
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b'', 'gameplay stderr must remain byte-empty'
    print('PASS: registered-rod casting clock, blocked movement, visible notice, explicit acknowledgement and byte-empty stderr:', result)


if __name__ == '__main__':
    main()
