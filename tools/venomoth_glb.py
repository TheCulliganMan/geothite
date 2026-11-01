#!/usr/bin/env python3
"""Canonical swept moth wings, coherent veins, striped abdomen and six tucked toes."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from venomoth_sculpt import sculpture,HEIGHT,CENTER
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
MAX_BYTES=1024*1024
SOURCE=sculpture();RAW=sculpture(normalize=False)
LOW=min(v for p in RAW['primitives'] for v in p['positions'][1::3])
SCALE=HEIGHT/(max(v for p in RAW['primitives'] for v in p['positions'][1::3])-LOW)
PARTS=tuple(p['part'] for p in SOURCE['primitives'])
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
REST_DIGEST='7670a960d16e7c38c3e79a8ffe12ba570ca202b6e77bb25f1d7f21d17dcb5323'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
LEGS=tuple((side,j,(side*(.176+j*.008),.372,z+.055)) for side in (-1,1) for j,z in enumerate((.077,-.020,-.107)))
BASE=[('root',None,(0.,0.,0.)),('thorax',0,CENTER),('head',1,(0.,.651,.055)),('abdomen',1,(0.,.430,-.025)),
 ('wing_left_front',1,(-.065,.485,-.079)),('wing_left_rear',1,(-.065,.449,-.079)),
 ('wing_right_front',1,(.065,.485,-.079)),('wing_right_rear',1,(.065,.449,-.079)),
 ('antenna_center',2,(0.,.748,.070)),('antenna_left',2,(-.048,.748,.069)),('antenna_right',2,(.048,.748,.069))]+[
 ('leg_'+str(side)+'_'+str(j),1,p) for side,j,p in LEGS]
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=[v/SCALE+(LOW if i==1 else 0.) for i,v in enumerate(point)]
 if part.startswith('Foot /'):
  side,j=map(int,part.split()[-2:]);return [11+(0 if side<0 else 3)+j,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Thorax /'):return [1,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Abdomen /','Abdomen band /')):return [3,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Antenna /'):
  owner=8 if 'central' in part else 9 if part.endswith('-1') else 10
  return [owner,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Wing /','Wing vein /')):
  words=part.split();side,rear=map(int,words[-4:-2] if part.startswith('Wing vein /') else words[-2:]);owner=4+(0 if side<0 else 2)+rear
  return [owner,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  side,j=map(int,part.split()[-2:]);owner=11+(0 if side<0 else 3)+j
  a=1.-smoothstep(.39,.44,y)
  return [1,owner,0,0],[1.-a,a,0.,0.]
 return [2,0,0,0],[1.,0.,0.,0.]

def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Venomoth clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);gain=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));gain=2.1 if clip=='attack' else -1.4
 a*=gain;b*=gain
 return [(0.,0.,0.),(0.,.012*a,0.),(.010*a,.020*b,0.),(0.,.025*b,0.),
  (0.,-.16*a,0.),(0.,-.13*b,0.),(0.,.16*a,0.),(0.,.13*b,0.),
  (.018*b,0.,.018*a),(.015*a,0.,-.035*b),(.015*a,0.,.035*b)]+[
  (.013*a*(1 if side<0 else -1),0.,0.) for side,j,p in LEGS]

def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])

def export_venomoth(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='venomoth' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Venomoth anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Venomoth swept veined moth wings, pointed antennae, striped abdomen and six tucked toes skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'venomoth/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'venomoth','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'venomoth','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'venomoth','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'venomoth/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'venomoth','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'venomoth.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'coherent pale gaze and paired mandibles; three antennae; four independent swept wings and fitted veins; striped abdomen; six tucked toes; source cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'venomoth','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/venomoth_sculpt.py','geometry_preservation':'original swept lilac moth with four veined wings, pale compound lenses, three pointed antennae, paired ivory mandibles, striped abdomen and six slender legs',
    'animation_contract':'identity floor root; six tucked toes and articulated legs; coherent eyes and mandibles; four independent wing carriers with matched veins; three antenna carriers; independent striped abdomen; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Venomoth exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Venomoth exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Venomoth exceeds existing byte limit')
 return blob


def decode_venomoth(blob):
 if len(blob)>MAX_BYTES or blob!=export_venomoth():raise ValueError('noncanonical Venomoth GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'venomoth','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_venomoth(path):
 with Path(path).open('rb') as f:return decode_venomoth(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_venomoth();(a.out/'venomoth.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
