#!/usr/bin/env python3
"""Canonical tapirs with continuous bodies, grounded feet and articulated held pendulum."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from tapir_sculpt import sculpture,HEIGHT
SPECIES=('drowzee','hypno');HEIGHTS={s:HEIGHT for s in SPECIES}
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'drowzee': '1789c22bcf0b67308e90a00a00965c00c6e42b8f5bde57c782bd0d930c4e0859', 'hypno': '7e70103e4d8495bf4b78adeb41eca93860c7bde563ec30811f43098380997915'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def joints(species):
 adult=species=='hypno'
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.38,0.)),('head',1,(0.,.72,.015))]
 for side in (-1,1):
  shoulder=(side*.195,.631,-.003) if adult else (side*.221,.618,-.003)
  elbow=(side*.326,.619,.008) if adult else (side*.307,.503,.015)
  wrist=(side*.352,.795,.089) if adult else (side*.347,.468,.120)
  j=len(raw);label='left' if side<0 else 'right'
  raw += [('arm_'+label,1,shoulder),('elbow_'+label,j,elbow),('wrist_'+label,j+1,wrist)]
 raw += [('leg_left',0,(-.121,.292,-.010) if adult else (-.164,.199,-.009)),('leg_right',0,(.121,.292,-.010) if adult else (.164,.199,-.009)),
         ('ear_left',2,(-.135,.950,.030) if adult else (-.207,.964,-.009)),('ear_right',2,(.135,.950,.030) if adult else (.207,.964,-.009))]
 if adult:raw += [('pendulum',8,(.354,.812,.125))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 low,span=authoring_transform(species);x,y,z=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position));adult=species=='hypno'
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Foot /'):return rigid(0)
 if part.startswith('Ear /'):return rigid(11 if part.endswith('-1') else 12)
 if part.startswith('Pendulum /'):return rigid(13)
 if part.startswith('Ruff /'):return rigid(1)
 if part.startswith('Body /'):
  a=smoothstep(.105,.30,y);h=smoothstep(.660,.735,y)
  return [0,1,2,0],[1.-a,a*(1.-h),a*h,0.]
 if part.startswith(('Leg /','Leg joint /')):
  owner=9 if part.endswith('-1') else 10
  if part.startswith('Leg joint /'):return rigid(owner)
  a=smoothstep(.060,.16 if adult else .12,y);b=smoothstep(.245 if adult else .15,.315 if adult else .24,y)
  return [owner,0,1,0],[.01+.99*a*(1.-b),.99*(1.-a),.99*a*b,0.]
 if part.startswith(('Arm /','Arm joint /','Hand /')):
  side=int(part.split()[-2] if part.startswith('Hand / curved finger') else part.split()[-1]);upper=3 if side<0 else 6;elbow=upper+1;hand=upper+2
  if part.startswith('Hand /'):return rigid(hand)
  if part.startswith('Arm joint / rounded elbow'):return rigid(upper)
  if part.startswith(('Arm / shoulder','Arm joint / rounded shoulder')):
   a=smoothstep(.190 if adult else .215,.250 if adult else .275,abs(x));return [1,upper,0,0],[1.-a,a,0.,0.]
  # A continuous projection along each forearm avoids nearest-segment folds.
  a=(.326,.619,.008) if adult else (.307,.503,.015);b=(.352,.795,.089) if adult else (.347,.468,.120)
  d=tuple(b[i]-a[i] for i in range(3));point=(abs(x),y,z);t=sum((point[i]-a[i])*d[i] for i in range(3))/sum(v*v for v in d)
  e=smoothstep(.05,.40,t);w=smoothstep(.72,.95,t)
  return [upper,elbow,hand,0],[(1.-e)*(1.-w),e*(1.-w),w,0.]
 return rigid(2)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid tapir clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.010*a,0.),(.012*a,.015*b,0.)]
 for side in (-1,1):out += [(.015*b,side*.018*b,side*.024*a),(.025*a,side*.015*b,0.),(.018*b,0.,side*.012*a)]
 out += [(0.,-.010*a,-.006*b),(0.,.010*a,.006*b),(0.,-.018*a,-.025*b),(0.,.018*a,.025*b)]
 if species=='hypno':out += [(.028*b,0.,.075*a)]
 return out

def export_tapir(species,model=None):
 if species not in SPECIES:raise ValueError('unknown tapir species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed tapir neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite tapir continuous bodies, planted feet, articulated hands, ears and held pendulum skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed feet; blended bodies and limbs; coherent fitted faces; independent ears and wrists; held pendulum swings from hand; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/tapir_sculpt.py','geometry_preservation':'original rounded tapir and slender hypnotist contours; welded wavy waist; fitted faces and ear linings; scalloped ruff, rooted fingers and open held pendulum',
    'animation_contract':'identity floor root; fixed feet; blended body, neck and leg transitions; coherent head and fitted features; independent ears, upper arms, elbows and wrists; adult pendulum pivots at held cord root; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'tapir exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('tapir exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('tapir exceeds existing byte limit')
 return blob


def decode_tapir(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_tapir(species):raise ValueError('noncanonical tapir GLB')
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


def read_tapir(path,species):
 with Path(path).open('rb') as f:return decode_tapir(f.read(MAX_BYTES+1),species)
def export_drowzee(model=None):return export_tapir('drowzee',model)
def read_drowzee(path):return read_tapir(path,'drowzee')
def export_hypno(model=None):return export_tapir('hypno',model)
def read_hypno(path):return read_tapir(path,'hypno')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_tapir(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
