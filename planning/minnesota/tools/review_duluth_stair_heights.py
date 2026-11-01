#!/usr/bin/env python3
"""Find flat paths around authored Duluth stair flights, without publishing."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path

from render_reference import layout_blocks, load_pack


def assess(blocks, collision):
    def kind(p):
        x,y=p
        return collision[f'{blocks[y//2*64+x//2]:02x}'][y%2*2+x%2]
    flights=[]
    for i,b in enumerate(blocks):
        if b not in [0x8d,0x8e,0x8f,0x90]:continue
        x,y=i%64*2,i//64*2
        lane=x+int(b in [0x8d,0x90])
        if b==0x90:
            flights.append(dict(metatile=[x//2,y//2],upper=[lane,y+1],step=[lane,y],lower=[lane,y-1],orientation='north'))
            continue
        flights.append(dict(metatile=[x//2,y//2],upper=[lane,y],step=[lane,y+1],lower=[lane,y+2]))
    assert len(flights)==11
    sealed={tuple(f['step']) for f in flights}
    def path(start,end):
        previous={start:None};todo=deque([start])
        while todo:
            p=todo.popleft()
            if p==end:
                result=[]
                while p is not None:result.append(p);p=previous[p]
                return result[::-1]
            x,y=p
            for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if 0<=q[0]<128 and 0<=q[1]<92 and q not in previous and q not in sealed and kind(q)=='FLOOR':previous[q]=p;todo.append(q)
        return None
    for f in flights:
        assert all(kind(tuple(f[key]))=='FLOOR' for key in ['upper','step','lower']),f
        f['flat_bypass_without_any_stair']=path(tuple(f['upper']),tuple(f['lower']))
        x,y=f['step'];f['step_side_collision']=[kind((x-1,y)),kind((x+1,y))]
    labels={};sizes={}
    for yy in range(92):
        for xx in range(128):
            start=(xx,yy)
            if start in labels or start in sealed or kind(start)!='FLOOR':continue
            label=len(sizes);labels[start]=label;todo=deque([start]);size=0
            while todo:
                x,y=todo.popleft();size+=1
                for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                    if 0<=q[0]<128 and 0<=q[1]<92 and q not in labels and q not in sealed and kind(q)=='FLOOR':labels[q]=label;todo.append(q)
            sizes[label]=size
    edges=[]
    for f in flights:
        f['upper_component']=labels[tuple(f['upper'])];f['lower_component']=labels[tuple(f['lower'])]
        edges.append(dict(kind='stair',location=f['metatile'],upper=f['upper_component'],lower=f['lower_component']))
    graph={}
    for edge in edges:
        graph.setdefault(edge['upper'],[]).append((edge['lower'],-1,edge))
        graph.setdefault(edge['lower'],[]).append((edge['upper'],1,edge))
    heights={};conflicts=[]
    for start in [labels[(64,37)],*graph]:
        if start in heights:continue
        heights[start]=0;todo=deque([start])
        while todo:
            node=todo.popleft()
            for neighbor,delta,edge in graph.get(node,[]):
                expected=heights[node]+delta
                if neighbor not in heights:heights[neighbor]=expected;todo.append(neighbor)
                elif heights[neighbor]!=expected:conflicts.append(dict(edge=edge,from_component=node,to_component=neighbor,expected=expected,actual=heights[neighbor]))
    return dict(status='PASS' if not conflicts and all(f['flat_bypass_without_any_stair'] is None for f in flights) else 'DEFECTS',stairs=flights,relative_component_heights=heights,height_conflicts=conflicts,component_sizes=sizes,scope='Static FLOOR graph with every native stair step sealed; actor bodies do not mask a bad height boundary. Stair and ledge edges must admit consistent relative heights. Native input remains separate.')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for key in ['pack','layout','output']:parser.add_argument('--'+key,type=Path,required=True)
    args=parser.parse_args();pack=load_pack(args.pack);layout=json.loads(args.layout.read_text());collision=dict(pack['data']['tilesets']['mn_duluth_expanded']['collision'])
    for recipe in layout['derived_blocks'][13:]:collision[recipe['id']]=recipe['collision']
    result=assess(layout_blocks(layout),collision);result['layout_sha256']=hashlib.sha256(args.layout.read_bytes()).hexdigest();assert not args.output.exists();args.output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(dict(status=result['status'],height_conflicts=result['height_conflicts'],bypasses=[dict(metatile=f['metatile'],steps=len(f['flat_bypass_without_any_stair'])-1) for f in result['stairs'] if f['flat_bypass_without_any_stair'] is not None])))


if __name__=='__main__':main()
