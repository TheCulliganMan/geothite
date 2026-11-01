#!/usr/bin/env python3
"""Bounded row-only city review with actor-aware reachability and negative cases."""
import argparse
import collections
import copy
import hashlib
import json
import sys
from pathlib import Path

from render_reference import load_pack
from refine_st_paul_interiors import ROWS

ROOT=Path(__file__).resolve().parents[3]
BASE=ROOT/'planning/minnesota'
OLD=BASE/'renders/st-paul/tight-20261027/v09'
T=BASE/'towns'

def verify(before,after,pack):
    errors=[];stats=[]
    b=copy.deepcopy(before);a=copy.deepcopy(after)
    if len(b['rooms'])!=len(a['rooms']):errors.append('room roster length')
    for old,new in zip(b['rooms'],a['rooms']):
        if old['id']!=new['id']:errors.append('room roster')
        if old['id'].removeprefix('MnStPaul') not in ROWS and old['rows']!=new['rows']:errors.append('unowned room rows')
        new['rows']=old['rows']
    if b!=a:errors.append('non-row native contract drift')
    def neighbors(p):
        x,y=p;return [(x+1,y),(x-1,y),(x,y+1),(x,y-1)]
    for old,new in zip(before['rooms'],after['rooms']):
        if old['rows']==new['rows']:continue
        w,h=new['dimensions_metatiles'];co=pack['data']['tilesets'][new['tileset']]['collision']
        for r in [old,new]:
            if len(r['rows'])!=h or any(len(row.split())!=w for row in r['rows']):errors.append('room extent '+r['id']);break
        else:
            ob=[int(v,16) for row in old['rows'] for v in row.split()];nb=[int(v,16) for row in new['rows'] for v in row.split()]
            if any(f'{v:02x}' not in co for v in nb):errors.append('unknown art '+new['id']);continue
            def kind(bs,p):
                x,y=p;return co[f'{bs[y//2*w+x//2]:02x}'][y%2*2+x%2]
            coords={(x,y) for y in range(h*2) for x in range(w*2)}
            if any(kind(nb,q) not in {'FLOOR','WALL','BOOKSHELF','COUNTER','WINDOW'} and kind(nb,q)!=kind(ob,q) for q in coords):errors.append('new unbound functional art '+new['id'])
            actors=[v for key in ['residents','trainers','items'] for v in new[key]]
            bodies={tuple(v['tile']) for v in actors}
            reserve={tuple(new['arrival_core'])}
            for v in actors:
                p=tuple(v['tile']);reserve.add(p)
                if 'approach' in v:reserve.add(tuple(v['approach']))
                reserve.update(q for q in neighbors(p) if q in coords and kind(ob,q)=='FLOOR')
            for v in new['warps']:
                p=tuple(v['tile'])
                if kind(nb,p)!=kind(ob,p):errors.append('warp collision '+new['id'])
                if v['passive']:reserve.add(p)
                reserve.update(q for q in neighbors(p) if q in coords and kind(ob,q)=='FLOOR')
            if any(kind(nb,q)!='FLOOR' for q in reserve):errors.append('reserved original Floor '+new['id']+' '+str(sorted(q for q in reserve if kind(nb,q)!='FLOOR')))
            allowed={q for q in coords if kind(nb,q) in {'FLOOR','WARP_CARPET_DOWN','STAIRCASE','LADDER'} and q not in bodies}
            start=tuple(new['arrival_core']);seen={start};todo=collections.deque([start])
            while todo:
                for q in neighbors(todo.popleft()):
                    if q in allowed and q not in seen:seen.add(q);todo.append(q)
            if allowed-seen:errors.append('unreachable ordinary floor '+new['id']+' '+str(sorted(allowed-seen)))
            if start in bodies or kind(nb,start)!='FLOOR':errors.append('occupied or blocked arrival '+new['id'])
            for v in new['warps']:
                q=tuple(v['tile'])
                if q in bodies:errors.append('occupied warp '+new['id'])
                if v['passive'] and q not in seen:errors.append('unreachable passive '+new['id'])
                if not v['passive'] and not any(t in seen for t in neighbors(q)):errors.append('unreachable exit '+new['id'])
            for v in actors:
                q=tuple(v['tile'])
                if kind(nb,q)!='FLOOR' or not any(t in seen for t in neighbors(q)):errors.append('actor approach '+new['id'])
                if 'approach' in v and tuple(v['approach']) not in seen:errors.append('explicit approach '+new['id'])
            def square(bs):
                dp={};best=0
                for y in range(h*2):
                    for x in range(w*2):
                        if (x,y) not in bodies and kind(bs,(x,y))=='FLOOR':
                            dp[x,y]=1+min(dp.get((x-1,y),0),dp.get((x,y-1),0),dp.get((x-1,y-1),0));best=max(best,dp[x,y])
                return best
            if ob!=nb:stats.append(dict(id=new['id'],changed_metatiles=sum(x!=y for x,y in zip(ob,nb)),reserved_floor_points=len(reserve),ordinary_reachable=len(seen),before_largest_clear_floor_square=square(ob),after_largest_clear_floor_square=square(nb),unreachable_ordinary=[]))
    return errors,stats

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--revision',default='v10');args=ap.parse_args();out=BASE/'renders/st-paul/tight-20261027'/args.revision;out.mkdir(exist_ok=True)
    before=json.loads((OLD/'MnStPaul.tight.expansion.contract.json').read_text());after=json.loads((T/'MnStPaul.tight.expansion.contract.json').read_text());p=load_pack(ROOT/'content-packs/minnesota-three-cities-tight-v07.crystalpack')
    control,_=verify(before,before,p);assert not control,control
    errors,stats=verify(before,after,p)
    assert (OLD/'MnStPaul.tight.layout.json').read_bytes()==(T/'MnStPaul.tight.layout.json').read_bytes()
    bd=json.loads((OLD/'MnStPaul.tight.candidate.json').read_text());ad=json.loads((T/'MnStPaul.tight.candidate.json').read_text());ad['new_rooms']=bd['new_rooms'];assert ad==bd
    negatives=[]
    def reject(name,mutation):
        bad=copy.deepcopy(after);mutation(bad);e,_=verify(before,bad,p);negatives.append(dict(name=name,rejected=bool(e),errors=e))
    def room(c,id):return next(r for r in c['rooms'] if r['id']=='MnStPaul'+id)
    def cell(c,id,x,y,value):
        r=room(c,id);row=r['rows'][y].split();row[x]=value;r['rows'][y]=' '.join(row)
    reject('unrelated program',lambda c:c['programs'].update({'UNRELATED':[]}))
    reject('actor moved',lambda c:room(c,'CapitolArchive')['residents'][1].update(tile=[3,9]))
    reject('quest program drift',lambda c:room(c,'CapitolArchive')['programs'].clear())
    reject('warp target drift',lambda c:room(c,'CapitolArchive')['warps'][0].update(target_warp=999))
    reject('trainer party drift',lambda c:room(c,'EastWorkshop')['trainers'][0]['party'][0].update(level=100))
    reject('reward flag drift',lambda c:room(c,'SummitMoveStudyHouse')['items'][0].update(flag='EVENT_WRONG'))
    reject('clinic unrelated terrain',lambda c:cell(c,'EastAidClinic',0,0,'00'))
    reject('archive arrival blocked',lambda c:cell(c,'CapitolArchive',1,4,'1a'))
    reject('archive explicit approach blocked',lambda c:cell(c,'CapitolArchive',2,4,'1a'))
    reject('trainer floor blocked',lambda c:cell(c,'EastWorkshop',4,3,'1a'))
    reject('carpet removed',lambda c:cell(c,'CommunityHall',1,5,'0f'))
    reject('window floor isolated by workbench',lambda c:cell(c,'EastWorkshop',5,0,'03'))
    reject('unbound native ladder introduced',lambda c:cell(c,'CommunityHall',6,4,'25'))
    reject('room dimensions changed',lambda c:room(c,'CivicMuseum').update(dimensions_metatiles=[9,6]))
    report=dict(source_only=True,native_playability_proved=False,errors=errors,changed_rooms=stats,changed_room_count=len(stats),distinct_refined_rows=len({tuple(r['rows']) for r in after['rooms'] if r['id'].removeprefix('MnStPaul') in ROWS}),exact_other_contract_values=True,exterior_layout_byte_exact=True,dossier_outside_new_room_rows_exact=True,negative_cases=negatives,all_negative_cases_rejected=all(v['rejected'] for v in negatives),source_hashes={f:hashlib.sha256((T/f).read_bytes()).hexdigest() for f in ['MnStPaul.tight.layout.json','MnStPaul.tight.expansion.contract.json','MnStPaul.tight.candidate.json']})
    (out/'interior-refinement-proof.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(errors=errors,changed_rooms=len(stats),negative_cases=len(negatives),all_rejected=report['all_negative_cases_rejected']),indent=2));assert not errors and report['all_negative_cases_rejected']

if __name__=='__main__':main()
