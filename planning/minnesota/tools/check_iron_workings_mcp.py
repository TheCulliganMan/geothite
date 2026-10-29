#!/usr/bin/env python3
"""Replay haulage-pressure refusal/retry over actual production stdio MCP."""
import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_north_shore_story_mcp import choose, finish


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for arg in ['client','pack','save']:parser.add_argument('--'+arg,type=Path,required=True)
    args=parser.parse_args()
    with tempfile.TemporaryFile() as stderr:
        p=subprocess.Popen([str(args.client.resolve()),'mcp',str(args.pack.resolve()),'--load',str(args.save.resolve())],
            stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,text=True,bufsize=1)
        try:
            c=Client(p);c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='iron-workings-check',version='1')))
            c.at('MnIronHaulage',29,12)
            for yes in [False,True]:
                c.press('up');c.press('a');choose(c,yes);pages=finish(c)
                if yes:assert 'gauge falls' in pages,pages
            c.press('up');c.press('a');assert 'line is isolated' in finish(c)
            c.press('start');assert 'PACK' in c.observe()['content'][0]['text']
        finally:
            p.stdin.close();assert p.wait(timeout=30)==0
        stderr.seek(0);assert stderr.read()==b'','gameplay stderr must stay byte-empty'
    print('PASS: actual pressure refusal/retry, valve action, repeat dialogue, Start, clean exit and byte-empty stderr.')


if __name__=='__main__':main()
