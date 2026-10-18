#!/usr/bin/env python3
"""Canonical Cubone and Marowak skins with recessed skulls and genuinely held bone clubs."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
import cubone_sculpt,marowak_sculpt
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('cubone','marowak');HEIGHTS={'cubone':cubone_sculpt.HEIGHT,'marowak':marowak_sculpt.HEIGHT}
def sculpture(species,normalize=True):return (cubone_sculpt if species=='cubone' else marowak_sculpt).sculpture(normalize)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'cubone': '5bd078b289b719850ffbd5621c43b25276736ac2bc919b1e60460c9e84814d77', 'marowak': 'a72e2b8c3dd27fc1f022bc51c29880bc5cf789e205311bfe86d4436d6ef30c0a'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,hi-lo
def normalized(species,point):
 lo,span=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))
def arms(species):
 if species=='cubone':return (((-.264,.5584,.054),(-.346,.4216,.140),(-.311,.3536,.267)),((.264,.5584,.040),(.394,.4648,.081),(.425,.384,.161)))
 return (((-.256,.904,0.),(-.420,1.128,-.140),(-.330,1.530,-.310)),((.267,.901,.020),(.484,1.023,.010),(.664,1.104,.084)))
def joints(species):
 young=species=='cubone'
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.38 if young else .48,0.)),('neck',1,(0.,.72 if young else .96,.04)),('head',2,(0.,.85 if young else 1.12,.06))]
 for side,points in zip(('left','right'),arms(species)):
  base=len(raw);raw += [('arm_'+side,1,points[0]),('elbow_'+side,base,points[1]),('wrist_'+side,base+1,points[2])]
 raw += [('leg_left',0,(-.204,.1944,.014) if young else (-.240,.266,-.005)),('leg_right',0,(.204,.1944,.014) if young else (.240,.266,-.005)),('tail',1,(0.,.2288,-.154) if young else (0.,.340,-.200)),('tail_tip',12,(.01,.1848,-.346) if young else (0.,.275,-.460))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 lo,span=authoring_transform(species);point=tuple(v*span/HEIGHTS[species]+(lo if i==1 else 0.) for i,v in enumerate(position));x,y,z=point;young=species=='cubone'
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Toe claw /')):return rigid(0)
 if part.startswith(('Bone shaft /','Bone lobe /','Thumb /')):return rigid(6)
 if part.startswith(('Head /','Skull /','Snout /','Orbit /','Eye /','Iris /','Pupil /','Eye glint /','Nostril /','Horn /','Cheek /')):return rigid(3)
 if part.startswith('Tail spine /'):return rigid(13)
 if part.startswith('Tail /'):
  a=smoothstep(.14,.29 if young else .40,-z);b=smoothstep(.30 if young else .42,.43 if young else .74,-z)
  return [1,12,13,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith('Haunch /'):
  owner=10 if part.endswith('-1') else 11;a=smoothstep(.09 if young else .12,.28 if young else .40,y);b=.20*math.sin(math.pi*a)**2
  return [0,owner,1,0],[(1.-a)*(1.-b),b,a*(1.-b),0.]
 if part.startswith(('Shoulder /','Arm /','Hand /','Finger /','Hand claw /')):
  side=int(part.split()[-2] if part.startswith('Finger /') else part.split()[-1]);base=4 if side<0 else 7
  if part.startswith(('Hand /','Finger /','Hand claw /')):return rigid(base+2)
  a,b,c=arms(species)[int(side>0)]
  # Continuous longitudinal projection avoids a nearest-segment seam at the bent elbow.
  d=tuple(c[i]-a[i] for i in range(3));t=sum((point[i]-a[i])*d[i] for i in range(3))/sum(v*v for v in d)
  upper=smoothstep(-.08,.18,t);elbow=smoothstep(.25,.65,t);wrist=smoothstep(.74,.94,t)
  return [1,base,base+1,base+2],[1.-upper,upper*(1.-elbow),upper*elbow*(1.-wrist),upper*elbow*wrist]
 if part.startswith('Neck /'):
  a=smoothstep(.70 if young else .94,.79 if young else 1.04,y);b=smoothstep(.78 if young else 1.02,.85 if young else 1.14,y)
  return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
 # Belly and its adult seams share the body's smooth support transition.
 a=smoothstep(.20 if young else .25,.40 if young else .50,y)
 return [0,1,0,0],[1.-a,a,0.,0.]
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid bone keeper clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.018*a,0.),(.010*b,.012*a,0.),(.012*b,.020*a,-.008*b)]
 for side in (-1,1):out += [(-.024*b,side*.018*a,-side*.020*b),(.040*a,side*.022*b,side*.014*a),(.030*b,side*.045*a,side*.020*b)]
 out += [(0.,-.020*a,0.),(0.,.020*a,0.),(.012*b,.045*a,0.),(.015*a,.060*b,0.)]
 return out

def export_bone_keeper(species,model=None):
 if species not in SPECIES:raise ValueError('unknown bone keeper species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed bone keeper neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite Cubone and Marowak articulated skull, planted feet and wrist-held bone skin v1'
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
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':species+'/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(species,part['part'],point);q=quantized_weights(ws)
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed feet; blended neck and bent arms; coherent recessed skull features; wrist-held rigid club; articulated tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/cubone_sculpt.py + tools/marowak_sculpt.py','geometry_preservation':'original rounded bone_keeper and slender marowaktist contours; welded wavy waist; fitted faces and ear linings; scalloped ruff, rooted fingers and open held pendulum',
    'animation_contract':'identity floor root; fixed feet and claws; blended body, neck and leg transitions; rigid skull and fitted eyes; independent upper arms, elbows and wrists; rigid bone travels with gripping wrist; two tail carriers; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'bone keeper exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('bone keeper exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('bone keeper exceeds existing byte limit')
 return blob


def decode_bone_keeper(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_bone_keeper(species):raise ValueError('noncanonical bone keeper GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_bone_keeper(path,species):
 with Path(path).open('rb') as f:return decode_bone_keeper(f.read(MAX_BYTES+1),species)
def export_cubone(model=None):return export_bone_keeper('cubone',model)
def read_cubone(path):return read_bone_keeper(path,'cubone')
def export_marowak(model=None):return export_bone_keeper('marowak',model)
def read_marowak(path):return read_bone_keeper(path,'marowak')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_bone_keeper(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
