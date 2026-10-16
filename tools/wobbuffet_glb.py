#!/usr/bin/env python3
"""Canonical Wobbuffet skin with planted lobes, elastic salute arms and watchful tail."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from wobbuffet_sculpt import sculpture,HEIGHT
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
REST_DIGEST='4afb5da2de8fcfaaa070c632289d645dc8b67e7790a2f57b19dc3dc3e5b20a09'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
ARMS=(((-.174,.718,-.010),(-.278,.855,-.012),(-.307,1.068,.001),(-.294,1.228,.004),(-.172,1.260,.024),(-.071,1.273,.027)),
      ((.180,.727,0.),(.277,.875,.002),(.327,1.023,0.),(.333,1.183,.004)))
TAIL=((.042,.166,-.058),(.164,.166,-.103),(.341,.199,-.116),(.431,.266,-.124))
BASE=(('root',None,(0.,0.,0.)),('torso',0,(0.,.45,0.)),
 ('salute_shoulder',1,ARMS[0][0]),('salute_elbow',2,ARMS[0][2]),('salute_hand',3,ARMS[0][-1]),
 ('upright_shoulder',1,ARMS[1][0]),('upright_elbow',5,ARMS[1][2]),('upright_hand',6,ARMS[1][-1]),
 ('tail_base',0,TAIL[0]),('tail_bend',8,TAIL[2]),('tail_paddle',9,TAIL[-1]))
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
 if part.startswith('Body /'):
  f=smoothstep(.18,.40,y);return [0,1,0,0],[1.-f,f,0.,0.]
 if part.startswith('Foot /'):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Hand /'):return [4 if 'salute' in part else 7,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Arm /'):
  side=0 if 'salute' in part else 1;a,b,c=(2,3,4) if side==0 else (5,6,7)
  t=path_fraction(raw,ARMS[side]);attach=smoothstep(0.,.18,t);bend=smoothstep(.25,.62,t);hand=smoothstep(.72,.97,t)
  return [1,a,b,c],[1.-attach,attach*(1.-bend),attach*bend*(1.-hand),attach*bend*hand]
 if part.startswith('Tail /'):
  t=path_fraction(raw,TAIL);attach=smoothstep(0.,.20,t);bend=smoothstep(.30,.78,t);paddle=smoothstep(.78,1.,t)
  return [0,8,9,10],[1.-attach,attach*(1.-bend),attach*bend*(1.-paddle),attach*bend*paddle]
 if part.startswith(('Tail paddle /','Tail eye /','Tail pupil /')):return [10,0,0,0],[1.,0.,0.,0.]
 return [1,0,0,0],[1.,0.,0.,0.]
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Wobbuffet clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)))
  strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 return [(0.,0.,0.),(.012*a,.018*a,.014*a),
  (.022*a,.018*a,-.035*a),(.035*b,0.,-.045*b),(.045*b,0.,-.018*b),
  (.025*a,-.018*a,.035*a),(.030*b,0.,.040*b),(.035*b,0.,.020*b),
  (0.,.045*a,0.),(0.,.070*b,.016*b),(0.,.055*b,0.)]
def export_wobbuffet(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='wobbuffet' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Wobbuffet anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Wobbuffet elastic arms and watchful tail skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'wobbuffet/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'wobbuffet','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'wobbuffet','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'wobbuffet','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(tuple(values),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'wobbuffet/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':g.accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'wobbuffet','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'wobbuffet.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted basal lobes; elastic shoulder/elbow/hand chains; coherent face and watchful tail eyes; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'wobbuffet','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/wobbuffet_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; fixed basal lobes; elastic lower body support; face rigid to upper torso; independent arm chains; coherent watchful tail paddle; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Wobbuffet exceeds existing accessor limit')
 return g.bytes()


def decode_wobbuffet(blob):
 if len(blob)>MAX_BYTES or blob!=export_wobbuffet():raise ValueError('noncanonical Wobbuffet GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'wobbuffet','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_wobbuffet(path):
 with Path(path).open('rb') as f:return decode_wobbuffet(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_wobbuffet();(a.out/'wobbuffet.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
