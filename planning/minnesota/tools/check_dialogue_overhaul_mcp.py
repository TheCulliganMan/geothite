#!/usr/bin/env python3
"""Replay the staged family/professor introductions through production stdio."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_winona_mcp import walk
from render_reference import load_pack


def enter(client, direction, destination):
    for _ in range(16):
        if client.status()['map'] == destination:
            return
        client.press(direction)
    raise AssertionError(('missing arrival', destination, client.status()))


def read(client, pages, acknowledge_last=True):
    observed = []
    for index, expected in enumerate(pages):
        lines = [line['text'] for line in client.observe()['structuredContent']['dialogue']]
        actual = '\n'.join(lines).strip()
        assert actual == '\n'.join(expected), (expected, actual)
        observed.append(actual)
        if acknowledge_last or index + 1 < len(pages):
            client.press('a')
    return '\n\n'.join(observed)


def decline_starter(client, species, pages):
    snapshot = client.observe()['structuredContent']
    prompt = [line['text'].strip() for line in snapshot['prompt']]
    assert prompt == [f'Picture: {species}', 'A/B: close picture'], prompt
    assert not snapshot['dialogue'], 'the portrait must precede the introduction'
    assert client.status()['pokedex']['owned'] == 0, 'starter must not be granted at its portrait'
    client.press('a')
    # Pending YES/NO is exposed during earlier pages, but the printer still
    # owns input. Read every reviewed page before moving the choice cursor.
    introduction = read(client, pages, acknowledge_last=False)
    prompt = [line['text'].strip() for line in client.observe()['structuredContent']['prompt']]
    assert prompt == ['> YES', 'NO'], prompt
    client.press('down')
    snapshot = client.observe()['structuredContent']
    prompt = [line['text'].strip() for line in snapshot['prompt']]
    assert prompt == ['YES', '> NO'], ('choice must be interactive after the final page', prompt)
    assert [line['text'] for line in snapshot['dialogue']] == pages[-1]
    client.press('a')
    assert client.status()['pokedex']['owned'] == 0, 'declining must not grant a starter'
    return introduction


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
            assert client.status()['dialogue'], 'Mom must approach on the stair arrival'
            actors = client.call('map_info')['structuredContent']['visible_objects']
            assert any(actor['id'] == 'MN_W_GUARDIAN' and
                       (actor['offset_x'], actor['offset_y']) == (0, 1)
                       for actor in actors), actors
            client.move('down', 3)
            client.at('MnWinonaHome1F', 9, 1)
            mom = read(client, texts['MnWinonaHome1F']['MnWHomeIntroText'])
            gear = read(client, texts['MnWinonaHome1F']['MnWHomeGearReceivedText'])
            assert not client.status()['dialogue'], 'Mom must release input after the receipt'
            actors = client.call('map_info')['structuredContent']['visible_objects']
            assert any(actor['id'] == 'MN_W_GUARDIAN' and
                       (actor['offset_x'], actor['offset_y']) == (-2, 3)
                       for actor in actors), actors
            client.press('start')
            for _ in range(8):
                menu = client.observe()['structuredContent']['menu']
                selected = [line['text'] for line in menu if line['text'].lstrip().startswith('>')]
                if selected and 'GEAR' in selected[0].upper():
                    break
                client.press('down')
            else:
                raise AssertionError(('POKEGEAR missing from Start', menu))
            client.press('a')
            assert 'pokegear' in json.dumps(client.observe()['structuredContent']).lower()
            client.press('b')
            client.press('start')
            client.press('b')
            walk(client, pack, (6, 6))
            enter(client, 'down', 'MnWinona')
            walk(client, pack, (18, 20))
            enter(client, 'up', 'MnWinonaFieldStation')
            client.at('MnWinonaFieldStation', 4, 2)
            client.move('right', 3)
            client.at('MnWinonaFieldStation', 4, 2)
            professor = read(client, texts['MnWinonaFieldStation']['MnOverhaulTamarackIntroText'])
            assert not client.status()['dialogue'], 'lab introduction must finish'
            client.press('right')
            client.press('a')
            reminder = read(client, texts['MnWinonaFieldStation']['MnWChooseText'])
            assert not client.status()['dialogue'], 'short reminder must release input'
            walk(client, pack, (7, 4))
            client.press('up')
            client.press('a')
            starter = decline_starter(
                client, 'KINDLIT', texts['MnWinonaFieldStation']['MN_W_CYNDAQUILText'])
            refusal = read(client, texts['MnWinonaFieldStation']['MnWDeclineText'])
            assert not client.status()['dialogue'], 'declining must return to exploration'
            client.press('start')
            assert 'PACK' in client.observe()['content'][0]['text']
            print('\n\n'.join([mom, gear, professor, reminder, starter, refusal]))
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0, 'client failed to exit'
        stderr.seek(0)
        assert stderr.read() == b'', 'gameplay stderr must remain byte-empty'
    print('PASS: New Game, Mom handoff, Pokégear from Start, lab, starter portrait, exact introduction and decline, clean stderr.')


if __name__ == '__main__':
    main()
