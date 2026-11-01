#!/usr/bin/env python3
"""Focused authored city input checks; never native gameplay evidence."""
from copy import deepcopy
import hashlib,json
from pathlib import Path
from render_reference import load_pack,layout_blocks
ROOT=Path(__file__).resolve().parents[3];B=ROOT/'planning/minnesota';T=B/'towns'
def check(p,ct,layout,dossier):
 e=[];old=p['data']['maps']['MnStPaul'];const='MN_ST_PAUL';w,h=layout['dimensions_metatiles'];blocks=layout_blocks(layout);col=p['data']['tilesets']['kanto']['collision'];rooms={r['native_constant']:r for r in ct['rooms']};names={r['id']:r for r in ct['rooms']}
 def require(ok,label):
  if not ok:e.append(label)
 def kind(at):x,y=at;return col[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
 require(layout.get('source_pixels_modified') is False,'source pixel declaration')
 require(set(layout['materials'])<={'floor','lawn','encounter','flowers','forest','oak','water','boulder','sign'},'typed Material enum')
 require(ct['dimensions_metatiles']==[64,48] and [w,h]==[64,48],'extent')
 require(ct['native_module_sha256']==hashlib.sha256(json.dumps(old,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest(),'installed module SHA')
 preimages={'object':old['objects'],'background':old['events']['bg_events'],'coordinate':old['events']['coord_events'],'warp':old['events']['warps']};seen=set()
 for a in ct['anchor_overrides']:
  key=(a['kind'],a['index']);require(key not in seen,'duplicate old indexed override');seen.add(key)
  source=preimages[a['kind']][a['index']];require(a['before']==[source['x'],source['y']],'old indexed preimage')
  if a['kind'] in {'object','warp'}:
   before=source['x'],source['y'];oc=p['data']['tilesets'][old['attributes']['tileset_name']]['collision'][f"{old['blocks'][before[1]//2*32+before[0]//2]:02x}"][before[1]%2*2+before[0]%2];require(kind(a['after'])==oc,'old anchor collision')
 require(len(seen)==27,'all9object8BG10warp overrides')
 require(len(ct['event_flags'])==len(set(ct['event_flags'])),'fresh flag uniqueness')
 oldflags=p['data']['story_event_script_constants']['global'];require(all(f not in oldflags and f.startswith('EVENT_MN_SPX_') for f in ct['event_flags']),'fresh flag namespace')
 require(dossier['interiors']==json.loads((T/'MnStPaul.json').read_text())['interiors'],'original room source untouched')
 for r in ct['rooms']:
  rw,rh=r['dimensions_metatiles'];rb=[int(v,16) for row in r['rows'] for v in row.split()];require(len(rb)==rw*rh,'room dimensions')
  def rk(at):x,y=at;return p['data']['tilesets'][r['tileset']]['collision'][f'{rb[y//2*rw+x//2]:02x}'][y%2*2+x%2]
  require([v['index'] for v in r['warps']]==list(range(1,len(r['warps'])+1)),'contiguous room warp indices')
  bodies={tuple(ac['tile']) for key in ['residents','trainers','items'] for ac in r[key]}
  require(tuple(r['arrival_core']) not in bodies,'occupied primary room arrival')
  for v in r['warps']:
   if v['passive']:require(tuple(v['tile']) not in bodies,'occupied passive room arrival')
   if v['passive']:require(rk(v['tile'])=='FLOOR' and v['target']==r['native_constant'] and v['target_warp']==v['index'],'passive room safe self pairing')
   elif v['target'] in rooms:
    dest=rooms[v['target']];dw=next((t for t in dest['warps'] if t['index']==v['target_warp']),None);require(dw is not None and dw['passive'],'active room stair must target passive')
   else:
    require(v['target']==const,'room target identity')
    outer=next((d for d in ct['doors'] if d['return_warp_index']==v['target_warp']),None);require(outer is not None and outer['target']==r['id'],'room exterior return pairing')
  for key in ['residents','trainers','items']:
   for ac in r.get(key,[]):
    if key=='trainers':
     require(ac['defeated_flag'] in ct['event_flags'],'trainer fresh flag')
     require(any(t['trainer_class']==ac['trainer_class'] for t in p['data']['trainers']['trainers'].values()),'native trainer class')
     require(all(m['species'] in p['data']['pokemon'] and 1<=m['level']<=100 for m in ac['party']),'native party species/level')
    if key=='items':require(ac['item'] in p['data']['items'] and ac['flag'] in ct['event_flags'] and ac['quantity']>0,'native item and flag')
    if ac.get('service')=='pokemart':require(bool(ac['stock']) and all(i in p['data']['items'] for i in ac['stock']),'native distinct store stock')
 for i,d in enumerate(ct['doors']):
  require(d['warp_index']==11+2*i and d['return_warp_index']==12+2*i,'appended exterior warp indices')
  target=names.get(d['target']);require(target is not None,'new door map name')
  if target:
   wp=next((v for v in target['warps'] if v['index']==d['target_warp']),None);require(wp is not None and wp['passive'],'facade arrival must target passive')
  require(kind(d['tile'])=='DOOR' and kind(d['return_tile'])=='FLOOR','facade/native passive collision')
 for layer in layout['layers']:
  if 'exact facade' in layer['name']:require(blocks[layer['position'][1]*w+layer['position'][0]]==int(layer['block'],16),'complete exact facade')
  if layer['kind'] in {'polygon','path','cliff_polygon'}:
   pts=layer['points'];pairs=zip(pts,pts[1:]+pts[:1]) if layer['kind']!='path' else zip(pts,pts[1:]);require(all(a[0]==b[0] or a[1]==b[1] for a,b in pairs),'orthogonal geometry')
 require(all(kind((x,y)) not in {'FLOOR','LADDER','TALL_GRASS'} or (x,y) in {(0,48),(0,49)} for y in range(h*2) for x in range(w*2) if x in {0,w*2-1} or y in {0,h*2-1}),'terrain closed unbound walking edge')
 require(not any('CUT_TREE' in col[f'{b:02x}'] for b in blocks),'incidental Cut tree')
 require(all(not kind((x,y)).startswith('WATER') for y in range(h*2) for x in range(w*2) if x in {0,w*2-1} or y in {0,h*2-1}),'visible closed Surf boundary')
 for r in ct['rooms']:
  for label,cmds in r['programs'].items():
   for i,cmd in enumerate(cmds):
    if cmd['command'] in {'text','line','para','cont'}:require(all(len(arg.strip(chr(34)))<=18 for arg in cmd['args']),'native text18columns')
    if cmd['command']=='giveitem':require(i+2<len(cmds) and cmds[i+1]['command']=='iffalse' and cmds[i+2]['command']=='setevent','transaction refusal before success event')
    if cmd['command']=='setevent' and cmd['args'][0].endswith('_REWARDED'):require(any(v['command']=='giveitem' for v in cmds[:i]),'reward flag only after grant')
 return e

def main():
 p=load_pack(ROOT/'content-packs/minnesota-northstar-seep-shelter-review.crystalpack');ct=json.loads((T/'MnStPaul.tight.expansion.contract.json').read_text());l=json.loads((T/'MnStPaul.tight.layout.json').read_text());d=json.loads((T/'MnStPaul.tight.candidate.json').read_text());errors=check(p,ct,l,d);neg=[]
 def reject(label,change):
  cc,ll,dd=deepcopy((ct,l,d));change(cc,ll,dd);result=check(p,cc,ll,dd);neg.append(dict(id=label,rejected=bool(result),reasons=sorted(set(result))))
 reject('old_module_hash',lambda c,l,d:c.update(native_module_sha256='0'*64))
 reject('old_warp_preimage',lambda c,l,d:c['anchor_overrides'][-1].update(before=[1,1]))
 reject('duplicate_native_index',lambda c,l,d:c['anchor_overrides'].append(c['anchor_overrides'][0]))
 reject('facade_active_arrival_loop',lambda c,l,d:c['doors'][0].update(target_warp=1))
 reject('occupied_archive_arrival',lambda c,l,d:next(a for r in c['rooms'] for a in r['residents'] if a['id']=='MN_SPX_ARCHIVE_RECORD_1').update(tile=[3,9]))
 reject('passive_on_carpet',lambda c,l,d:c['rooms'][0]['warps'][2].update(tile=[2,7]))
 reject('stair_targets_active',lambda c,l,d:next(r for r in c['rooms'] if r['id']=='MnStPaulLowertownExchange')['warps'][3].update(target_warp=1))
 reject('unknown_species',lambda c,l,d:next(r for r in c['rooms'] if r['trainers'])['trainers'][0]['party'][0].update(species='INVALID_MON'))
 reject('shared_original_flag',lambda c,l,d:c['event_flags'].append('EVENT_MN_COURIER_DEFEATED'))
 reject('incidental_cut_canopy',lambda c,l,d:l['layers'].append(dict(kind='point',name='bad decorative Cut',position=[12,23],block='35')))
 reject('unsafe_unbound_annex_floor',lambda c,l,d:l['layers'].append(dict(kind='point',name='bad open annex',position=[14,0],block='31')))
 reject('unsafe_surf_edge',lambda c,l,d:l['layers'].append(dict(kind='point',name='bad open sea',position=[63,25],block='43')))
 reject('diagonal_polygon',lambda c,l,d:l['layers'].append(dict(kind='polygon',name='bad diagonal',material='lawn',points=[[2,2],[4,3],[2,4]])))
 reject('unrelated_old_room_drift',lambda c,l,d:d['interiors'][0]['rows'].__setitem__(0,'00 00 00 00 00'))
 def premature(c,l,d):
  r=next(r for r in c['rooms'] if r['id']=='MnStPaulFairQuizPavilion');r['programs']['MN_SPX_QUIZ_HOST'].insert(0,dict(command='setevent',args=['EVENT_MN_SPX_QUIZ_REWARDED']))
 reject('premature_quest_success',premature)
 reject('raw_text_overflow',lambda c,l,d:next(r for r in c['rooms'] if r['id']=='MnStPaulFairQuizPavilion')['programs']['MN_SPX_QUIZWrongText'][0].update(args=['"Definitely over eighteen columns"']))
 out=B/'renders/st-paul/tight-20261027/contract-proof';out.mkdir(exist_ok=True);report=dict(source_only=True,native_playability_proved=False,errors=errors,negative_cases=neg,all_negative_cases_rejected=all(v['rejected'] for v in neg),source_hashes={f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in [T/'MnStPaul.tight.layout.json',T/'MnStPaul.tight.expansion.contract.json',T/'MnStPaul.tight.candidate.json']});(out/'review.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2));assert not errors and report['all_negative_cases_rejected']
if __name__=='__main__':main()
