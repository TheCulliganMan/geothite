#!/usr/bin/env python3
"""Canonical Electabuzz skin with coherent striped limbs, antennae and a curved tail."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
import electric_pair_sculpt as source
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('electabuzz',);HEIGHTS={'electabuzz':source.HEIGHTS['electabuzz']}
def sculpture(species='electabuzz',normalize=True):return source.sculpture(species,normalize)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'electabuzz': 'b8587f1ad8a7c2bfcf108f28c065448958c95600f35fc98eeda1ec4b3c1da4da'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,hi-lo
def normalized(species,point):
 lo,span=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))
def joints(species='electabuzz'):
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.64,-.014)),('neck',1,(0.,.94,.014)),('head',2,(0.,1.02,.033))]
 for side,s in (('left',-1),('right',1)):
  points,_=source.adult_arm(s);base=len(raw);raw += [('arm_'+side,1,points[0]),('elbow_'+side,base,points[1]),('wrist_'+side,base+1,points[-1])]
 raw += [('leg_left',0,(-.184,.235,.046)),('leg_right',0,(.184,.235,.046)),('antenna_left',3,(-.162,1.145,-.023)),('antenna_right',3,(.162,1.145,-.023)),('crest_left',3,(-.017,1.178,.012)),('crest_right',3,(.017,1.178,.012)),('tail_base',1,source.ADULT_TAIL_POINTS[0]),('tail_mid',16,(.12,.33,-.40)),('tail_tip',17,(.33,.48,-.52))]
 return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for n,parent,p in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
@lru_cache(maxsize=3)
def path_segments(key):
 points=source.ADULT_TAIL_POINTS if key=='tail' else source.adult_arm(-1 if key=='left' else 1)[0]
 path,_=source.smooth(points,[1.]*len(points),12);lengths=[0.]
 for a,b in zip(path,path[1:]):lengths.append(lengths[-1]+math.dist(a,b))
 return [(a,b,lo/lengths[-1],hi/lengths[-1]) for a,b,lo,hi in zip(path,path[1:],lengths,lengths[1:])]
def along_path(point,key):
 best=float('inf');parameter=0.
 for a,b,lo,hi in path_segments(key):
  d=tuple(b[i]-a[i] for i in range(3));t=max(0.,min(1.,sum((point[i]-a[i])*d[i] for i in range(3))/sum(v*v for v in d)));dist=sum((point[i]-a[i]-t*d[i])**2 for i in range(3))
  if dist<best:best=dist;parameter=lo+t*(hi-lo)
 return parameter
def body_weights(y):
 a=smoothstep(.86,.96,y);b=smoothstep(.97,1.03,y)
 return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
def skin_weights(species,part,position):
 lo,span=authoring_transform(species);point=tuple(v*span/HEIGHTS[species]+(lo if i==1 else 0.) for i,v in enumerate(position));x,y,z=point
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Toe claw /')):return rigid(0)
 if part.startswith(('Head /','Eye socket /','Eye /','Pupil /','Forehead /','Jaw /','Mouth back /','Tongue /','Fang /','Nostril /')):return rigid(3)
 if part.startswith(('Antenna /','Antenna ball /')):return rigid(12 if part.endswith('-1') else 13)
 if part.startswith('Crest /'):return rigid(14 if part.endswith('-1') else 15)
 if part.startswith(('Arm /','Arm stripe /','Hand /','Knuckle /','Thumb /','Wrist fur /')):
  side=part.split()[-2] if part.startswith(('Arm stripe /','Knuckle /','Wrist fur /')) else part.split()[-1];base=4 if side=='-1' else 7
  if part.startswith(('Hand /','Knuckle /','Thumb /')):return rigid(base+2)
  t=along_path(point,'left' if side=='-1' else 'right');upper=smoothstep(-.05,.15,t);elbow=smoothstep(.28,.62,t);wrist=smoothstep(.72,.94,t)
  return [1,base,base+1,base+2],[1.-upper,upper*(1.-elbow),upper*elbow*(1.-wrist),upper*elbow*wrist]
 if part.startswith(('Leg /','Leg stripe /')):
  side=part.split()[-2] if part.startswith('Leg stripe /') else part.split()[-1];a=smoothstep(.07,.43,y);b=.28*math.sin(math.pi*a)**2
  return [0,10 if side=='-1' else 11,1,0],[(1.-a)*(1.-b),b,a*(1.-b),0.]
 if part.startswith('Tail tip /'):return rigid(18)
 if part.startswith(('Tail /','Tail stripe /')):
  t=along_path(point,'tail');a=smoothstep(.05,.25,t);b=smoothstep(.30,.60,t);c=smoothstep(.70,.95,t)
  return [1,16,17,18],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
 return body_weights(y)
def pose_angles(species,clip,t):
 if species!='electabuzz' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid electric clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*19
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.014*a,0.),(0.,.012*b,0.),(.008*b,.020*a,0.)]
 for s in (-1,1):out += [(.014*b,s*.018*a,-s*.023*b),(.023*a,s*.020*b,s*.012*a),(.014*b,s*.024*a,s*.014*b)]
 out += [(0.,-.026*a,0.),(0.,.026*a,0.),(.014*b,0.,-.024*b),(.014*b,0.,.024*b),(0.,.012*a,-.020*b),(0.,-.012*a,.020*b),(0.,.030*a,0.),(.018*b,.035*b,0.),(.022*a,.040*b,0.)]
 return out

def pack_parts(species,model):
 parts=[];groups={}
 for order,part in enumerate(model['primitives']):
  name=part['part'];kind=name.split(' /')[0];words=name.split();key=None
  if kind in ('Arm stripe','Leg stripe','Wrist fur','Knuckle','Toe claw'):key=(kind,words[-2])
  elif kind in ('Eye socket','Eye','Pupil','Flank stripe','Antenna','Antenna ball','Crest','Fang','Nostril','Tail stripe','Thumb'):key=(kind,)
  if key is None:parts.append((part,{'source_order':order}));continue
  if key not in groups:
   joined={'part':kind+' / complete electric assembly'+(' '+key[1] if len(key)>1 else ''),'positions':[],'normals':[],'indices':[],'base_color':part['base_color']};meta={'source_parts':[]};groups[key]=(joined,meta);parts.append((joined,meta))
  joined,meta=groups[key];assert joined['base_color']==part['base_color'];start=len(joined['positions'])//3;first=len(joined['indices']);meta['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(part['positions'])//3,'index_start':first,'index_count':len(part['indices'])});joined['positions']+=part['positions'];joined['normals']+=part['normals'];joined['indices'] += [i+start for i in part['indices']]
 return parts
PACKED_REST_DIGESTS={'electabuzz': 'b043a49e3f3885b12adb6401eee1999cb9dcaa956952ca39e300cabc97bc0c34'}
def export_electric(species,model=None):
 if species not in SPECIES:raise ValueError('unknown electric species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed electric neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite striped Electabuzz with anatomical limb and tail skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed electric geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed complete feet and claws; coherent open jaws and fitted markings; articulated asymmetric arms, antennae, scalp tufts and striped tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/electric_pair_sculpt.py','geometry_preservation':'original asymmetric electric feline with open fang-bearing jaws, knuckled fists and thumb folds, scalp tufts, ball-tipped antennae and striped curved limbs/tail; adaptive fitted surfaces and focused round sampling',
    'animation_contract':'identity floor root; fixed feet and toe claws; coherent jaw, fangs, tongue and gaze; stripe wraps share their actual limb and tail fields; independent shoulders, elbows, wrists, supported legs, antennae, crests and tail carriers; lossless packing preserves every source solid; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'electric exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('electric exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('electric exceeds existing byte limit')
 return blob


def decode_electric(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_electric(species):raise ValueError('noncanonical electric GLB')
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


def read_electric(path,species):
 with Path(path).open('rb') as f:return decode_electric(f.read(MAX_BYTES+1),species)
def export_electabuzz(model=None):return export_electric('electabuzz',model)
def decode_electabuzz(blob):return decode_electric(blob,'electabuzz')
def read_electabuzz(path):return read_electric(path,'electabuzz')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_bytes(export_electabuzz())
if __name__=='__main__':main()
