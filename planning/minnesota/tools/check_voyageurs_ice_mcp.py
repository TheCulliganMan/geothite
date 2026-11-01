#!/usr/bin/env python3
"""Replay a controller-produced local ice fixture through production stdio MCP."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client


def open_client(binary, pack, load, save, stderr):
    env = dict(os.environ)
    env.pop('CRYSTAL_DIALOGUE_TRACE', None)
    process = subprocess.Popen([str(binary), 'mcp', str(pack), '--load', str(load),
                                '--save', str(save)], stdin=subprocess.PIPE,
                               stdout=subprocess.PIPE, stderr=stderr, text=True,
                               bufsize=1, env=env)
    client = Client(process)
    client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                 clientInfo=dict(name='voyageurs-ice-check', version='1')))
    return process, client


def until_map(client, direction, destination):
    for _ in range(8):
        if client.status()['map'] == destination:
            return
        client.press(direction)
    raise AssertionError(client.status())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--client', type=Path, required=True)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--load', type=Path, required=True)
    parser.add_argument('--save', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    report = {'fixture': 'Actual controller-exported local empty-party review; not earned regional access.'}
    with tempfile.TemporaryFile() as stderr:
        process, client = open_client(args.client.resolve(), args.pack.resolve(),
                                      args.load.resolve(), args.save.resolve(), stderr)
        try:
            client.at('MnR20C01R00', 6, 34)
            report['before'] = client.status()
            until_map(client, 'up', 'MnVoyageursIce1F')
            client.at('MnVoyageursIce1F', 5, 19)
            client.press('start')
            assert 'PACK' in client.observe()['content'][0]['text']
            client.press('b')
            client.move('up', 3)
            client.at('MnVoyageursIce1F', 5, 16)
            client.move('left')
            client.at('MnVoyageursIce1F', 4, 16)
            client.press('left')
            client.press('a')
            assert client.status()['dialogue']
            client.press('down')
            client.at('MnVoyageursIce1F', 4, 16)
            assert 'ice carries' in client.finish_dialogue().lower()
            client.move('right')
            client.move('down', 3)
            client.at('MnVoyageursIce1F', 5, 19)
            until_map(client, 'down', 'MnR20C01R00')
            client.at('MnR20C01R00', 6, 34)
            until_map(client, 'up', 'MnVoyageursIce1F')
            client.at('MnVoyageursIce1F', 5, 19)
            report['after'] = client.status()
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b'', 'production MCP stderr must remain empty'
    with tempfile.TemporaryFile() as stderr:
        process, client = open_client(args.client.resolve(), args.pack.resolve(),
                                      args.save.resolve(), args.save.resolve(), stderr)
        try:
            client.at('MnVoyageursIce1F', 5, 19)
            report['continued'] = client.status()
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b''
    report.update(status='PASS', production_start_menu=True, same_mouth_reentry=True,
                  continued_save=True, guide_dialogue=True, movement_owned_by_dialogue=True, stderr_bytes=0)
    args.report.write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
