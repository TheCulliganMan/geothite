#!/usr/bin/env python3
"""Independently preserve every field except ten reviewed Confluence bank cells."""
import argparse,base64,hashlib,json
from copy import deepcopy
from pathlib import Path
from render_reference import load_pack,layout_blocks
from audit_winding_shelter_541 import raw_identity
MAP='MnMississippiConfluenceC01R00'
PATCHES={MAP:[(6,2,2,3),(7,1,1,3)],'MnMississippiConfluenceC02R00':[(19,3,2,5),(20,3,2,86),(21,3,2,5),(20,1,2,3),(20,2,2,3),(19,2,2,3),(21,2,1,3),(22,2,2,3)]}
def audit(before,after,layout):
 checks=[]
 def equal(name,a,b):checks.append(dict(check=name,passed=a==b))
 a,b=before['data'],after['data'];normalized=dict(b,maps=dict(b['maps']),map_blocks=dict(b['map_blocks']))
 equal('map roster',set(a['maps']),set(b['maps']))
 equal('exactly two owned changed maps',set(PATCHES),{n for n in a['maps']if a['maps'][n]!=b['maps'].get(n)})
 for n,patches in PATCHES.items():
  old,new=a['maps'][n],b['maps'][n];expected=old['blocks'].copy();label=old['attributes']['blocks_label']
  for x,y,pre,post in patches:equal(f'{n} preimage {x},{y}',pre,expected[y*32+x]);expected[y*32+x]=post
  equal(n+' exact terrain',expected,new['blocks'])
  equal(n+' attributes actors events programs',old,dict(new,blocks=old['blocks']))
  equal(n+' old encoded mirror',old['blocks'],list(base64.b64decode(a['map_blocks'][label],validate=True)))
  equal(n+' new encoded mirror',expected,list(base64.b64decode(b['map_blocks'][label],validate=True)))
  normalized['maps'][n]=dict(new,blocks=old['blocks']);normalized['map_blocks'][label]=a['map_blocks'][label]
 equal('source extent',[74,12],layout['dimensions_metatiles']);equal('native alias','johto',layout['compiled_tileset']);compiled=layout_blocks(layout)
 for n,ox,oy in [('MnMississippiConfluenceC00R00',0,4),(MAP,6,0),('MnMississippiConfluenceC02R00',38,0),('MnMississippiConfluenceC03R00',70,0)]:
  m=b['maps'][n];equal(n+' entire current source',m['blocks'],[compiled[(oy+y)*74+ox+x]for y in range(m['attributes']['height'])for x in range(m['attributes']['width'])])
 equal('ENTIRE data residual exact',a,normalized)
 for n in sorted(set(a['maps'])-set(PATCHES)):equal(n+' complete exact',a['maps'][n],b['maps'].get(n))
 equal('report residual exact',before['report'],dict(after['report'],manifests=before['report']['manifests']))
 equal('sole manifest append',before['report']['manifests']+['minnesota-confluence-grass-v1'],after['report']['manifests'])
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
