#!/usr/bin/env python3
"""Canonical Hitmontop skin with a fixed spin point, coherent helmet and raised kicking limbs."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from hitmontop_sculpt import sculpture,HEIGHT
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
REST_DIGEST='554e1edaafe9860ab6aa4b5c306a67549b3f8e7c2773e3adefad4901867a3de3'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('helmet',0,(0.,1.37,-.027)),('neck',1,(0.,.95,.03)),('torso',2,(0.,.84,.025)),
 ('arm_left',3,(-.148,.759,.029)),('arm_right',3,(.148,.759,.029)),
 ('leg_left',3,(-.127,.578,.018)),('leg_right',3,(.127,.578,.018)),
 ('tail_base',3,(0.,.563,-.099)),('tail_bend',8,(.076,.409,-.312)),('tail_ball',9,(.087,.171,-.579)))
# The sculpture rotates the upright authored cage 180 degrees around Z.
JOINTS=tuple((n,parent,pivot((-p[0],-p[1],p[2])) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE);u=-y
 if part.startswith('Spin point /'):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Head /','Eye /','Pupil /','Helmet crease /','Mouth /')):return [1,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Arm /','Wrist wrap /','Hand /')):return [4 if part.endswith('-1') else 5,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Leg /','Ankle wrap /','Foot /','Foot spike /','Foot sole /')):
  side=part.split()[-2] if part.startswith('Foot spike /') else part.split()[-1]
  return [6 if side=='-1' else 7,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Tail ball /','Tail spike /')):return [10,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail /'):
  a=smoothstep(.12,.23,-z);b=smoothstep(.25,.40,-z);c=smoothstep(.43,.55,-z)
  return [3,8,9,10],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
 a=1.-smoothstep(.90,.98,u);b=1.-smoothstep(.80,.87,u)
 return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Hitmontop clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 # Slow near the neutral boundary without reversing the complete idle turn.
 # Literal identity endpoints use shortest-arc slerp across the +/-q seam.
 return [(0.,0.,0.),(0.,math.tau*t-.75*math.sin(math.tau*t)+.025*a if clip=='idle' else .025*a,0.),(0.,.015*a,0.),(.010*a,.020*a,0.),
  (.020*b,.020*a,-.025*b),(.020*b,-.020*a,.025*b),
  (.020*a,.030*b,-.025*a),(.020*a,-.030*b,.025*a),
  (0.,.025*a,0.),(.020*b,.030*b,0.),(.020*b,.025*b,0.)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])
def export_hitmontop(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='hitmontop' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Hitmontop anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Hitmontop continuous idle spin with fixed balance point skin v2'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'hitmontop/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'hitmontop','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'hitmontop','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'hitmontop','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'hitmontop/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'hitmontop','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'hitmontop.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed spin point; continuous complete idle turn; coherent helmet and fitted gaze; shared belly/neck field; raised wrapped limbs; delayed tail ball; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'hitmontop','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/hitmontop_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; fixed spin point; full-turn idle helmet with fitted gaze and shortest-arc quaternion wrap; shared body/belly/neck field; coherent raised legs with ankle wraps, spiked feet and fitted soles; moving arms with cuffs and fists; tail ball and spike share terminal joint; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Hitmontop exceeds existing accessor limit')
 return g.bytes()


def decode_hitmontop(blob):
 if len(blob)>MAX_BYTES or blob!=export_hitmontop():raise ValueError('noncanonical Hitmontop GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'hitmontop','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_hitmontop(path):
 with Path(path).open('rb') as f:return decode_hitmontop(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_hitmontop();(a.out/'hitmontop.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
