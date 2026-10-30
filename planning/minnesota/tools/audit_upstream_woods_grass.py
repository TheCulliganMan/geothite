#!/usr/bin/env python3
"""Independently preserve every field except five reviewed upper-river bank cells."""
import argparse,base64,hashlib,json
from copy import deepcopy
from pathlib import Path
from render_reference import load_pack,layout_blocks
from audit_winding_shelter_541 import raw_identity
MAP='MnMississippiUpstreamWoodsC01R00'
PATCHES=[(4,9,3,76),(5,9,3,1),(5,8,2,3),(7,9,2,3),(7,10,2,3)]
def audit(before,after,layout):
 checks=[]
 def equal(name,a,b):checks.append(dict(check=name,passed=a==b))
 a,b=before['data'],after['data'];old,new=a['maps'][MAP],b['maps'][MAP];label=old['attributes']['blocks_label'];expected=old['blocks'].copy()
 for x,y,pre,post in PATCHES:
  equal(f'preimage {x},{y}',pre,expected[y*32+x]);expected[y*32+x]=post
 equal('map roster',set(a['maps']),set(b['maps']))
 equal('sole changed map',{MAP},{n for n in a['maps']if a['maps'][n]!=b['maps'].get(n)})
 equal('five exact terrain edits',expected,new['blocks'])
 equal('source extent',[64,68],layout['dimensions_metatiles'])
 equal('source native alias','johto',layout['compiled_tileset'])
 compiled=layout_blocks(layout)
 for name,module in b['maps'].items():
  if name.startswith('MnMississippiUpstreamWoods'):
   col=int(name[-5:-3]);row=int(name[-2:]);cells=[compiled[(row*24+y)*64+col*32+x]for y in range(module['attributes']['height'])for x in range(32)];equal('full current source '+name,module['blocks'],cells)
 equal('all owned attributes actors events and programs exact',old,dict(new,blocks=old['blocks']))
 equal('old encoded mirror',old['blocks'],list(base64.b64decode(a['map_blocks'][label],validate=True)))
 equal('new encoded mirror',expected,list(base64.b64decode(b['map_blocks'][label],validate=True)))
 normalized=dict(b,maps=dict(b['maps']),map_blocks=dict(b['map_blocks']))
 normalized['maps'][MAP]=dict(new,blocks=old['blocks']);normalized['map_blocks'][label]=a['map_blocks'][label]
 equal('ENTIRE data residual exact',a,normalized)
 for n in sorted(set(a['maps'])-{MAP}):equal(n+' complete exact',a['maps'][n],b['maps'].get(n))
 equal('report residual exact',before['report'],dict(after['report'],manifests=before['report']['manifests']))
 equal('sole manifest append',before['report']['manifests']+['minnesota-upstream-woods-grass-v1'],after['report']['manifests'])
 equal('identity counts exact',before['identity'],dict(after['identity'],content_hash=before['identity']['content_hash'],runtime_modpack_id=before['identity']['runtime_modpack_id']))
 for k in before:
  if k not in ['data','report','identity']:equal('whole field '+k,before[k],after[k])
 return checks

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--before',type=Path,required=True);ap.add_argument('--pack',type=Path,required=True);ap.add_argument('--layout',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);args=ap.parse_args()
 inputs=[(p,p.read_bytes())for p in [args.before,args.pack,args.layout]];before,after=load_pack(args.before),load_pack(args.pack);layout=json.loads(args.layout.read_text());checks=audit(before,after,layout);assert all(v['passed']for v in checks),[v for v in checks if not v['passed']];negatives=[]
 for mutation in ['owned terrain','owned native attributes','other map','shared art','foreign wild slot']:
  bad=dict(after,data=dict(after['data'],maps=dict(after['data']['maps'])));m=deepcopy(after['data']['maps'][MAP]);bad['data']['maps'][MAP]=m
  if mutation=='owned terrain':m['blocks'][0]=1
  elif mutation=='owned native attributes':m['attributes']['border_block']=1
  elif mutation=='other map':
   n=next(n for n in bad['data']['maps']if n!=MAP);other=deepcopy(bad['data']['maps'][n]);other['blocks'][0]^=1;bad['data']['maps'][n]=other
  elif mutation=='foreign wild slot':
   bad['data']['wild_encounters']=deepcopy(after['data']['wild_encounters']);bad['data']['wild_encounters'][MAP]['grass']['day'][0]['species']='PIDGEY'
  else:
   bad['runtime_files']=dict(after['runtime_files']);key=next(iter(bad['runtime_files']));bad['runtime_files'][key]=b'unreviewed art'
  failed=[v['check']for v in audit(before,bad,layout)if not v['passed']];assert failed;negatives.append(dict(mutation=mutation,rejected=True,failed_checks=failed))
 identities={str(p):raw_identity(p)for p in [args.before,args.pack]};assert all(v['exact']for v in identities.values());assert all(p.read_bytes()==raw for p,raw in inputs)
 report=dict(status='PASS',checks=checks,negative_cases=negatives,raw_identities=identities,inputs=[dict(path=str(p),sha256=hashlib.sha256(raw).hexdigest())for p,raw in inputs],reviewer_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest());args.output.parent.mkdir(parents=True,exist_ok=True);assert not args.output.exists();args.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(status='PASS',checks=len(checks),negatives=len(negatives))))
if __name__=='__main__':main()
