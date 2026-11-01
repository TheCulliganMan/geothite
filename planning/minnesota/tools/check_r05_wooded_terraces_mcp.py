#!/usr/bin/env python3
"""Walk the played River Bluffs terrace save through production stdio input."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import subprocess

from check_mina_battle_mcp import LiveRecorded
from render_reference import load_pack
from review_r05_wooded_terraces import graph


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def at(client, point):
    x,y=point
    client.at(f'MnR05C00R{y//48:02}',x,y%48)


def walk(client,edges,target):
    status=client.status();start=status['position']['x'],status['position']['y']+int(status['map'][-2:])*48
    pending,previous=deque([start]),{start:None}
    while pending:
        point=pending.popleft()
        if point==target:break
        for q in edges.get(point,[]):
            if q not in previous:previous[q]=point;pending.append(q)
    assert target in previous,(start,target)
    route=[];q=target
    while q!=start:
        p=previous[q];route.append((q,{(1,0):'right',(-1,0):'left',(0,1):'down',(0,-1):'up'}[q[0]-p[0],q[1]-p[1]]));q=p
    for q,key in reversed(route):client.move(key);at(client,q)


def main():
    cli=argparse.ArgumentParser(description=__doc__)
    for name in ['pack','client','save','output']:cli.add_argument('--'+name,type=Path,required=True)
    args=cli.parse_args();args.output.mkdir(parents=True,exist_ok=False)
    inputs={str(p):sha(p) for p in [args.pack,args.client,args.save]};pack=load_pack(args.pack)
    _,edges=graph(pack,[pack['data']['maps'][f'MnR05C00R{row:02}'] for row in range(3)])
    results=[]
    for case in ['paths-stairs-cliff-faces','fresh-process-continue']:
        out=args.output/case;out.mkdir();records=[];process=None;error=None
        source=args.save if case=='paths-stairs-cliff-faces' else args.output/'paths-stairs-cliff-faces/actual-autosave.crystalsave'
        try:
            with (out/'stderr.log').open('x') as stderr:
                process=subprocess.Popen([str(args.client.resolve()),'mcp',str(args.pack.resolve()),'--load',str(source.resolve()),'--save',str((out/'actual-autosave.crystalsave').resolve())],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,text=True,bufsize=1)
                c=LiveRecorded(process,records);c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='r05-wooded-terraces',version='1')))
                at(c,(31,28));before=c.status();assert before['badges']==0
                if case=='paths-stairs-cliff-faces':
                    for x,top,bottom in [(25,13,16),(37,25,28),(15,39,42),(29,51,54),(13,75,78)]:
                        walk(c,edges,(x,top))
                        for y in range(top+1,bottom+1):c.move('down');at(c,(x,y))
                        for y in range(bottom-1,top-1,-1):c.move('up');at(c,(x,y))
                    for point,key in [((25,34),'down'),((23,68),'down'),((18,71),'right'),((33,28),'up'),((40,32),'right')]:
                        walk(c,edges,point);c.move(key);at(c,point)
                    walk(c,edges,(31,28))
                c.press('start');assert c.observe()['structuredContent']['menu'];c.press('b');at(c,(31,28))
                after=c.status();assert after['badges']==before['badges']==0 and after['money']==before['money']
                process.stdin.close();process.wait(timeout=30);assert process.returncode==0
        except BaseException as exc:error=repr(exc)
        finally:
            if process is not None and process.poll() is None:process.terminate();process.wait(timeout=30)
            (out/'transcript.json').write_text(json.dumps(records,indent=2)+'\n');stderr_empty=(out/'stderr.log').exists() and not (out/'stderr.log').read_bytes()
            result=dict(case=case,calls=len(records),error=error,stderr_empty=stderr_empty);results.append(result);print(result,flush=True)
            (args.output/'receipt.json').write_text(json.dumps(dict(inputs=inputs,checks=results),indent=2)+'\n')
        assert error is None and stderr_empty,result
    assert all(sha(Path(p))==value for p,value in inputs.items())


if __name__=='__main__':main()
