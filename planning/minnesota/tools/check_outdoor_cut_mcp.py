#!/usr/bin/env python3
"""Exercise R06's native Cut permissions using exact-pack initial fixtures."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

from check_duluth_stairs_mcp import RecordedClient


def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()


def main():
    cli=argparse.ArgumentParser(description=__doc__)
    for key in ['pack','fixtures','client','output']:cli.add_argument('--'+key,type=Path,required=True)
    args=cli.parse_args();args.client=args.client.resolve();args.output.mkdir(exist_ok=False,parents=True)
    inputs={str(p):sha(p) for p in [args.pack,args.client]};results=[]
    for name,allowed in [('no-cut',False),('no-hive',False),('cut-and-hive',True)]:
        source=args.fixtures/(name+'.crystalsave');initial=sha(source);folder=args.output/name;folder.mkdir(exist_ok=False);records=[];process=None;error=None
        try:
            with (folder/'stderr.log').open('x') as stderr:
                process=subprocess.Popen([str(args.client),'mcp',str(args.pack),'--load',str(source),'--save',str(folder/'actual-autosave.crystalsave')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,text=True,bufsize=1)
                c=RecordedClient(process,records);c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='native-cut-gate-check',version='1')))
                def at(x,y,settled=True):
                    s=c.status();assert s['map']=='MnR06C00R00' and s['position']==dict(x=x,y=y),s
                    if settled:assert s['mode']=='overworld' and not s['dialogue'],s
                    assert s['money']==10000,s
                    return s
                at(29,30);c.move('down',1);at(29,31);c.call('press',button='down',times=3);at(29,31)
                c.press('a');page=c.observe();(folder/'first-interaction.json').write_text(json.dumps(page,indent=2)+'\n')
                c.call('press',button='a',times=8);c.call('press',button='b',times=2);at(29,31)
                if allowed:
                    c.move('down',2);at(29,33);c.move('up',3);at(29,30)
                else:
                    c.move('down',1);at(29,31)
                process.stdin.close();process.wait(timeout=30);assert process.returncode==0
        except BaseException as exc:error=repr(exc)
        finally:
            if process is not None and process.poll() is None:process.terminate();process.wait(timeout=30)
        stderr=(folder/'stderr.log').read_text()
        if stderr and error is None:error='native stderr was not empty'
        if sha(source)!=initial:error='initial fixture changed'
        r=dict(status='PASS' if error is None else 'FAIL',case=name,cut_permitted=allowed,error=error,initial_sha256=initial,stderr=stderr,calls=records,exit_code=None if process is None else process.returncode)
        (folder/'receipt.json').write_text(json.dumps(r,indent=2)+'\n');results.append(r);print(name,r['status'],error or '',flush=True)
    unchanged=all(sha(Path(p))==h for p,h in inputs.items());report=dict(status='PASS' if unchanged and all(r['status']=='PASS' for r in results) else 'FAIL',inputs=inputs,inputs_unchanged=unchanged,cases=[dict(name=r['case'],status=r['status'],error=r['error']) for r in results],scope='Explicit position/party/move/Hive fixtures before first input; subsequent actions solely through production MCP. No earned HM or campaign claim.')
    (args.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n');assert report['status']=='PASS','Inspect retained native Cut failures'


if __name__=='__main__':main()
