#!/usr/bin/env python3
"""Replay Ada's card refusal/retry over actual production stdio MCP."""
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
            c=Client(p);c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='headquarters-check',version='1')))
            c.at('MnIceRecords',24,5)
            for yes in [False,True]:
                c.press('right');c.press('a');choose(c,yes);pages=finish(c)
                if yes:assert 'CARD KEY' in pages,pages
            c.press('right');c.press('a');assert 'staff doors' in finish(c)
            c.press('start');assert 'PACK' in c.observe()['content'][0]['text']
        finally:
            p.stdin.close();assert p.wait(timeout=30)==0
        stderr.seek(0);assert stderr.read()==b'','gameplay stderr must stay byte-empty'
    print('PASS: actual Ada refusal/retry, card grant, repeat dialogue, Start, clean exit and byte-empty stderr.')


if __name__=='__main__':main()
