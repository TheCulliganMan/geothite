#!/usr/bin/env python3
"""Focused metadata-only checks for native two-hop source/clamp separation."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).parent))
import review_winter_native as winter


def module(name,w,h,links):
    return dict(id=name,attributes=dict(width=w,height=h,connections=[dict(direction=d,target_map=t,offset=o) for d,t,o in links]))


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);args=p.parse_args();assert not args.output.exists()
    a=module('MnA',32,24,[('east','MnB',0)])
    b=module('MnB',32,24,[('south','MnC',0),('east','MnD',0),('west','MnA',0)])
    c=module('MnC',32,8,[('north','MnB',0),('south','MnTooFar',0)])
    d=module('MnD',32,24,[('west','MnB',0)])
    far=module('MnTooFar',32,8,[]);pack=dict(data=dict(maps={m['id']:m for m in [a,b,c,d,far]}))
    rows=winter.connected_sources(pack,'MnA');flat=[(m['id'],x,y)for m,x,y in rows]
    assert flat==[('MnA',0,0),('MnC',1024,768),('MnD',2048,0),('MnB',1024,0)]
    assert len(rows)==4 and 'MnTooFar'not in [m['id']for m,x,y in rows]
    direct=winter.original_sources(pack,'MnA');assert len(direct)==2
    actual=winter.camera_origin(rows,[63,45]);expected=winter.original_camera_origin(direct,[63,45]);incorrect=winter.original_camera_origin(rows,[63,45]);assert actual==expected and actual!=incorrect
    assert actual==((320,360),[116,78]);assert incorrect==((320,288),[116,81])
    # Metadata ownership at the omitted corner is real depth2 terrain, while
    # a point beyond every rectangle still falls back to the active border.
    def owner(x,y):
        for m,ox,oy in [rows[0],*reversed(rows[1:])]:
            if ox<=x<ox+m['attributes']['width']*32 and oy<=y<oy+m['attributes']['height']*32:return m['id']
        return rows[0][0]['id']+' border'
    assert owner(1032,776)=='MnC';assert owner(1000,776)=='MnA border'
    result=dict(status='PASS',checks=['Two-hop BFS placement/order exact with stable depth2-before-direct ordering.','Reciprocal links do not duplicate the active map.','Depth3 source excluded.','Depth2 terrain contributes to source ownership but not immediate-neighbor camera clamp.','Outside every real rectangle, active native border ownership is retained.'],sources=flat,correct_camera=actual,deliberately_wrong_all_sources_camera=incorrect,isolated_reviewer_sha256=hashlib.sha256(Path(winter.__file__).read_bytes()).hexdigest())
    args.output.parent.mkdir(parents=True,exist_ok=True)
    with args.output.open('x')as f:f.write(json.dumps(result,indent=2)+'\n')
    print('PASS5 metadata source-resolution checks')


if __name__=='__main__':main()
