#!/usr/bin/env python3
"""Replay authored rival choices over the production stdio input path."""
import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_north_shore_story_mcp import choose, finish


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['client', 'pack', 'save']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--scene', choices=['rochester', 'intake', 'aftermath'], required=True)
    args = parser.parse_args()
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen(
            [str(args.client.resolve()), 'mcp', str(args.pack.resolve()), '--load', str(args.save.resolve())],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True, bufsize=1)
        try:
            client = Client(process, timeout=90)
            client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                clientInfo=dict(name='rival-arc-check', version='1')))
            if args.scene == 'rochester':
                client.at('MnRochesterPokecenter1F', 8, 10)
                for _ in range(2):
                    client.press('right'); client.press('a'); choose(client, False)
                    assert 'I can wait' in finish(client)
            elif args.scene == 'intake':
                client.at('MnBoundaryIntake', 17, 12)
                client.press('up'); client.press('a'); choose(client, False)
                finish(client)
                client.press('up'); client.press('a'); choose(client, True)
                pages = finish(client)
                for witnessed in ['gate lifts', 'returning water', 'last crew is clear', 'readings are public']:
                    assert witnessed in pages, (witnessed, pages)
                client.at('MnBoundaryIntake', 13, 15)
                client.move('up', 3)
                client.move('right', 4)
                client.press('up'); client.press('a')
                assert 'removing the winch' in finish(client)
            else:
                client.at('MnStCloudStation', 14, 16)
                client.press('right'); client.press('a')
                assert 'Do not ask me to thank you' in finish(client)
            client.press('start')
            assert 'PACK' in client.observe()['content'][0]['text']
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b'', 'gameplay stderr must remain byte-empty'
    print('PASS: production input, visible choice/dialogue, Start menu, clean exit and byte-empty stderr:', args.scene)


if __name__ == '__main__':
    main()
