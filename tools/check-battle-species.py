#!/usr/bin/env python3
"""Validate original exact-species meshes, names, volume and anatomical parts.

No game content is read. Optional --allow-pending permits authoring builders
whose exports have not yet been installed as a coherent runtime increment.
"""
import argparse, ast, hashlib, json, math, re
from pathlib import Path
from model_asset_storage import read_model_json
from battle_model_assets import read_species_model, species_paths
ROOT=Path(__file__).resolve().parents[1]
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--models',type=Path,default=ROOT/'crates/crystal-voxel-view/models/battle_species');p.add_argument('--allow-pending',action='store_true');p.add_argument('--registry',type=Path,default=ROOT/'crates/crystal-voxel-view/src/battle_species_models.rs');a=p.parse_args()
 source=a.registry.read_text();registry=dict(re.findall(r'^\s*"([A-Z0-9_]+)" => "([a-z0-9_]+)",',source,re.M));assert registry,'empty exact species registry'
 tree=ast.parse((ROOT/'tools/build-battle-species.py').read_text());authored=[t.lower() for f in tree.body if isinstance(f,ast.FunctionDef) for d in f.decorator_list if isinstance(d,ast.Call) and isinstance(d.func,ast.Name) and d.func.id=='register' for t in d.args[0].value.split()];assert len(authored)==len(set(authored)),'duplicate authored species'
 assert all(name==species.lower() for species,name in registry.items());assert set(registry.values()) <= set(authored),'un-authored species registered'
 if not a.allow_pending:assert set(authored)==set(registry.values()),'pending authored exports; pass --allow-pending only for a reviewed partial increment'
 paths=species_paths(a.models)
 assert set(paths)==set(registry.values()),'runtime files and registry differ'
 geometric_hashes={};triangles=vertices=0
 for species,name in registry.items():
  path=paths[name];d=read_species_model(path);assert d['name']==name and d['version']==1;assert d['coordinate_system']=='+Y up; front +Z; floor-centered root';points=[];count=0;geometry=[];parts=[]
  for part in d['primitives']:
   parts.append(part['part']);pos=part['positions'];nor=part['normals'];ind=part['indices'];col=part['base_color'];tag=(name,part['part']);assert len(pos)>=9 and len(pos)%3==0 and len(nor)==len(pos),tag;assert all(math.isfinite(x) for x in pos+nor),tag;assert len(col)==4 and all(0<=v<=1 for v in col),tag;assert ind and len(ind)%3==0 and all(type(i)==int and 0<=i<len(pos)//3 for i in ind),tag
   ps=list(zip(*[iter(pos)]*3));ns=list(zip(*[iter(nor)]*3));points.extend(ps);volume=0
   for v in ns:assert abs(sum(x*x for x in v)-1)<.000025,(tag,'normal not unit')
   for k in range(0,len(ind),3):
    x,y,z=(ps[i] for i in ind[k:k+3]);u=[y[j]-x[j] for j in range(3)];v=[z[j]-x[j] for j in range(3)];cross=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];normal=[sum(ns[i][j] for i in ind[k:k+3]) for j in range(3)];assert sum(cross[j]*normal[j] for j in range(3))>=-1e-7,(tag,'winding and normal disagree');volume+=x[0]*(y[1]*z[2]-y[2]*z[1])+x[1]*(y[2]*z[0]-y[0]*z[2])+x[2]*(y[0]*z[1]-y[1]*z[0])
   assert volume>1e-11,(tag,'flat or inverted closed surface',volume);count+=len(ind)//3;vertices+=len(ps);geometry.append(([round(x,4) for x in pos],ind))
  assert len(parts)>=3 and len(set(parts))==len(parts),(name,'missing or repeated anatomy names')
  digest=hashlib.sha256(json.dumps(geometry,separators=(',',':')).encode()).hexdigest();assert digest not in geometric_hashes,(name,'same geometry under different species',geometric_hashes.get(digest));geometric_hashes[digest]=name
  low=[min(v[i] for v in points) for i in range(3)];high=[max(v[i] for v in points) for i in range(3)];assert abs(low[1])<.0001,(name,'not grounded');assert all(high[i]-low[i]>.005 for i in range(3)),(name,'not volumetric');assert max(abs(v) for v in low+high)<4,(name,'unexpected dimensions');assert 100<=count<22000,(name,'triangle budget',count);triangles+=count
 print(f'PASS: {len(registry)} exact original species; {triangles:,} triangles; {vertices:,} vertices; named anatomy, unique geometry, finite attributes, outward winding, smooth normals and floor roots')
 if a.allow_pending:print(f'Authoring queue: {len(set(authored)-set(registry.values()))} builders not yet registered')
if __name__=='__main__':main()
