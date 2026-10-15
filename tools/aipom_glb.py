#!/usr/bin/env python3
"""Canonical Aipom standard skin with an articulated prehensile tail hand."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from aipom_sculpt import sculpture,HEIGHT,tail_hand_point
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('aipom',)
MAX_BYTES=1024*1024

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
REST_DIGESTS={'aipom':'5dfead35f664f46a4a4ae8d8b59a8f22345dae52cf785d5d9a200fed882a0140'}
SOURCE=sculpture()
PARTS={'aipom':tuple(p['part'] for p in SOURCE['primitives'])}
RAW=sculpture(normalize=False)
LOW=min(v for p in RAW['primitives'] for v in p['positions'][1::3])
SCALE=HEIGHT/(max(v for p in RAW['primitives'] for v in p['positions'][1::3])-LOW)
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
TAIL=((0,.295,-.071),(-.065,.287,-.195),(-.24,.330,-.31),(-.39,.49,-.36),(-.445,.73,-.30),(-.475,.952,-.225),(-.48,1.035,-.20))
ARMS={-1:((-.085,.521,.012),(-.185,.459,.023),(-.265,.458,.079)),1:((.085,.521,.012),(.197,.574,.037),(.281,.600,.082))}
BASE_JOINTS=(('root',None,(0.,0.,0.)),('torso',0,(0.,.36,0.)),('neck',1,(0.,.56,.006)),('head',2,(0.,.65,.005)),
 ('ear_left',3,(-.23,.75,-.005)),('ear_right',3,(.23,.75,-.005)),
 ('arm_left',1,ARMS[-1][0]),('hand_left',6,ARMS[-1][-1]),('arm_right',1,ARMS[1][0]),('hand_right',8,ARMS[1][-1]),
 ('tail_base',1,TAIL[0]),('tail_middle',10,TAIL[3]),('tail_tip',11,TAIL[4]),('tail_palm',12,TAIL[-1]))
DIGITS=tuple(('tail_finger_'+str(i),13,tail_hand_point((-.48+.072*math.cos(a),1.055+.069*math.sin(a),-.20))) for i,a in enumerate((.21*math.pi,.70*math.pi,1.20*math.pi)))
TUFTS=tuple(('tuft_'+str(i),3,(x*.4,.918,z)) for i,(x,z) in enumerate(((0,.009),(-.026,.014),(.028,.011))))
JOINTS={'aipom':tuple((n,p,pivot(v) if p is not None else v) for n,p,v in BASE_JOINTS+DIGITS+TUFTS)}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))

def path_fraction(point,path):
 best=(float('inf'),0.)
 lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);travel=0.
 for a,b,length in zip(path,path[1:],lengths):
  d=[y-x for x,y in zip(a,b)]
  t=max(0.,min(1.,sum((p-x)*v for p,x,v in zip(point,a,d))/(length*length)))
  distance=math.dist(point,[x+t*v for x,v in zip(a,d)])
  if distance<best[0]:best=(distance,(travel+t*length)/total)
  travel+=length
 return best[1]

def skin_weights(species,part,position):
 x,y,z=position;raw=(x/SCALE,y/SCALE+LOW,z/SCALE);side=-1 if x<0 else 1
 if part.startswith(('Foot /','Toe fold /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Body /','Belly /','Leg /')):
  upper=smoothstep(.28,.43,raw[1]);return [0,1,0,0],[1.-upper,upper,0.,0.]
 if part.startswith('Neck /'):
  upper=smoothstep(.56,.62,raw[1]);return [1,2,0,0],[1.-upper,upper,0.,0.]
 if part.startswith('Ear /'):return [4 if side<0 else 5,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Arm /'):
  t=path_fraction(raw,ARMS[side]);attach=smoothstep(0.,.25,t);hand=smoothstep(.5,.95,t)
  a,h=(6,7) if side<0 else (8,9)
  return [1,a,h,0],[1.-attach,attach*(1.-hand),attach*hand,0.]
 if part.startswith('Hand /'):return [7 if side<0 else 9,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail /'):
  t=path_fraction(raw,TAIL)
  if t<.12:
   f=smoothstep(0.,.12,t);return [1,10,0,0],[1.-f,f,0.,0.]
  if t<.60:
   f=smoothstep(.25,.60,t);return [10,11,0,0],[1.-f,f,0.,0.]
  if t<.85:
   f=smoothstep(.62,.85,t);return [11,12,0,0],[1.-f,f,0.,0.]
  f=smoothstep(.88,1.,t);return [12,13,0,0],[1.-f,f,0.,0.]
 if part.startswith('Tail hand /'):return [13,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail fingertip /'):return [14+int(part.rsplit(' ',1)[1]),0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail finger /'):
  raw=tail_hand_point(raw,inverse=True)
  i=int(part.rsplit(' ',1)[1]);f=smoothstep(.075,.15,math.hypot(raw[0]+.48,raw[1]-1.055))
  return [13,14+i,0,0],[1.-f,f,0.,0.]
 if part.startswith('Tuft /'):return [17+int(part.rsplit(' ',1)[1]),0,0,0],[1.,0.,0.,0.]
 return [3,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species!='aipom' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Aipom clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*20
 w=math.sin(math.tau*t);b=.5-.5*math.cos(math.tau*t)
 def lag(phase):return math.sin(math.tau*t-phase)+math.sin(phase)
 if clip=='idle':
  brace=.010*b;head=.012*lag(.3);arm=.025*w;tail=.018*w;response=.026*lag(.55);finger=.023*b
 else:
  early=envelope(t,((0.,0.),(.18,1.),(.43,.65),(.74,.08),(1.,0.)))
  late=envelope(t,((0.,0.),(.31,1.),(.56,.55),(.81,-.08),(1.,0.)))
  sign=1. if clip=='attack' else -.8
  brace=sign*.025*early;head=sign*.030*late;arm=sign*.07*early;tail=sign*.025*early;response=sign*.040*late;finger=sign*.045*late
 return [(0.,0.,0.),(brace,.005*w,0.),(head*.4,0.,0.),(head,.014*w,0.),
         (0.,.018*w,-.018*b),(0.,-.018*w,.018*b),
         (arm,0.,-.015*b),(.025*lag(.3),.018*w,0.),(-arm,0.,.015*b),(-.025*lag(.3),-.018*w,0.),
         (tail,0.,tail*.5),(response,.009*w,0.),(response*.8,0.,-.012*lag(.4)),(response,.018*lag(.6),.012*w),
         (finger,0.,-.01*b),(finger,0.,.01*b),(finger,0.,0.),
         (.018*lag(.3),0.,.008*w),(.016*lag(.5),0.,-.008*w),(.014*lag(.7),0.,.008*w)]

def joint_rotation(joint,angles):
 q=quaternion_xyz(*angles)
 # Conjugate hand rotations by the fixed rest turn: preserve local curl axes.
 if 13<=joint<=16:return (*tail_hand_point(q[:3],vector=True),q[3])
 return q


def export_aipom(model=None):
 species='aipom'
 if model is None:model=SOURCE
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed prehensile monkey neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct prehensile monkey anatomy and standard skin v1'
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
  key=(tuple(values),kind,component,target,normalized)
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
    'NORMAL':g.accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':species,'primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in joint_rotation(joint,pose_angles(species,clip,i/(count-1))[joint])]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':species+'.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'body brace; delayed prehensile tail hand and fingers; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/aipom_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; fixed supporting feet; fitted face; articulated head ears arms tail hand fingers and crown tufts; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('prehensile monkey exceeds existing accessor limit')
 return g.bytes()


def decode_aipom(blob):
 species='aipom'
 if len(blob)>MAX_BYTES or blob!=export_aipom():raise ValueError('noncanonical prehensile monkey GLB')
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


def read_aipom(path):
 with Path(path).open('rb') as f:return decode_aipom(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--only',choices=SPECIES);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 for n in ([a.only] if a.only else SPECIES):
  blob=export_aipom();(a.out/(n+'.glb')).write_bytes(blob);print(n,len(blob),'bytes',len(PARTS[n]),'parts',len(JOINTS[n]),'joints',REST_DIGESTS[n])
if __name__=='__main__':main()
