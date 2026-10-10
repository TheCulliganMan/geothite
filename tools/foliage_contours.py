#!/usr/bin/env python3
"""Soften original crown lighting across material seams without moving foliage."""
import argparse,copy,json
from pathlib import Path
from sculpt_geometry import crease_normals


def prepare(model):
    model=copy.deepcopy(model);selected=[i for i,p in enumerate(model['primitives']) if p['material'].startswith('leaf')]
    combined={'positions':[],'normals':[],'indices':[]};ranges=[]
    for i in selected:
        p=model['primitives'][i];offset=len(combined['positions'])//3;start=len(combined['indices'])
        combined['positions'].extend(p['positions']);combined['normals'].extend(p['normals']);combined['indices'].extend(j+offset for j in p['indices']);ranges.append((i,start,len(combined['indices'])))
    shaded=crease_normals(combined,65.)
    for i,start,end in ranges:
        p=model['primitives'][i];ids=shaded['indices'][start:end];used=sorted(set(ids));remap={j:k for k,j in enumerate(used)}
        for key in ('positions','normals'):p[key]=[round(c,7) for j in used for c in shaded[key][j*3:j*3+3]]
        p['indices']=[remap[j] for j in ids]
    return model


def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('paths',type=Path,nargs='+');a=a.parse_args()
    for path in a.paths:
        m=prepare(json.loads(path.read_text()));path.write_text(json.dumps(m,separators=(',',':')))
        print(path.name,sum(len(p['indices'])//3 for p in m['primitives']),'unchanged triangles; smooth crown, original trunk and bounds')
if __name__=='__main__':main()
