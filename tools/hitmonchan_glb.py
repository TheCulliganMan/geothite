#!/usr/bin/env python3
"""Canonical Hitmonchan skin with planted boots, fitted crown gaze and articulated boxing gloves."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from hitmonchan_sculpt import sculpture,HEIGHT
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
REST_DIGEST='59c5d015861346b850e7aca45fa3056246a8d8bc950ce5d9793a0035d7856daf'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('torso',0,(0.,.85,.015)),('neck',1,(0.,1.06,.06)),('head',2,(0.,1.12,.08)),
 ('shoulder_left',1,(-.19,1.08,.02)),('shoulder_right',1,(.19,1.08,.02)),
 ('arm_left',4,(-.210,1.069,.023)),('wrist_left',6,(-.374,1.033,.157)),
 ('arm_right',5,(.209,1.063,.028)),('wrist_right',8,(.396,.956,.244)),
 ('leg_left',0,(-.171,.319,.043)),('leg_right',0,(.171,.319,.043)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE)
 if part.startswith(('Boot /','Boot cuff /','Boot seam /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  a=smoothstep(.16,.53,y);g=.25*smoothstep(.15,.23,y)*(1.-smoothstep(.43,.56,y))
  return [0,1,10 if part.endswith('-1') else 11,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith('Shoulder guard /'):return [4 if part.endswith('-1') else 5,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Arm /'):
  a=smoothstep(.05,.31 if part.endswith('-1') else .36,z);owner=6 if part.endswith('-1') else 8
  return [owner,owner+1,0,0],[1.-a,a,0.,0.]
 if part.startswith(('Glove cuff /','Glove /','Glove thumb /','Glove seam /')):return [7 if part.endswith('-1') else 9,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Head /','Eye socket /','Eye /','Iris /','Pupil /','Glint /','Mouth /')):return [3,0,0,0],[1.,0.,0.,0.]
 a=smoothstep(1.03,1.10,y);b=smoothstep(1.10,1.17,y)
 return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Hitmonchan clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength;c=a*(.20 if clip=='idle' else 1.);d=b*(.20 if clip=='idle' else 1.)
 return [(0.,0.,0.),(0.,.012*a,0.),(0.,.015*a,0.),(.006*a,.020*a,0.),
  (0.,.010*b,-.015*b),(0.,-.010*b,.015*b),
  (.030*c,.040*d,-.025*c),(.120*c,.060*d,-.040*c),
  (.025*c,-.035*d,.025*c),(.100*c,-.040*d,.040*c),
  (0.,.018*a,0.),(0.,-.018*a,0.)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])
def export_hitmonchan(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='hitmonchan' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Hitmonchan anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Hitmonchan planted boxer with articulated gloves skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'hitmonchan/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'hitmonchan','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'hitmonchan','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'hitmonchan','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'hitmonchan/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'hitmonchan','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'hitmonchan.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted boots and ankle cuffs; fitted crown gaze; articulated shoulder guards and bent arms; coherent glove cuffs, mitts, thumbs and seams; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'hitmonchan','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/hitmonchan_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; fixed boots, ankle cuffs and piping; shared neck field; rigid fitted crown gaze; articulated shoulder guards, bent arms and wrists; coherent glove cuffs, mitts, thumbs and seams; supporting leg middles; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Hitmonchan exceeds existing accessor limit')
 return g.bytes()


def decode_hitmonchan(blob):
 if len(blob)>MAX_BYTES or blob!=export_hitmonchan():raise ValueError('noncanonical Hitmonchan GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'hitmonchan','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_hitmonchan(path):
 with Path(path).open('rb') as f:return decode_hitmonchan(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_hitmonchan();(a.out/'hitmonchan.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
