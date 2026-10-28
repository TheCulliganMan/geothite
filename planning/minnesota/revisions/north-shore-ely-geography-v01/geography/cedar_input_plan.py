#!/usr/bin/env python3
"""Declared source Foot input plans; these are not production execution receipts."""
import cbor2,json
from collections import deque
from pathlib import Path
ROOT=Path(__file__).resolve().parents[5];source=ROOT/'planning/minnesota/revisions/north-shore-ely-geography-v01/geography/cedar-neck-v03/contract.json';contract=json.loads(source.read_bytes());p=cbor2.loads((ROOT/'content-packs/minnesota-superior-warm-coast-v01.crystalpack').read_bytes()[22:]);collision=p['data']['tilesets']['mn_winter_north_shore']['collision'];plans=[]
for name,w,start,end,label in [('cedar',32,(63,8),(0,8),'town_to_neck'),('cedar',32,(0,8),(63,8),'neck_to_town'),('neck',8,(15,8),(0,8),'cedar_to_gunflint'),('neck',8,(0,8),(15,8),'gunflint_to_cedar'),('cedar',32,(63,8),(55,21),'town_to_native_stair'),('cedar',32,(55,21),(55,18),'stair_to_walker_facing_up'),('cedar',32,(55,18),(55,21),'summit_return'),('cedar',32,(0,8),(29,18),'neck_to_keeper_facing_down')]:
 b=[int(v,16)for row in contract[name]['rows']for v in row.split()];actors={(29,19),(55,17)}if name=='cedar'else set()
 def kind(x,y):return collision[f'{b[y//2*w+x//2]:02x}'][y%2*2+x%2]
 allowed={(x,y)for y in range(38)for x in range(w*2)if kind(x,y)in{'FLOOR','TALL_GRASS'}and(x,y)not in actors};q=deque([start]);prev={start:None}
 while q:
  a=q.popleft()
  if a==end:break
  x,y=a
  for nn,button in [((x-1,y),'left'),((x+1,y),'right'),((x,y-1),'up'),((x,y+1),'down')]:
   if nn in allowed and nn not in prev:prev[nn]=(a,button);q.append(nn)
 assert end in prev;actions=[];coords=[end];pos=end
 while pos!=start:pos,button=prev[pos];actions.append(button);coords.append(pos)
 actions.reverse();coords.reverse();runs=[]
 for a in actions:
  if runs and runs[-1]['button']==a:runs[-1]['count']+=1
  else:runs.append(dict(button=a,count=1))
 plans.append(dict(map='MnMaraisCedarApproach'if name=='cedar'else'MnMaraisInlandNeck',label=label,start_core=list(start),end_core=list(end),steps=len(actions),runs=runs,positions_core=coords))
receipt=dict(status='Source input candidates only. Parent must execute VisibleShellController; these are not movement, battle, gate or save acceptance.',contract=source.relative_to(ROOT).as_posix(),plans=plans,planned_connections=[dict(source='MnGrandMarais',source_core=[0,8],button='left',target='MnMaraisCedarApproach',target_core=[63,8]),dict(source='MnMaraisCedarApproach',source_core=[0,8],button='left',target='MnMaraisInlandNeck',target_core=[15,8]),dict(source='MnMaraisInlandNeck',source_core=[0,8],button='left',target='MnR12C01R01',target_core=[63,8])],interactions=[dict(map='MnMaraisCedarApproach',core=[55,18],face='up',target=[55,17],action='A',expected='existing WALKER native trainer program'),dict(map='MnMaraisCedarApproach',core=[29,18],face='down',target=[29,19],action='A',expected='existing KEEPER native trainer program')]);out=ROOT/'planning/minnesota/renders/north-shore-ely-geography-v01/cedar-neck-v03/controller-input-candidates.json';assert not out.exists();out.write_text(json.dumps(receipt,indent=2)+'\n');print([(x['label'],x['steps'])for x in plans])
