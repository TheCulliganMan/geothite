#!/usr/bin/env python3
"""Replay the staged family/professor introductions through production stdio."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_winona_mcp import choose, walk
from render_reference import load_pack


def enter(client, direction, destination):
    for _ in range(16):
        if client.status()['map'] == destination:
            return
        client.press(direction)
    raise AssertionError(('missing arrival', destination, client.status()))


def read(client, pages):
    observed = []
    for expected in pages:
        lines = [line['text'] for line in client.observe()['structuredContent']['dialogue']]
        actual = '\n'.join(lines).strip()
        assert actual == '\n'.join(expected), (expected, actual)
        observed.append(actual)
        client.press('a')
    for _ in range(4):
        if not client.status()['dialogue']:
            return '\n\n'.join(observed)
        client.press('a')
    raise AssertionError('text did not release input')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--client', type=Path, required=True)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--story-source', type=Path,
                        help='Matching frozen canonical source for an older review pack')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    root = Path(__file__).resolve().parents[3]
    source = args.story_source or root / 'planning/minnesota/story.json'
    texts = json.loads(source.read_text())['dialogue_overhaul']['texts']
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen(
            [str(args.client.resolve()), 'mcp', str(args.pack.resolve())],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
            text=True, bufsize=1)
        try:
            client = Client(process, timeout=90)
            client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                clientInfo=dict(name='dialogue-overhaul-check', version='1')))
            client.at('MnWinonaHome2F', 3, 3)
            walk(client, pack, (7, 1))
            enter(client, 'up', 'MnWinonaHome1F')
            if client.status()['position']['y'] == 0:
                client.press('down')
            client.at('MnWinonaHome1F', 9, 1)
            client.move('down', 3)
            client.at('MnWinonaHome1F', 9, 1)
            mom = read(client, texts['MnWinonaHome1F']['MnWHomeIntroText'])
            walk(client, pack, (6, 6))
            enter(client, 'down', 'MnWinona')
            walk(client, pack, (18, 20))
            enter(client, 'up', 'MnWinonaFieldStation')
            client.at('MnWinonaFieldStation', 4, 2)
            client.move('right', 3)
            client.at('MnWinonaFieldStation', 4, 2)
            professor = read(client, texts['MnWinonaFieldStation']['MnOverhaulTamarackIntroText'])
            walk(client, pack, (7, 4))
            client.press('up')
            client.press('a')
            choose(client, False)
            read(client, texts['MnWinonaFieldStation']['MnWDeclineText'])
            client.press('start')
            assert 'PACK' in client.observe()['content'][0]['text']
            print(mom + '\n\n' + professor)
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0, 'client failed to exit'
        stderr.seek(0)
        assert stderr.read() == b'', 'gameplay stderr must remain byte-empty'
    print('PASS: New Game, staged Mom/lab, exact pages, input lock, starter decline, Start, clean stderr.')


if __name__ == '__main__':
    main()
