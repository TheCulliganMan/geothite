#!/usr/bin/env python3
"""Walk Duluth's authored stair/ledge courses through the production MCP client.

Reads native collision only to plan ordinary paths. All authoritative movement,
turns, hops, saves, input ownership and Continue belong to geothite's existing
VisibleShellController-backed stdio server. No state or coordinates are rewritten.
"""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

from check_minneapolis_mcp import Client
from render_reference import load_pack

CITY = 'MnDuluth'
COURSES = [
    ('observation-west',(14,13),(14,18),(14,13),(14,18)),
    ('observation-east',(37,15),(37,18),(37,15),(37,18)),
    ('civic-upper',(46,13),(46,18),(46,13),(46,18)),
    ('hillside-west',(71,11),(69,16),(71,11),(71,16)),
    ('hillside-front',(80,13),(80,16),(80,13),(80,16)),
    ('outdoor-supplies',(87,10),(88,12),(88,10),(88,12)),
    ('weather-west',(99,11),(97,16),(99,11),(99,16)),
    ('weather-front',(106,13),(106,16),(106,13),(106,16)),
    ('weather-east',(114,9),(114,12),(114,9),(114,12)),
    ('garden-north',(47,25),(47,27),(47,25),(47,27)),
    ('garden-corner',(56,39),(56,42),(56,39),(56,42)),
]
CLIFF_EDGES = [(24,14),(25,14),(84,14),(85,14)] + [(x,40) for x in range(40,50)]
DIRECTIONS = {(1,0):'right',(-1,0):'left',(0,1):'down',(0,-1):'up'}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class RecordedClient(Client):
    def __init__(self, process, records):
        super().__init__(process, timeout=90)
        self.records = records

    def rpc(self, method, params):
        record = dict(method=method, params=params, id=self.sequence+1)
        self.records.append(record)
        result = super().rpc(method, params)
        record['result'] = result
        return result


class Walk:
    def __init__(self, client, module, collision, money):
        self.client, self.module, self.collision, self.money = client, module, collision, money
        self.occupied = {(o['x'],o['y']) for o in module['objects']}
        self.tiles = 0

    def kind(self, point):
        x,y = point
        return self.collision[f"{self.module['blocks'][y//2*64+x//2]:02x}"][y%2*2+x%2]

    def at(self, point):
        status = self.client.status()
        assert status['map']==CITY and status['position']==dict(x=point[0],y=point[1]), status
        assert status['mode']=='overworld' and not status['dialogue'] and status['money']==self.money, status
        return status

    def current(self):
        s = self.client.status()
        assert s['map']==CITY and s['mode']=='overworld' and not s['dialogue'], s
        return s['position']['x'], s['position']['y']

    def direct(self, direction, steps, target):
        assert steps > 0
        self.client.move(direction, steps)
        self.at(target)
        self.tiles += steps

    def ordinary(self, target):
        start = self.current()
        previous = {start:None}
        todo = deque([start])
        while todo and target not in previous:
            x,y = todo.popleft()
            for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if 0<=q[0]<128 and 0<=q[1]<92 and q not in previous and q not in self.occupied and self.kind(q)=='FLOOR':
                    previous[q]=(x,y);todo.append(q)
        assert target in previous, ('no ordinary body-aware source path',start,target)
        path=[];point=target
        while point!=start:path.append(point);point=previous[point]
        path.reverse();origin=start;i=0
        while i<len(path):
            delta=(path[i][0]-origin[0],path[i][1]-origin[1]);direction=DIRECTIONS[delta];end=i+1
            while end<len(path) and end-i<6 and (path[end][0]-path[end-1][0],path[end][1]-path[end-1][1])==delta:end+=1
            self.direct(direction,end-i,path[end-1]);origin=path[end-1];i=end
        self.at(target)
        return len(path)

    def denied(self, direction, point):
        self.client.call('press',button=direction,times=3)
        self.at(point)


def phase(args, name, save, play, records, phases):
    autosave = args.output/(name+'-actual-autosave.crystalsave')
    stderr_path = args.output/(name+'-stderr.log')
    assert not autosave.exists()
    process=None;error=None
    try:
        with stderr_path.open('x') as stderr:
            command=[str(args.client),'mcp',str(args.pack),'--load',str(save),'--save',str(autosave)]
            process=subprocess.Popen(command,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,text=True,bufsize=1)
            client=RecordedClient(process,records)
            client.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='duluth-stair-review',version='1')))
            play(client,autosave)
            process.stdin.close();process.wait(timeout=30)
            assert process.returncode==0,process.returncode
    except BaseException as exc:
        error=repr(exc);raise
    finally:
        if process is not None and process.poll() is None:process.terminate();process.wait(timeout=30)
        phases.append(dict(name=name,input_save=str(save),autosave=str(autosave),exit_code=None if process is None else process.returncode,stderr=stderr_path.read_text(),error=error))
    assert not stderr_path.read_bytes()
    return autosave


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for key in ['pack','save','client','output']:parser.add_argument('--'+key,type=Path,required=True)
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=False)
    initial_hashes={str(p):digest(p) for p in [args.pack,args.save,args.client]}
    pack=load_pack(args.pack);module=pack['data']['maps'][CITY];assert [module['attributes']['width'],module['attributes']['height']]==[64,46]
    collision=pack['data']['tilesets'][module['attributes']['tileset_name']]['collision']
    assert [module['blocks'].count(b) for b in [0x8d,0x8e,0x8f,0x90]]==[3,6,1,1]
    records=[];phases=[];checks=[];error=None;money=None
    def stairs(client,autosave):
        nonlocal money
        status=client.status();money=status['money'];walk=Walk(client,module,collision,money);walk.at((64,37))
        for name,upper,lower,crest,foot in COURSES:
            walk.ordinary(upper)
            if upper!=crest:walk.direct('right',crest[0]-upper[0],crest)
            step=(crest[0],foot[1]-1);walk.direct('down',step[1]-crest[1],step)
            walk.denied('left',step)
            if name!='garden-north':walk.denied('right',step)
            walk.direct('down',1,foot)
            if foot!=lower:walk.direct('left',foot[0]-lower[0],lower)
            snapshot=args.output/(name+'-actually-reached-lower.crystalsave');shutil.copyfile(autosave,snapshot)
            if foot!=lower:walk.direct('right',foot[0]-lower[0],foot)
            walk.direct('up',foot[1]-crest[1],crest)
            if crest!=upper:walk.direct('left',crest[0]-upper[0],upper)
            checks.append(dict(kind='bidirectional-stair',name=name,upper=upper,lower=lower,side_denials=1 if name=='garden-north' else 2,lower_save_sha256=digest(snapshot)))
            print('native stair PASS',name,flush=True)
        walk.ordinary((64,37));checks.append(dict(kind='stair-journey',ordinary_and_stair_tiles=walk.tiles))
    def ledges(client,autosave):
        walk=Walk(client,module,collision,money);walk.at((64,37))
        for x,y in CLIFF_EDGES:
            upper=(x,y);lower=(x,y+2);walk.ordinary(upper);walk.denied('down',upper)
            walk.ordinary(lower);walk.denied('up',lower)
            checks.append(dict(kind='high-cliff-down-and-up-denial',upper=upper,lower=lower))
            print('native high cliff PASS',x,y,flush=True)
        walk.ordinary((64,37));client.press('start');assert 'SAVE' in json.dumps(client.observe()).upper();client.press('b');walk.at((64,37));checks.append(dict(kind='ledge-journey',ordinary_and_hop_tool_steps=walk.tiles))
    try:
        reached=phase(args,'all-eleven-stairs',args.save,stairs,records,phases)
        phase(args,'continued-closed-cliff-edges',reached,ledges,records,phases)
    except BaseException as exc:error=repr(exc)
    unchanged=all(digest(Path(p))==h for p,h in initial_hashes.items())
    receipt=dict(status='PASS' if error is None and unchanged else 'FAIL',error=error,inputs=initial_hashes,inputs_unchanged=unchanged,scope='Continuous actual production MCP journey from a previously reached save. Initial historical setup was an explicit local fixture; no earned regional progression claim. Collision is used only for path planning; no post-input state repair.',checks=checks,phases=phases,calls=records)
    (args.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    assert error is None and unchanged,error
    assert len([c for c in checks if c['kind']=='bidirectional-stair'])==11
    assert len([c for c in checks if c['kind']=='high-cliff-down-and-up-denial'])==len(CLIFF_EDGES)
    print('PASS:',len(records),'actual MCP calls, eleven stairs and',len(CLIFF_EDGES),'closed high-cliff lanes; Continue and empty stderr',flush=True)


if __name__=='__main__':main()
