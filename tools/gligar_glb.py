#!/usr/bin/env python3
"""Canonical Gligar skin with planted talons, coupled sails/pincers and articulated venom tail."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from gligar_sculpt import sculpture,HEIGHT
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
REST_DIGEST='9d3742959ab097a791488894bd78c071e6ace9c2dcb6c41b7dbd7e81ced1c10d'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
ARMS=(((-.123,.513,.008),(-.264,.578,.016),(-.409,.649,.048)),((.123,.505,.006),(.230,.551,.024),(.350,.618,.049)))
TAIL=((0.,.226,-.079),(-.024,.159,-.290),(-.229,.150,-.417),(-.403,.223,-.320),(-.479,.333,-.133))
BASE=(('root',None,(0.,0.,0.)),('torso',0,(0.,.334,0.)),('head',1,(0.,.685,.022)),
 ('ear_left',2,(-.175,.792,.012)),('ear_right',2,(.175,.792,.012)),('tongue',2,(0.,.59,.186)),
 ('arm_left',1,ARMS[0][0]),('pincer_left',6,ARMS[0][-1]),('arm_right',1,ARMS[1][0]),('pincer_right',8,ARMS[1][-1]),
 ('tail_base',1,TAIL[0]),('tail_middle',10,TAIL[2]),('tail_tip',11,TAIL[-1]),('venom_bulb',12,(-.486,.354,-.102)),('stinger',13,(-.526,.393,-.075)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def path_fraction(point,path):
 best=(math.inf,0.);lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.
 for a,b,length in zip(path,path[1:],lengths):
  delta=[y-x for x,y in zip(a,b)];t=max(0.,min(1.,sum((p-x)*d for p,x,d in zip(point,a,delta))/(length*length)))
  distance=math.dist(point,[x+t*d for x,d in zip(a,delta)])
  best=min(best,(distance,(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(part,point):
 raw=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE);x,y,z=raw
 if part.startswith(('Body /','Leg /')):
  f=smoothstep(.13,.22,y);return [0,1,0,0],[1.-f,f,0.,0.]
 if part.startswith(('Foot /','Foot claw /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Ear /'):return [3 if part.endswith('-1') else 4,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Tongue /','Tongue crease /')):return [5,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Pincer /'):return [7 if part.endswith('-1') else 9,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Arm /','Wing /','Wing spar /')):
  left=part.endswith('-1');arm,claw=(6,7) if left else (8,9)
  attach=smoothstep(.12,.23,abs(x));tip=smoothstep(.24,.40,abs(x)) if left else smoothstep(.23,.35,abs(x))
  return [1,arm,claw,0],[1.-attach,attach*(1.-tip),attach*tip,0.]
 if part.startswith('Tail /'):
  t=path_fraction(raw,TAIL);stops=(0.,.20,.48,.78,1.);owners=(1,10,11,12,13)
  i=min(3,next((i for i in range(4) if t<=stops[i+1]),3));f=smoothstep(stops[i],stops[i+1],t)
  return [owners[i],owners[i+1],0,0],[1.-f,f,0.,0.]
 if part.startswith('Tail bulb /'):return [13,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Stinger /'):return [14,0,0,0],[1.,0.,0.,0.]
 return [2,0,0,0],[1.,0.,0.,0.]
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Gligar clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)))
  strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 return [(0.,0.,0.),(.008*a,.012*a,.009*a),(.008*b,.012*b,0.),
  (0.,0.,-.015*b),(0.,0.,.015*b),(.015*b,.025*b,0.),
  (0.,0.,-.020*a),(0.,0.,-.025*b),(0.,0.,.020*a),(0.,0.,.025*b),
  (0.,.025*a,.010*a),(.015*b,.035*b,0.),(.020*b,.040*b,0.),(.015*b,.030*b,0.),(.010*b,.020*b,0.)]
def export_gligar(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='gligar' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Gligar anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Gligar coupled sail/pincer and venom tail skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'gligar/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'gligar','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'gligar','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'gligar','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(tuple(values),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'gligar/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':g.accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'gligar','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'gligar.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted feet/talons; coupled arms sails and spars; coherent crescent pincers; head ears tongue and articulated venom tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'gligar','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/gligar_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; fixed feet and talons; fitted face follows head; tongue/crease share owner; arm/sail/spar weights agree; rigid open pincers; articulated tail bulb and stinger; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Gligar exceeds existing accessor limit')
 return g.bytes()


def decode_gligar(blob):
 if len(blob)>MAX_BYTES or blob!=export_gligar():raise ValueError('noncanonical Gligar GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'gligar','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_gligar(path):
 with Path(path).open('rb') as f:return decode_gligar(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_gligar();(a.out/'gligar.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
