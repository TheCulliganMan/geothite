#!/usr/bin/env python3
"""Inspect physical western component and detached compact proposals, read-only."""
from collections import deque
from copy import deepcopy
import hashlib
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from audit_outdoor_connections import neighbor_origin, OPPOSITE
from render_reference import load_pack
from review_superior_inland_pack import positions_from_connections

PACK=ROOT/'content-packs/minnesota-superior-inland-winter-v01.crystalpack'
SHA='65cf6f98e65ada839ec5afaa57728bf8289d3a50ba53ba6b9b50f08d3620bb17'
OUT=Path(__file__).with_name('western-component-v01.md')


def table(headers,rows):
    return ['| '+' | '.join(headers)+' |','| '+' | '.join('---' for _ in headers)+' |']+['| '+' | '.join(str(v) for v in r)+' |' for r in rows]


def bounds(names,maps,positions):
    return (min(positions[n][0] for n in names),min(positions[n][1] for n in names),max(positions[n][0]+maps[n]['attributes']['width'] for n in names),max(positions[n][1]+maps[n]['attributes']['height'] for n in names))


def overlaps(maps,positions):
    names=sorted(positions); result=[]
    for i,n in enumerate(names):
        x,y=positions[n];a=maps[n]['attributes']
        for t in names[i+1:]:
            u,v=positions[t];b=maps[t]['attributes']
            left,top=max(x,u),max(y,v);right,bottom=min(x+a['width'],u+b['width']),min(y+a['height'],v+b['height'])
            if left<right and top<bottom:result.append((n,t,(left,top,right,bottom)))
    return result


def solve(maps):
    pos={'MnMinneapolis':(0,0)};q=deque(pos);errors=[]
    while q:
        n=q.popleft();a=maps[n]['attributes']
        if len({e['direction'] for e in a['connections']})!=len(a['connections']):errors.append((n,'duplicate direction'))
        for e in a['connections']:
            t=e['target_map']
            if t not in maps:errors.append((n,t,'missing map'));continue
            b=maps[t]['attributes'];reverse=dict(direction=OPPOSITE[e['direction']],target_map=n,offset=-e['offset'])
            if reverse not in b['connections']:errors.append((n,t,'missing reciprocal'))
            point=neighbor_origin(pos[n],a,b,e)
            if t in pos:
                if pos[t]!=point:errors.append((n,t,'cycle',point,pos[t]))
            else:pos[t]=point;q.append(t)
    return pos,errors


def proposal(exterior,r12x,cedarwidth):
    maps=deepcopy(exterior)
    removed=['MnMaraisInlandNeck','MnR12C01R00','MnR12C01R01','MnR12C00R02','MnR12C01R02']
    for n in removed:del maps[n]
    def links(n,*edges):maps[n]['attributes']['connections']=[dict(direction=d,target_map=t,offset=o) for d,t,o in edges]
    maps['MnR11C01R00']['attributes']['width']=4
    maps['MnR12C00R01']['attributes']['height']=8
    maps['MnMaraisCedarApproach']['attributes']['width']=cedarwidth
    links('MnR12C00R00',('south','MnR12C00R01',0),('west','MnBoundaryWaters',2),('east','MnMaraisCedarApproach',13))
    links('MnR12C00R01',('north','MnR12C00R00',0))
    # Only the external R12 edge changes; every other western edge is retained.
    bw=maps['MnBoundaryWaters']['attributes']['connections']
    for e in bw:
        if e['direction']=='east':e.update(target_map='MnR12C00R00',offset=-2)
    cedarx=r12x+32; townx=226;offset=townx-cedarx
    links('MnMaraisCedarApproach',('west','MnR12C00R00',-13),('south','MnGrandMarais',offset))
    town=maps['MnGrandMarais']['attributes']['connections']
    for e in town:
        if e['direction']=='north':e['offset']=-offset
    return maps,removed


def main():
    assert hashlib.sha256(PACK.read_bytes()).hexdigest()==SHA
    pack=load_pack(PACK);allmaps=pack['data']['maps']
    exterior={n:m for n,m in allmaps.items() if n.startswith('Mn') and (m['attributes']['environment'] or '').lower() in {'town','route','outdoor'}}
    positions=positions_from_connections(exterior)
    component={'MnBoundaryWaters'};q=deque(component)
    while q:
        n=q.popleft()
        for e in allmaps[n]['attributes']['connections']:
            t=e['target_map']
            if not t.startswith('MnR12') and t not in component:component.add(t);q.append(t)
    assert 'MnDuluth' not in component
    constant={m['attributes']['map_constant']:n for n,m in allmaps.items()}
    adj={n:set() for n in allmaps}
    for n,m in allmaps.items():
        targets=[e['target_map'] for e in m['attributes']['connections']]
        targets += [constant.get(w.get('target_map_constant') or w.get('target_map'),w.get('target_map')) for w in m['events']['warps']]
        for t in targets:
            if t in adj:adj[n].add(t);adj[t].add(n)
    travel=set(component);q=deque(travel)
    while q:
        for t in adj[q.popleft()]:
            if not t.startswith('MnR12') and t not in travel:travel.add(t);q.append(t)
    out=['# Western component and detached compact placement checks','',f'Exact inspected parent file SHA256 `{SHA}`. All calculations are in metatiles. No canonical geometry, pack or source is changed.','',
         f'Excluding every `MnR12*` node and its incident edges, Boundary Waters reaches **{len(component)} physical exteriors** by native connections. Its rectangle union bounding box is `{bounds(component,exterior,positions)}` (half-open). Duluth is **not** in this component. Including native warp links in either direction yields **{len(travel)} maps**, with the same physical exteriors and {len(travel-component)} attached interiors; there is still no path to Duluth. This is topology, not proof of story/HM access.','',
         '## Exact physical component','']
    out+=table(['Exterior','Origin','Extent','Half-open rectangle'],[(n,positions[n],(exterior[n]['attributes']['width'],exterior[n]['attributes']['height']),bounds([n],exterior,positions)) for n in sorted(component)])
    out+=['','Attached native interiors (undirected warp closure): '+', '.join(f'`{n}`' for n in sorted(travel-component))+'.','', '## Every external connection edge in the original parent','']
    edges=[]
    for n,m in exterior.items():
        for e in m['attributes']['connections']:
            if (n in component)!=(e['target_map'] in component):edges.append((n,e['direction'],e['target_map'],e['offset']))
    out+=table(['Source','Direction','Target','Offset'],sorted(edges))
    warp_boundary=[]
    for n,m in allmaps.items():
        for w in m['events']['warps']:
            t=constant.get(w.get('target_map_constant') or w.get('target_map'),w.get('target_map'))
            if (n in travel)!=(t in travel):warp_boundary.append((n,w['index'],t,w['target_warp_id']))
    out+=['',f'Warp edges crossing the complete 31-map western travel component: **{len(warp_boundary)}**. All exterior-to-interior door/cave/League edges remain internal to that closure. The only severed physical link is the reciprocal Boundary Waters ↔ R12 connection above. No internal offsets, local doors or room coordinates need alteration for a rigid translation. Numeric map identities remain unchanged.','',
          '## Detached proposed graphs','',
          'Both checks below remove Neck and four retired right/bottom R12 exterior chunks, retain the two left-column R12 names (32×24 plus 32×8), shrink only the top R11 east headland from24 to4metatiles and derive the town origin from native links. All other footprints remain unchanged except the rigid western component placement. These are graph/rectangle checks; candidate terrain, collision seams, north/south story gates and player traversal are not established by them.','']
    for title,x,width in [('Shortest requested placement',190,20),('Earlier 31×19 approach',176,31),('Earlier 45×19 alternative',162,45)]:
        revised,removed=proposal(exterior,x,width)
        if width==20:
            revised['MnMaraisCedarApproach']['attributes']['height']=12
            upper=revised['MnR12C00R00']['attributes']['connections']
            upper[:]=[e for e in upper if e['direction']!='east']
            revised['MnR12C00R01']['attributes']['connections'].append(dict(direction='east',target_map='MnMaraisCedarApproach',offset=-4))
            revised['MnMaraisCedarApproach']['attributes']['connections']=[dict(direction='west',target_map='MnR12C00R01',offset=4),dict(direction='south',target_map='MnGrandMarais',offset=4)]
            next(e for e in revised['MnGrandMarais']['attributes']['connections'] if e['direction']=='north')['offset']=-4
        pos,errors=solve(revised);hits=overlaps(revised,pos)
        delta=(pos['MnBoundaryWaters'][0]-positions['MnBoundaryWaters'][0],pos['MnBoundaryWaters'][1]-positions['MnBoundaryWaters'][1])
        rigid=all(pos[n]==(positions[n][0]+delta[0],positions[n][1]+delta[1]) for n in component)
        out += [f'### {title}','',f'R12 origin `{pos["MnR12C00R00"]}`, town `{pos["MnGrandMarais"]}`, Cedar `{pos["MnMaraisCedarApproach"]}` extent `{width}×{revised["MnMaraisCedarApproach"]["attributes"]["height"]}`, Boundary Waters `{pos["MnBoundaryWaters"]}`. Western rigid translation `{delta}`, new component bounding box `{bounds(component,revised,pos)}`.','',
                f'Checked all **{len(pos)} reached exteriors** / **{len(pos)*(len(pos)-1)//2} rectangle pairs**: **{len(hits)} overlaps**, **{len(errors)} reciprocal/cycle/target errors**. Complete western rigid-translation equality: **{rigid}**. All retained original exteriors are reached: **{set(pos)==set(revised)}**.','']
        if errors:out+=table(['Graph issue'],[(repr(e),) for e in errors])+['']
        if hits:out+=table(['First map','Second map','Overlap rectangle'],hits)+['']
        out+=table(['Map','Derived origin','Extent'],[(n,pos[n],(revised[n]['attributes']['width'],revised[n]['attributes']['height'])) for n in ['MnR11C01R00','MnGrandMarais','MnMaraisCedarApproach','MnR12C00R00','MnR12C00R01','MnBoundaryWaters','MnR13C00R00','MnR13C00R02','MnEly']])+['']
    out+=['## Implementation caveat','',
          'The shortest proposal uses whole-R12 EASTcore63,54/55, which belongs to LOWER C00R01 local63,6/7. Native C00R01east→Cedarwest offsets are−4/+4, while+20/−20 describes conceptual whole-route origins and must not be written on the upper chunk. Cedarwestcore0,14/15 then matches exactly; Cedarsouthcore26/27,23 ↔ townnorth18/19,0 offsets+4/−4. The upper four Cedarwest metatiles touch R12upperchunk, so must be visibly gated rather than left as a nonexistent second west connection. Earlier31×19/45×19 proposals used upperchunkoffsets+13/−13 and need different sourceports.','',
          'For any subsequent translation `(dx,dy)`, retain every internal western offset exactly and check all translated component rectangles against every fixed exterior. Touching boundaries are legal; positive-area intersections are not. The inspection script recomputes the entire proposed graph from the guarded current parent.','']
    OUT.write_text('\n'.join(out));print(f'Wrote {OUT.relative_to(ROOT)}; western={len(component)}, warp closure={len(travel)}')


if __name__=='__main__':main()
