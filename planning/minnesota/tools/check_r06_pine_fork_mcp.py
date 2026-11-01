#!/usr/bin/env python3
"""Replay actual Pine Country fork and qualified road saves through stdio input."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import subprocess

from check_mina_battle_mcp import LiveRecorded
from render_reference import load_pack
from review_r05_wooded_terraces import graph


def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()


def at(client,point):
    x,y=point;client.at(f'MnR06C00R{y//48:02}',x,y%48)


def cross(client,key,name,target):
    for _ in range(12):
        status=client.status()
        if status['map']==name and status['position']==dict(x=target[0],y=target[1]):return
        client.move(key)
    client.at(name,*target)


def walk(client,terrain,edges,target):
    status=client.status();start=status['position']['x'],status['position']['y']+int(status['map'][-2:])*48
    previous={start:None};pending=deque([start])
    while pending:
        p=pending.popleft()
        if p==target:break
        for q in edges.get(p,[]):
            if q not in previous:previous[q]=p;pending.append(q)
    assert target in previous,(start,target)
    route=[];q=target
    while q!=start:
        p=previous[q];route.append((q,{(1,0):'right',(-1,0):'left',(0,1):'down',(0,-1):'up'}[q[0]-p[0],q[1]-p[1]]));q=p
    grass=0
    for q,key in reversed(route):client.move(key);at(client,q);grass+=terrain[q]=='TALL_GRASS'
    return grass


def main():
    cli=argparse.ArgumentParser(description=__doc__)
    for name in ['pack','client','save','output']:cli.add_argument('--'+name,type=Path,required=True)
    cli.add_argument('--road-save',type=Path);args=cli.parse_args();args.output.mkdir(parents=True,exist_ok=False)
    inputs={str(p):sha(p) for p in [args.pack,args.client,args.save,*([args.road_save] if args.road_save else [])]}
    pack=load_pack(args.pack);terrain,edges=graph(pack,[pack['data']['maps'][f'MnR06C00R{row:02}'] for row in range(2)],hops=False)
    cases=['new-drops-grassy-returns','fresh-process-continue']+(['both-town-seams'] if args.road_save else []);results=[]
    for case in cases:
        out=args.output/case;out.mkdir();records=[];process=None;error=None
        source=args.road_save if case=='both-town-seams' else (args.save if case=='new-drops-grassy-returns' else args.output/'new-drops-grassy-returns/actual-autosave.crystalsave')
        try:
            with (out/'stderr.log').open('x') as stderr:
                process=subprocess.Popen([str(args.client.resolve()),'mcp',str(args.pack.resolve()),'--load',str(source.resolve()),'--save',str((out/'actual-autosave.crystalsave').resolve())],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,text=True,bufsize=1)
                c=LiveRecorded(process,records);c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='r06-pine-fork',version='1')));at(c,(43,25));before=c.status()
                if case=='new-drops-grassy-returns':
                    assert before['badges']==0
                    for x in [42,43]:
                        walk(c,terrain,edges,(x,25));c.move('down');at(c,(x,26));c.move('down');at(c,(x,28));c.move('up');at(c,(x,28));assert walk(c,terrain,edges,(x,25))>=6
                    for x in [41,44]:
                        walk(c,terrain,edges,(x,25));c.move('down');at(c,(x,25))
                        walk(c,terrain,edges,(x,28));c.move('up');at(c,(x,28))
                    for x in [44,45]:
                        walk(c,terrain,edges,(x,9));c.move('down');at(c,(x,10));c.move('down');at(c,(x,12));c.move('up');at(c,(x,12));assert walk(c,terrain,edges,(x,9))>=8
                    walk(c,terrain,edges,(43,25))
                elif case=='both-town-seams':
                    assert before['badges']==8
                    for point,key,name,target in [((42,0),'up','MnBrainerd',(32,47)),((43,0),'up','MnBrainerd',(33,47)),((16,87),'down','MnStCloud',(24,0)),((17,87),'down','MnStCloud',(25,0))]:
                        walk(c,terrain,edges,point);cross(c,key,name,target);cross(c,'down' if key=='up' else 'up',f'MnR06C00R{point[1]//48:02}',(point[0],point[1]%48));at(c,point)
                    walk(c,terrain,edges,(43,25))
                c.press('start');assert c.observe()['structuredContent']['menu'];c.press('b');at(c,(43,25))
                after=c.status();assert after['badges']==before['badges'] and after['money']==before['money']
                process.stdin.close();process.wait(timeout=30);assert process.returncode==0
        except BaseException as exc:error=repr(exc)
        finally:
            if process is not None and process.poll() is None:process.terminate();process.wait(timeout=30)
            (out/'transcript.json').write_text(json.dumps(records,indent=2)+'\n');stderr_empty=(out/'stderr.log').exists() and not (out/'stderr.log').read_bytes()
            result=dict(case=case,calls=len(records),error=error,stderr_empty=stderr_empty);results.append(result);print(result,flush=True)
            (args.output/'receipt.json').write_text(json.dumps(dict(inputs=inputs,checks=results),indent=2)+'\n')
        assert error is None and stderr_empty,result
    assert all(sha(Path(p))==h for p,h in inputs.items())


if __name__=='__main__':main()
