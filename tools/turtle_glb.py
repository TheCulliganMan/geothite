#!/usr/bin/env python3
"""Canonical turtle skins with coherent rigid shells, feather ears and genuinely hollow cannons."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
import wartortle_sculpt,blastoise_sculpt
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('wartortle','blastoise');HEIGHTS={'wartortle':wartortle_sculpt.HEIGHT,'blastoise':blastoise_sculpt.HEIGHT}
def sculpture(species,normalize=True):return (wartortle_sculpt if species=='wartortle' else blastoise_sculpt).sculpture(species,normalize)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'wartortle': '880d8205d6099ec0b0256c00cc20d8c590ff2b7e0b3281df595634bf49e52d6e', 'blastoise': '894507c9ac4c53dad84866af1e44e99f44f2bef9452dd732ee12e9e23a93ed69'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,hi-lo
def normalized(species,point):
 lo,span=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))
def arms(species):
 adult=species=='blastoise'
 return tuple(((s*.25,.652,.024),(s*.405,.589,.093),(s*.477,.516,.203)) if adult else ((s*.18,.601,.026),(s*.30,.552,.089),(s*.385,.537,.136)) for s in (-1,1))
def joints(species):
 adult=species=='blastoise';raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.455 if adult else .44,-.02)),('head',1,(0.,.76 if adult else .72,.06))]
 for side,points in zip(('left','right'),arms(species)):
  base=len(raw);raw += [('arm_'+side,1,points[0]),('elbow_'+side,base,points[1]),('wrist_'+side,base+1,points[2])]
 raw += [('leg_left',0,(-.235,.235,.044) if adult else (-.171,.184,.041)),('leg_right',0,(.235,.235,.044) if adult else (.171,.184,.041))]
 if adult:raw += [('barrel_left',1,(-.265,.744,-.082)),('barrel_right',1,(.265,.744,-.082))]
 else:raw += [('feather_left',2,(-.132,.917,.068)),('feather_right',2,(.132,.917,.068))]
 raw += [('tail_base',1,(0.,.265,-.215) if adult else (-.02,.30,-.17)),('tail_tip',13,(0.,.18,-.35) if adult else (.25,.58,-.35))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 lo,span=authoring_transform(species);point=tuple(v*span/HEIGHTS[species]+(lo if i==1 else 0.) for i,v in enumerate(position));x,y,z=point;adult=species=='blastoise'
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Toe /','Foot claw /')):return rigid(0)
 if part.startswith(('Cannon /','Cannon floor /','Cannon lining /')):return rigid(11 if part.endswith('-1') else 12)
 if part.startswith(('Ear /','Ear lobe /','Ear inset /')):
  if adult:return rigid(2)
  a=smoothstep(.91,1.02,y);return [2,11 if part.endswith('-1') else 12,0,0],[1.-a,a,0.,0.]
 if part.startswith(('Head /','Muzzle /','Jaw /','Eye socket /','Eye /','Iris /','Pupil /','Glint /','Mouth /','Nostril /','Fang /')):return rigid(2)
 if part.startswith('Hand claw /'):
  side=int(part.split()[-2]);return rigid(5 if side<0 else 8)
 if part.startswith('Arm /'):
  side=int(part.split()[-1]);base=3 if side<0 else 6;a,b,c=arms(species)[int(side>0)];d=tuple(c[i]-a[i] for i in range(3));t=sum((point[i]-a[i])*d[i] for i in range(3))/sum(v*v for v in d)
  upper=smoothstep(-.10,.18,t);elbow=smoothstep(.30,.65,t);wrist=smoothstep(.68,.88,t)
  return [1,base,base+1,base+2],[1.-upper,upper*(1.-elbow),upper*elbow*(1.-wrist),upper*elbow*wrist]
 if part.startswith(('Haunch /','Leg /')):
  side=int(part.split()[-1]);a=smoothstep(.07,.26 if adult else .21,y);b=.22*math.sin(math.pi*a)**2
  return [0,9 if side<0 else 10,1,0],[(1.-a)*(1.-b),b,a*(1.-b),0.]
 if part.startswith(('Tail /','Tail fold /')):
  if adult:a=smoothstep(.22,.32,-z);b=smoothstep(.34,.44,-z)
  else:a=smoothstep(.28,.46,y);b=smoothstep(.52,.76,y)
  return [1,13,14,0],[1.-a,a*(1.-b),a*b,0.]
 return rigid(1)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid turtle clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*15
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.016*a,0.),(.012*b,.020*a,-.006*b)]
 for side in (-1,1):out += [(-.022*b,side*.022*a,-side*.018*b),(.035*a,side*.020*b,side*.012*a),(.022*b,side*.028*a,side*.016*b)]
 out += [(0.,-.024*a,0.),(0.,.024*a,0.)]
 if species=='blastoise':out += [(-.018*b,-.010*a,0.),(-.018*b,.010*a,0.)]
 else:out += [(.020*b,0.,-.030*a),(.020*b,0.,.030*a)]
 # The broad cloud fan bends in its own plane. Out-of-plane yaw across its
 # thin rim can fold the stored surface despite small joint angles.
 out += [(.008*b,.036*a,0.),(.012*a,.045*b,0.)] if species=='blastoise' else [(0.,0.,.024*a),(0.,0.,.028*b)]
 return out

def pack_parts(species,model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name=source['part'];key=None;label=None;words=name.split()
  if name.startswith(('Shell seam /','Belly seam /','Tail fold /')):
   kind=name.split(' /')[0];key=(kind,);label=kind+' / complete fitted lines'
  elif name.startswith(('Hand claw /','Foot claw /','Toe /')):
   kind=name.split(' /')[0];side=words[-2];key=(kind,side);label=kind+' / complete three digits '+side
  elif name.startswith(('Eye socket /','Eye /','Iris /','Pupil /','Glint /','Nostril /')):
   kind=name.split(' /')[0];key=(kind,);label=kind+' / paired turtle gaze'
  if key is None:parts.append((source,{'source_order':order}));continue
  if key not in groups:
   joined={'part':label,'positions':[],'normals':[],'indices':[],'base_color':source['base_color']};metadata={'source_parts':[]};groups[key]=(joined,metadata);parts.append((joined,metadata))
  joined,metadata=groups[key];assert joined['base_color']==source['base_color'];start=len(joined['positions'])//3;first=len(joined['indices'])
  metadata['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])})
  joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGESTS={'wartortle': '8f76359639251ae9161bf295201020e8e4bafafe252c9f286fd3812218eb0a58', 'blastoise': '0e4dfa76fc3b7fd8a1a2360552fa7c2663e821fd88a442bb899b32dd6e859548'}

def export_turtle(species,model=None):
 if species not in SPECIES:raise ValueError('unknown turtle species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed turtle neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite turtle shell, feather ears and hollow cannon skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed turtle geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed feet and matching toe claws; coherent rigid shell, fitted scutes and breast; articulated head, arms, wrists, tail and feather ears or hollow cannons; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/'+species+'_sculpt.py','geometry_preservation':'original feather-eared cloud-tailed turtle and heavy armored cannon turtle; fitted gaze, shell scutes and belly lines; genuine recessed cannon voids; continuous closed forelimb and palm surfaces replace intersecting wrist solids',
    'animation_contract':'identity floor root; fixed feet and matching claws; rigid shell, breast and fitted seams; coherent skull and gaze; independent upper arms, elbows, wrists, thigh support and tail carriers; feather ears or rigid hollow cannon assemblies follow their true owners; lossless packing preserves every source solid; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'turtle exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('turtle exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('turtle exceeds existing byte limit')
 return blob


def decode_turtle(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_turtle(species):raise ValueError('noncanonical turtle GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 parts=[]
 for p,extra in zip(doc['meshes'][0]['primitives'],doc['extras']['source_part_ranges']):
  positions=values(p['attributes']['POSITION']);normals=values(p['attributes']['NORMAL']);indices=values(p['indices']);color=doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
  sources=extra.get('source_parts',[{'part':p['extras']['part'],'order':extra.get('source_order'),'vertex_start':0,'vertex_count':len(positions)//3,'index_start':0,'index_count':len(indices)}])
  for source in sources:
   a=source['vertex_start'];b=a+source['vertex_count'];i=source['index_start'];j=i+source['index_count']
   parts.append((source['order'],{'part':source['part'],'positions':positions[3*a:3*b],'normals':normals[3*a:3*b],'indices':[k-a for k in indices[i:j]],'base_color':color}))
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':[p for order,p in sorted(parts,key=lambda pair:pair[0])]}


def read_turtle(path,species):
 with Path(path).open('rb') as f:return decode_turtle(f.read(MAX_BYTES+1),species)
def export_wartortle(model=None):return export_turtle('wartortle',model)
def read_wartortle(path):return read_turtle(path,'wartortle')
def export_blastoise(model=None):return export_turtle('blastoise',model)
def read_blastoise(path):return read_turtle(path,'blastoise')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_turtle(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
