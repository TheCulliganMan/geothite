#!/usr/bin/env python3
"""Canonical dual-layer star with ten radial arm chains and a coherent gemstone mount."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from starmie_sculpt import sculpture,HEIGHT,CENTER,RADIUS,FRONT_ANGLES,BACK_ANGLES
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
REST_DIGEST='d9e6623d83cb207c09a01879bc85d5124e8a3a87fcdb6bc93ca79084369a6897'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=[('root',None,(0.,0.,0.)),('core',0,(0.,.40,.10)),('front_layer',1,CENTER)]
for i,angle in enumerate(FRONT_ANGLES):
 base=3+2*i
 BASE += [('front_arm_'+str(i)+'_base',2,(.145*math.cos(angle),.40+.145*math.sin(angle),0.)),('front_arm_'+str(i)+'_tip',base,(.305*math.cos(angle),.40+.305*math.sin(angle),0.))]
BASE.append(('back_layer',1,(0.,.40,-.10)))
for i,angle in enumerate(BACK_ANGLES):
 base=14+2*i
 BASE += [('back_arm_'+str(i)+'_base',13,(.145*math.cos(angle),.40+.145*math.sin(angle),-.10)),('back_arm_'+str(i)+'_tip',base,(.305*math.cos(angle),.40+.305*math.sin(angle),-.10))]
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 if not part.startswith(('Front star /','Front ridge /','Back star /','Back ridge /')):return [1,0,0,0],[1.,0.,0.,0.]
 x=point[0]/SCALE;y=point[1]/SCALE+LOW-.40;radius=math.hypot(x,y)/RADIUS;theta=math.atan2(y,x)
 rear=part.startswith('Back ');angles=BACK_ANGLES if rear else FRONT_ANGLES;layer=13 if rear else 2
 i=min(range(5),key=lambda j:abs(math.atan2(math.sin(theta-angles[j]),math.cos(theta-angles[j]))))
 base=(14 if rear else 3)+2*i;tip=base+1;a=smoothstep(.27,.55,radius);b=smoothstep(.58,.86,radius)
 js=[layer,base,tip,0];ws=[1.-a,a*(1.-b),a*b,0.]
 if rear and i==2:
  pin=smoothstep(.58,.88,radius);ws=[(1.-pin)*w for w in ws];ws[3]=pin
 return js,ws

def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Starmie clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.35)+math.sin(.35);early=.018*a;late=.027*b;turn=.006*a;twist=.012*b
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));gain=1. if clip=='attack' else -.65
  early=.060*a*gain;late=.085*b*gain;turn=.012*a*gain;twist=.026*b*gain
 poses=[(0.,0.,0.),(0.,turn,0.),(0.,.006*a,.55*twist)]
 for i,angle in enumerate(FRONT_ANGLES):
  for bend in (early,late):poses.append((-math.sin(angle)*bend,math.cos(angle)*bend,0.))
 poses.append((0.,-.008*a,-twist))
 for i,angle in enumerate(BACK_ANGLES):
  gain=.20 if i==2 else .90
  for bend in (early,late):poses.append((math.sin(angle)*bend*gain,-math.cos(angle)*bend*gain,0.))
 return poses
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])

def export_starmie(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='starmie' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Starmie anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Starmie dual purple star, ten radial arm chains and eight-spoke core skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'starmie/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'starmie','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'starmie','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'starmie','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'starmie/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'starmie','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'starmie.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'two offset star layers; ten blended radial arm chains; coherent octagonal rose core and eight gold rays; protected lowest rear tip; source cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'starmie','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/starmie_sculpt.py','geometry_preservation':'original continuous dual purple stars, ten seated radial ridges, eight-spoke octagonal golden mount and faceted rose gemstone',
    'animation_contract':'identity authoring root; standard render-only hovering clearance; coherent central gemstone mount; independent offset star layers and ten smoothly weighted radial arm chains; protected lowest rear tip; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Starmie exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Starmie exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Starmie exceeds existing byte limit')
 return blob


def decode_starmie(blob):
 if len(blob)>MAX_BYTES or blob!=export_starmie():raise ValueError('noncanonical Starmie GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'starmie','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_starmie(path):
 with Path(path).open('rb') as f:return decode_starmie(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_starmie();(a.out/'starmie.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
