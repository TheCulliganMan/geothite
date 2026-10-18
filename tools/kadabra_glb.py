#!/usr/bin/env python3
"""Canonical reviewed Kadabra sculpture with an extended neck and articulated psychic props."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
SPECIES=('kadabra',);HEIGHTS={'kadabra':1.12}
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

PARTS={'kadabra':('Tapered abdomen', 'Angular chest cuirass', 'Narrow neck', 'Sculpted fox skull and projecting tapered muzzle', 'Long tapered fox ear', 'Recessed ochre ear plane', 'Narrow focused white eye', 'Focused slit pupil', 'Angular brow plane', 'Long swept moustache', 'Sculpted shoulder armor', 'Bent upper arm', 'Sculpted tapered forearm', 'Three-fingered hand palm', 'Curled hand finger', 'Pale finger claw', 'Curled hand finger.001', 'Pale finger claw.001', 'Curled hand finger.002', 'Pale finger claw.002', 'Bent muscular hind leg', 'Splayed hind foot', 'Three long hind toes', 'Ivory toe claw', 'Three long hind toes.001', 'Ivory toe claw.001', 'Three long hind toes.002', 'Ivory toe claw.002', 'Long tapered fox ear.001', 'Recessed ochre ear plane.001', 'Narrow focused white eye.001', 'Focused slit pupil.001', 'Angular brow plane.001', 'Long swept moustache.001', 'Sculpted shoulder armor.001', 'Bent upper arm.001', 'Sculpted tapered forearm.001', 'Three-fingered hand palm.001', 'Curled hand finger.003', 'Pale finger claw.003', 'Curled hand finger.004', 'Pale finger claw.004', 'Curled hand finger.005', 'Pale finger claw.005', 'Bent muscular hind leg.001', 'Splayed hind foot.001', 'Three long hind toes.003', 'Ivory toe claw.003', 'Three long hind toes.004', 'Ivory toe claw.004', 'Three long hind toes.005', 'Ivory toe claw.005', 'Heavy curved psychic tail', 'Broad tail saddle band', 'Three wavy abdomen markings', 'Three wavy abdomen markings.001', 'Three wavy abdomen markings.002', 'Five-point red forehead star', 'Held silver spoon handle', 'Hollow silver spoon bowl')}
REST_DIGESTS={'kadabra': '465503e6eeaaa78d03ec1985ed08c33fb2bea8dd2097d449bb829c5f3ee5e6a2'}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024
@lru_cache(maxsize=1)
def authoring_model(species='kadabra'):
 if species!='kadabra':raise ValueError('unknown Kadabra species')
 return neutral_model((Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/kadabra.glb').read_bytes())
# Baked sculpt transform, measured from the original unmodified abdomen rings.
# Blender authoring uses +Z up/-Y front; the canonical file is +Y up/+Z front.
SCALE=.8811947368421053
OFFSET=(-.0000025,-.0035247368421053,.1273585)
def normalized(species,point):
 x,y,z=point;return (x*SCALE+OFFSET[0],z*SCALE+OFFSET[1],-y*SCALE+OFFSET[2])
def raw_point(point):
 x,y,z=point;return ((x-OFFSET[0])/SCALE,-(z-OFFSET[2])/SCALE,(y-OFFSET[1])/SCALE)
def arm_points(side):
 return ((side*.225,-.015,.635),(side*.34,-.045,.57 if side<0 else .68),(side*.39,-.18,.68 if side<0 else .89))
def joints(species):
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.015,.43)),('head',1,(0.,0.,.835))]
 for side in (-1,1):
  shoulder,elbow,wrist=arm_points(side);j=len(raw);label='left' if side<0 else 'right';raw += [('arm_'+label,1,shoulder),('elbow_'+label,j,elbow),('wrist_'+label,j+1,wrist)]
 raw += [('leg_left',0,(-.21,.02,.29)),('leg_right',0,(.21,.02,.29)),
         ('ear_left',2,(-.165,.03,.985)),('ear_right',2,(.165,.03,.985)),
         ('moustache_left',2,(-.018,-.262,.902)),('moustache_right',2,(.018,-.262,.902)),
         ('tail_base',1,(0.,.125,.30)),('tail_tip',15,(-.05,.48,.26)),('spoon',8,(.392,-.239,.90))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={'kadabra':joints('kadabra')}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
@lru_cache(maxsize=1)
def tail_path():
 from bovine_sculpt import smooth
 return smooth(((0,.125,.30),(0,.33,.23),(-.05,.48,.26),(-.10,.56,.46),(-.07,.50,.66)),(.11,.145,.13,.095,.018))[0]
def projection(point,a,b):
 d=[b[k]-a[k] for k in range(3)];return sum((point[k]-a[k])*d[k] for k in range(3))/sum(v*v for v in d)
def path_fraction(point,path):
 lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.;best=(math.inf,0.)
 for a,b,length in zip(path,path[1:],lengths):
  t=max(0.,min(1.,projection(point,a,b)));best=min(best,(math.dist(point,[a[k]+t*(b[k]-a[k]) for k in range(3)]),(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(species,part,position):
 point=raw_point(position);x,y,z=point;side=-1 if x<0 else 1
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Splayed hind foot','Three long hind toes','Ivory toe claw')):return rigid(0)
 if part.startswith(('Long tapered fox ear','Recessed ochre ear plane')):return rigid(11 if side<0 else 12)
 if part.startswith('Long swept moustache'):return rigid(14 if part.endswith('.001') else 13)
 if part.startswith(('Held silver spoon','Hollow silver spoon')):return rigid(17)
 if part.startswith('Sculpted shoulder armor'):return rigid(1)
 if part in ('Tapered abdomen','Angular chest cuirass') or part.startswith('Three wavy abdomen markings'):
  a=smoothstep(.23,.42,z);return [0,1,0,0],[1.-a,a,0.,0.]
 if part=='Narrow neck':
  a=smoothstep(.67,.82,z);return [1,2,0,0],[1.-a,a,0.,0.]
 if part.startswith('Bent muscular hind leg'):
  owner=9 if side<0 else 10;a=smoothstep(.09,.26,z);b=.01+.70*smoothstep(.08,.17,z)*(1.-smoothstep(.30,.41,z))
  return [owner,0,1,0],[b,(1.-a)*(1.-b),a*(1.-b),0.]
 if part.startswith(('Bent upper arm','Sculpted tapered forearm','Three-fingered hand palm','Curled hand finger','Pale finger claw')):
  upper=3 if side<0 else 6;elbow=upper+1;hand=upper+2
  if part.startswith(('Three-fingered hand palm','Curled hand finger','Pale finger claw')):return rigid(hand)
  shoulder,elbow_point,wrist=arm_points(side)
  if part.startswith('Bent upper arm'):
   a=smoothstep(0.,.40,projection(point,shoulder,elbow_point));return [1,upper,0,0],[1.-a,a,0.,0.]
  t=projection(point,elbow_point,wrist);a=smoothstep(.05,.40,t);b=smoothstep(.72,.95,t)
  return [upper,elbow,hand,0],[(1.-a)*(1.-b),a*(1.-b),b,0.]
 if part.startswith(('Heavy curved psychic tail','Broad tail saddle band')):
  t=path_fraction(point,tail_path());a=smoothstep(0.,.18,t);b=smoothstep(.48,.82,t)
  return [1,15,16,0],[1.-a,a*(1.-b),a*b,0.]
 return rigid(2)
def pose_angles(species,clip,t):
 if species!='kadabra' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Kadabra clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*18
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength;out=[(0.,0.,0.),(0.,.008*a,0.),(.010*a,.015*b,0.)]
 for side in (-1,1):out += [(.015*b,side*.018*b,side*.024*a),(.025*a,side*.015*b,0.),(.018*b,0.,side*.018*a)]
 out += [(0.,-.010*a,-.006*b),(0.,.010*a,.006*b),(0.,-.018*a,-.020*b),(0.,.018*a,.020*b),(.010*b,0.,-.018*a),(.010*b,0.,.018*a),(.015*b,.025*a,0.),(.020*a,.045*b,0.),(.025*b,.025*a,-.025*b)]
 return out

def pack_parts(species,model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name=source['part'];key=None;label=None;side='left' if sum(source['positions'][::3])<0 else 'right'
  if name.startswith(('Three-fingered hand palm','Curled hand finger')):key='hand_'+side;label='Hand / '+side+' palm and three curled fingers'
  elif name.startswith('Pale finger claw'):key='hand_claw_'+side;label='Hand claw / '+side+' three ivory finger tips'
  elif name.startswith(('Splayed hind foot','Three long hind toes')):key='foot_'+side;label='Foot / '+side+' instep and three splayed toes'
  elif name.startswith('Ivory toe claw'):key='foot_claw_'+side;label='Foot claw / '+side+' three ivory tips'
  elif name.startswith('Three wavy abdomen markings'):key='waves';label='Marking / three pink abdomen waves'
  if key is None:parts.append((source,{'source_order':order}));continue
  if key not in groups:
   joined={'part':label,'positions':[],'normals':[],'indices':[],'base_color':source['base_color']};metadata={'source_parts':[]};groups[key]=(joined,metadata);parts.append((joined,metadata))
  joined,metadata=groups[key];assert joined['base_color']==source['base_color'];start=len(joined['positions'])//3;first=len(joined['indices'])
  metadata['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])})
  joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGESTS={'kadabra': 'd027eeb4ab22721959745884fec132d2073f9ed1671cb3088292166a42360f58'}

def fit_tail_band(model):
 """Clip the actual tail triangles into a closed, slightly raised saddle band.

 Shared position normals keep the offset shell welded across authored hard
 corners. The tail itself is untouched; the old approximate overlapping tube
 is replaced by this conforming solid. Input is reduced to f32 first so a
 reviewed canonical file and its original JSON use the same geometric input.
 """
 from copy import deepcopy
 model=deepcopy(model);tail=next(p for p in model['primitives'] if p['part']=='Heavy curved psychic tail')
 def f32s(values):return list(struct.unpack('<'+'f'*len(values),packed(values,5126)))
 ps=[tuple(f32s(tail['positions'][i:i+3])) for i in range(0,len(tail['positions']),3)];ns=[tuple(f32s(tail['normals'][i:i+3])) for i in range(0,len(tail['normals']),3)]
 sums={}
 for p,n in zip(ps,ns):
  accum=sums.setdefault(p,[0.,0.,0.])
  for k in range(3):accum[k]+=n[k]
 def unit(n):
  length=math.sqrt(sum(v*v for v in n));return tuple(v/length for v in n)
 shared={p:unit(n) for p,n in sums.items()};lo=normalized('kadabra',(0.,.36,0.))[2];hi=normalized('kadabra',(0.,.26,0.))[2];thickness=.0018
 polygons=[]
 for i in range(0,len(tail['indices']),3):
  poly=[ps[j] for j in tail['indices'][i:i+3]]
  for plane,sign in ((lo,1),(hi,-1)):
   clipped=[]
   for a,b in zip(poly,poly[1:]+poly[:1]):
    inside_a=(a[2]-plane)*sign>=-1e-10;inside_b=(b[2]-plane)*sign>=-1e-10
    if inside_a:clipped.append(a)
    if inside_a!=inside_b:
     t=(plane-a[2])/(b[2]-a[2]);p=tuple(a[k]+t*(b[k]-a[k]) for k in range(3));na=shared[a];nb=shared[b];shared[p]=unit(tuple(na[k]+t*(nb[k]-na[k]) for k in range(3)));clipped.append(p)
   poly=clipped
   if len(poly)<3:break
  if len(poly)>=3:polygons.append(poly)
 # Weld source positions, then offset along the continuous shared field.
 positions=[];normals=[];indices=[];lookup={};edges={}
 def vertex(p):
  key=tuple(round(v,10) for v in p)
  if key not in lookup:
   lookup[key]=len(positions)//3;n=shared[p];positions.extend(p[k]+thickness*n[k] for k in range(3));normals.extend(n)
  return lookup[key]
 for poly in polygons:
  ids=[vertex(p) for p in poly]
  for a,b in zip(ids,ids[1:]+ids[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append((a,b))
  for j in range(1,len(ids)-1):indices += [ids[0],ids[j],ids[j+1]]
 # The two boundary loops are capped inside the tail. Oppose the wall edges.
 boundary={b:a for uses in edges.values() if len(uses)==1 for a,b in uses};loops=[]
 while boundary:
  start=next(iter(boundary));loop=[start];p=boundary.pop(start)
  while p!=start:
   loop.append(p);p=boundary.pop(p)
  loops.append(loop)
 if len(loops)!=2:raise ValueError('Kadabra tail band did not produce two closed rims')
 for loop in loops:
  center=[sum(positions[3*i+k] for i in loop)/len(loop) for k in range(3)];normal=(0.,0.,-1. if sum(positions[3*i+2] for i in loop)/len(loop)<(lo+hi)/2 else 1.)
  cap=[]
  for i in loop:
   cap.append(len(positions)//3);positions.extend(positions[3*i:3*i+3]);normals.extend(normal)
  ci=len(positions)//3;positions.extend(center);normals.extend(normal)
  for a,b in zip(cap,cap[1:]+cap[:1]):indices += [ci,a,b]
 old=next(p for p in model['primitives'] if p['part']=='Broad tail saddle band');old.update(positions=positions,normals=normals,indices=indices)
 return model

def export_kadabra(species='kadabra',model=None):
 if species not in SPECIES:raise ValueError('unknown kadabra species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed kadabra neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite Kadabra continuous neck, planted claws, articulated tail, moustache and held spoon skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS[species]
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':species+'/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':species,'mesh':0,'skin':0})
 g.doc['scenes']=[{'name':species,'nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':species,'skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  # Preserve exact f32 bytes, including signed zero in authored normals.
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 packed_parts=pack_parts(species,model)
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed kadabra geometry')
 for part,metadata in packed_parts:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':species+'/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  ranges=metadata.get('source_parts',[{'part':part['part'],'vertex_start':0,'vertex_count':len(part['positions'])//3}])
  for source in ranges:
   for i in range(source['vertex_start']*3,(source['vertex_start']+source['vertex_count'])*3,3):
    point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(species,source['part'],point);q=quantized_weights(ws)
    ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':species,'primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(species,clip,i/(count-1))[joint])]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':species+'.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed feet and claws; blended neck and limbs; coherent head and fitted features; independent ears, moustache and held spoon; curved tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/build-battle-species.py::sculpt_kadabra;canonical reviewed neutral','geometry_preservation':'original reviewed Kadabra retopology, extended anatomical neck, star, gaze, three pink waves, curled fingers, ivory claws, swept moustache, banded tail and hollow held spoon',
    'animation_contract':'identity floor root; fixed complete feet and claws; blended neck and limbs; coherent head and fitted features; independent ears, wrists, moustache, two tail carriers and held spoon; all logical parts preserved by lossless material packing; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'kadabra exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('kadabra exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('kadabra exceeds existing byte limit')
 return blob


def neutral_model(blob):
 from animated_glb import parse_glb
 if len(blob)>MAX_BYTES:raise ValueError('Kadabra exceeds byte limit')
 try:
  doc,binary=parse_glb(blob)
  if len(doc['meshes'])!=1 or not 1<=len(doc['meshes'][0]['primitives'])<=64 or len(doc['accessors'])>256:raise ValueError('invalid Kadabra dimensions')
  def values(index):
   if type(index) is not int or not 0<=index<len(doc['accessors']):raise ValueError('invalid accessor index')
   a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']];width={'VEC3':3,'SCALAR':1}.get(a['type']);component=a['componentType'];code={5126:'f',5123:'H'}.get(component)
   if width is None or code is None or 'sparse' in a or a.get('byteOffset',0) or view.get('byteStride') or a.get('normalized'):raise ValueError('unsupported neutral accessor')
   count=a['count'];start=view.get('byteOffset',0);size=4 if component==5126 else 2
   if type(count) is not int or not 0<count<=120000 or start<0 or start%4 or view['byteLength']!=count*width*size or start+view['byteLength']>len(binary):raise ValueError('unbounded neutral accessor')
   return list(struct.unpack_from('<'+code*(count*width),binary,start))
  parts=[]
  for p,extra in zip(doc['meshes'][0]['primitives'],doc['extras']['source_part_ranges']):
   positions=values(p['attributes']['POSITION']);normals=values(p['attributes']['NORMAL']);indices=values(p['indices']);color=doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
   sources=extra.get('source_parts',[{'part':p['extras']['part'],'order':extra.get('source_order'),'vertex_start':0,'vertex_count':len(positions)//3,'index_start':0,'index_count':len(indices)}])
   for source in sources:
    a=source['vertex_start'];b=a+source['vertex_count'];i=source['index_start'];j=i+source['index_count']
    if not 0<=a<b<=len(positions)//3 or not 0<=i<j<=len(indices):raise ValueError('unbounded source ranges')
    parts.append((source['order'],{'part':source['part'],'positions':positions[3*a:3*b],'normals':normals[3*a:3*b],'indices':[k-a for k in indices[i:j]],'base_color':color}))
  model={'name':'kadabra','version':1,'coordinate_system':COORDINATES,'primitives':[p for order,p in sorted(parts,key=lambda pair:pair[0])]}
  if tuple(p['part'] for p in model['primitives'])!=PARTS['kadabra'] or geometry_digest(model)!=REST_DIGESTS['kadabra']:raise ValueError('unreviewed Kadabra neutral geometry')
  return model
 except (KeyError,IndexError,TypeError,struct.error,OverflowError,UnicodeError,RecursionError) as error:raise ValueError('malformed Kadabra neutral GLB') from error

def decode_kadabra(blob,species='kadabra'):
 model=neutral_model(blob)
 if species!='kadabra' or blob!=export_kadabra('kadabra',model):raise ValueError('noncanonical Kadabra GLB')
 return model

def read_kadabra(path):
 with Path(path).open('rb') as f:return decode_kadabra(f.read(MAX_BYTES+1))

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--input',type=Path);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 model=None if a.input is None else (read_kadabra(a.input) if a.input.suffix=='.glb' else json.loads(a.input.read_bytes()))
 blob=export_kadabra('kadabra',model);a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_bytes(blob);print('kadabra',len(blob),'bytes')
if __name__=='__main__':main()
