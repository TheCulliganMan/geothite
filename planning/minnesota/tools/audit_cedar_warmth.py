#!/usr/bin/env python3
"""Independently verify the sole coastal Cedar art change and exact residual."""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path
from audit_city_expansions import fingerprint
from audit_winding_shelter_541 import raw_identity
from render_reference import load_pack, layout_blocks
MAP='MnMaraisCedarApproach'
WARM='mn_north_shore'
MAPPING={0x83:0x01,0x84:0x03,0x88:0x45,0x8b:0x05}
MANIFEST='minnesota-marais-cedar-warmth-v1'

def audit(before,after,layout):
 checks=[]
 def equal(label,a,b):
  checks.append(dict(check=label,passed=a==b))
 a,b=before['data'],after['data'];old,new=a['maps'][MAP],b['maps'][MAP];label=old['attributes']['blocks_label'];expected=[MAPPING.get(v,v)for v in old['blocks']];attrs=dict(old['attributes'],tileset_name=WARM,border_block=5)
 equal('complete map roster',set(a['maps']),set(b['maps']))
 equal('exact changed map roster',{MAP},{n for n in a['maps']if a['maps'][n]!=b['maps'].get(n)})
 equal('exact warm native attributes',attrs,new['attributes'])
 equal('exact warm attribute mirror',attrs,b['map_attributes'][MAP])
 equal('complete native block mapping',expected,new['blocks'])
 equal('staged source alias',WARM,layout['compiled_tileset'])
 equal('staged source extent',[20,12],layout['dimensions_metatiles'])
 equal('all staged source cells',expected,layout_blocks(layout))
 equal('unique owned encoded label',[MAP],sorted(n for n,m in a['maps'].items()if m['attributes']['blocks_label']==label))
 equal('old encoded native payload',old['blocks'],list(base64.b64decode(a['map_blocks'][label],validate=True)))
 equal('new encoded native payload',expected,list(base64.b64decode(b['map_blocks'][label],validate=True)))
 equal('all native objects/events/programs preserved',old,dict(new,blocks=old['blocks'],attributes=old['attributes']))
 cold=a['tilesets'][old['attributes']['tileset_name']]['collision'];warm=b['tilesets'][WARM]['collision']
 equal('every used quadrant and border collision exact',True,all(cold[f'{x:02x}']==warm[f'{z:02x}']for x,z in zip(old['blocks']+[old['attributes']['border_block']],new['blocks']+[new['attributes']['border_block']])))
 equal('no warm ICE',False,any('ICE'in warm.get(f'{v:02x}',[])for v in new['blocks']+[new['attributes']['border_block']]))
 normalized=dict(b,maps=dict(b['maps']),map_attributes=dict(b['map_attributes']),map_blocks=dict(b['map_blocks']))
 normalized['maps'][MAP]=dict(new,blocks=old['blocks'],attributes=old['attributes']);normalized['map_attributes'][MAP]=a['map_attributes'][MAP];normalized['map_blocks'][label]=a['map_blocks'][label]
 equal('ENTIRE data residual after only declared terrain and attributes restored',a,normalized)
 for n in sorted(set(a['maps'])-{MAP}):equal(n+' complete native module exact',a['maps'][n],b['maps'].get(n))
 equal('sole provenance manifest appended',before['report'],dict(after['report'],manifests=before['report']['manifests']))
 equal('manifest sequence exact',before['report']['manifests']+[MANIFEST],after['report']['manifests'])
 equal('all identity counts retained',before['identity'],dict(after['identity'],content_hash=before['identity']['content_hash'],runtime_modpack_id=before['identity']['runtime_modpack_id']))
 for k in before:
  if k not in ['data','report','identity']:equal('whole pack field exact '+k,before[k],after[k])
 return checks

def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--before',type=Path,required=True);ap.add_argument('--pack',type=Path,required=True);ap.add_argument('--layout',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);args=ap.parse_args();inputs=[(p,p.read_bytes())for p in [args.before,args.pack,args.layout]];before,after=load_pack(args.before),load_pack(args.pack);layout=json.loads(args.layout.read_text());checks=audit(before,after,layout);assert all(x['passed']for x in checks),[x for x in checks if not x['passed']];negatives=[]
 for kind in ['coastal winter alias restored','owned actor drift','wrong warm tile','unrelated inland art drift']:
  bad=dict(after,data=dict(after['data'],maps=dict(after['data']['maps'])));m=deepcopy(after['data']['maps'][MAP]);bad['data']['maps'][MAP]=m
  if kind=='coastal winter alias restored':m['attributes']['tileset_name']='mn_winter_north_shore'
  elif kind=='owned actor drift':m['objects'][0]['x']+=1
  elif kind=='wrong warm tile':m['blocks'][0]=4
  else:
   bad['runtime_files']=dict(after['runtime_files']);file=next(k for k in bad['runtime_files']if 'mn_winter' in k);bad['runtime_files'][file]=b'unreviewed art drift'
  rejected=audit(before,bad,layout);failed=[x['check']for x in rejected if not x['passed']];assert failed,kind;negatives.append(dict(mutation=kind,rejected=True,failed_checks=failed))
 identities={str(p):raw_identity(p)for p in [args.before,args.pack]};assert all(v['exact']for v in identities.values());assert all(p.read_bytes()==raw for p,raw in inputs),'input changed during audit'
 report=dict(status='PASS',scope=__doc__,checks=checks,negative_cases=negatives,changed_metatiles=sum(x!=y for x,y in zip(before['data']['maps'][MAP]['blocks'],after['data']['maps'][MAP]['blocks'])),inputs=[dict(path=str(p),sha256=hashlib.sha256(raw).hexdigest())for p,raw in inputs],raw_content_identities=identities,reviewer_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest());raw=(json.dumps(report,indent=2)+'\n').encode();args.output.mkdir(parents=True,exist_ok=False);path=args.output/('proof-'+hashlib.sha256(raw).hexdigest()+'.json');path.write_bytes(raw);print(json.dumps(dict(status='PASS',checks=len(checks),negative_cases=len(negatives),changed_metatiles=report['changed_metatiles'],proof=str(path))))
if __name__=='__main__':main()
