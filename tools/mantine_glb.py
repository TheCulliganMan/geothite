#!/usr/bin/env python3
"""Canonical Mantine with shared wing seams, seated borders and an attached articulated Remora."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from sea_sculpt import sculpture as sea_sculpture,HEIGHTS
HEIGHT=HEIGHTS['mantine']
def sculpture():return sea_sculpture('mantine')
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
MAX_BYTES=1024*1024
SOURCE=sculpture()
PARTS=tuple(p['part'] for p in SOURCE['primitives'])
REFERENCE_SPAN=max(struct.unpack('<f',packed([v]))[0] for p in SOURCE['primitives'] for v in p['positions'][2::3])-min(struct.unpack('<f',packed([v]))[0] for p in SOURCE['primitives'] for v in p['positions'][2::3])
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
REST_DIGEST='712a8adc15f3cc0fde8aab46912528e87185c6ea8c953ec4283a9f00db1dd7b7'
TRIANGLE_DIGEST='5e376dff70947ca933869c3ad0a542b0ca33a0d379d048719cf81f29ace4910e'
RAW=sea_sculpture('mantine',normalize=False)
LOW=min(v for p in RAW['primitives'] for v in p['positions'][1::3])
SCALE=HEIGHT/(max(v for p in RAW['primitives'] for v in p['positions'][1::3])-LOW)
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('body',0,(0.,.35,0.)),
 ('wing_left_base',1,(-.12,.38,-.04)),('wing_left_tip',2,(-.94,.653,.052)),
 ('wing_right_base',1,(.12,.38,-.04)),('wing_right_tip',4,(.94,.653,.052)),
 ('cephalic_left',1,(-.145,.48,.25)),('cephalic_right',1,(.145,.48,.25)),
 ('tail_base',1,(0.,.37,-.28)),('tail_tip',8,(.18,.36,-.65)),
 ('remora',4,(.43,.44,0.)),('remora_tail',10,(.43,.34,-.045)),('remora_tail_fin',11,(.46,.29,-.18)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE)
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Wing /','Wing border /')):
  # Zigzags end at raw x=.94; keep their narrow triangles on the coherent span.
  base=2 if part.endswith('-1') else 4;a=smoothstep(.945,.985,abs(x))
  return [base,base+1,0,0],[1.-a,a,0.,0.]
 if part.startswith('Cephalic fin /'):return rigid(6 if part.endswith('-1') else 7)
 if part.startswith(('Tail leaf /','Tail leaf back /')):return rigid(9)
 if part.startswith('Tail /'):
  a=smoothstep(.47,.71,-z);return [8,9,0,0],[1.-a,a,0.,0.]
 if part.startswith('Remora / slender'):
  a=smoothstep(.11,.18,-z);return [11,12,0,0],[1.-a,a,0.,0.]
 if part.startswith('Remora tail /'):return rigid(12)
 if part.startswith('Remora'):return rigid(10)
 return rigid(1)
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Mantine clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);lift=.5*(1.-math.cos(math.tau*t))
 else:
  e=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  d=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)))
  strength=2.4 if clip=='attack' else -2.;a=e*strength;b=d*strength;lift=e*abs(strength)
 # The resting ray is the lowest pose; lifting wings also lifts the attached fish.
 return [(0.,0.,0.),(0.,.018*a,0.),(0.,.020*a,-.100*lift),(0.,.025*b,-.060*lift),
  (0.,-.020*a,.100*lift),(0.,-.025*b,.060*lift),(.020*a,.040*a,-.020*b),(.020*a,-.040*a,.020*b),
  (0.,.075*a,0.),(0.,.120*b,0.),(0.,.020*lift,0.),(0.,.060*a,0.),(0.,.100*b,0.)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])
def export_mantine(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='mantine' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Mantine anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Mantine shared wing seams and attached Remora skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'mantine/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'mantine','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'mantine','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'mantine','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'mantine/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'mantine','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'mantine.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'coherent two-tone swimming fins and fitted borders; attached Remora; articulated cephalic fins and linked two-layer tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'mantine','source_sha256':REST_DIGEST,
    'reference_span':REFERENCE_SPAN,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/sea_sculpt.py','geometry_preservation':'exact source triangle positions, order and colors; fitted mantle, wing borders and Remora markings share vertices with crease-aware smooth normals',
    'animation_contract':'identity floor root; body yaw preserves contact; coherent marked wing spans with unmarked distal tip flex share exact seams and fitted borders; cephalic paddles; linked two-layer tail; Remora follows proximal right wing with coherent face and articulated tail; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Mantine exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Mantine exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Mantine exceeds existing byte limit')
 return blob


def decode_mantine(blob):
 if len(blob)>MAX_BYTES or blob!=export_mantine():raise ValueError('noncanonical Mantine GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'mantine','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_mantine(path):
 with Path(path).open('rb') as f:return decode_mantine(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_mantine();(a.out/'mantine.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
