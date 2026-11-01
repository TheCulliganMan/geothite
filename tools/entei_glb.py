#!/usr/bin/env python3
"""Entei standard skin with a coherent armored head, planted paws, mane and smoke trails."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from entei_sculpt import sculpture as sculpt,HEIGHT,TAIL_POINTS
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

SPECIES=('entei',)
REST_DIGESTS={'entei':'db453e0d91aa89189660d76f63e17f62e26c4d80e750455f9fb7e066cbcc249c'}
PARTS={'entei':tuple(p['part'] for p in sculpt()['primitives'])};MAX_BYTES=1024*1024

def sculpture(species):
 if species!='entei':raise ValueError('unsupported volcano_beast')
 return sculpt()
@lru_cache(maxsize=1)
def authoring_transform(species):
 raw=sculpt(normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(point):
 low,span=authoring_transform('entei');return tuple((v-(low if i==1 else 0.))*HEIGHT/span for i,v in enumerate(point))
JOINTS={'entei':(('root',None,(0.,0.,0.)),('torso',0,normalized((0.,.59,-.17))),
 ('neck',1,normalized((0.,.93,.16))),('head',2,normalized((0.,1.20,.34))),
 ('mane_left',2,normalized((-.20,.98,.20))),('mane_right',2,normalized((.20,.98,.20))),
 ('smoke_base',1,normalized((0.,1.12,-.10))),('smoke_tip',6,normalized((0.,1.02,-.42))),
 ('blade_left',1,normalized((-.19,.79,-.32))),('blade_right',1,normalized((.19,.79,-.32))),
 ('fore_left',1,normalized((-.235,.57,.25))),('fore_right',1,normalized((.235,.57,.25))),
 ('rear_left',1,normalized((-.235,.55,-.49))),('rear_right',1,normalized((.235,.55,-.49))),
 ('tail_base',1,normalized(TAIL_POINTS[0])),('tail_tip',14,normalized(TAIL_POINTS[2])))}

CLIPS=(('idle',4.,85),('attack',1.,65),('hit',1.,65))


def skin_weights(species,part,position):
 if part.startswith(('Foot /','Toe /','Cuff /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  owner=(12 if 'rear' in part else 10)+(1 if part.endswith('1') and not part.endswith('-1') else 0)
  flex=smoothstep(normalized((0.,.15,0.))[1],normalized((0.,.48,0.))[1],position[1]);return [0,owner,0,0],[1.-flex,flex,0.,0.]
 if part.startswith('Mane /'):return [4 if part.endswith('-1') else 5,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Back blade /'):return [8 if ' -1 ' in part else 9,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Smoke /'):
  # smoothstep expects ascending endpoints; rearward distance has opposite sign.
  blend=smoothstep(-normalized((0.,0.,-.20))[2],-normalized((0.,0.,-.55))[2],-position[2]);return [6,7,0,0],[1.-blend,blend,0.,0.]
 if part.startswith(('Tail /','Tail cloud /')):
  low,span=authoring_transform(species);raw=[position[k]*span/HEIGHT+(low if k==1 else 0.) for k in range(3)];best=float('inf');parameter=0.
  for j,(a,b) in enumerate(zip(TAIL_POINTS,TAIL_POINTS[1:])):
   delta=[b[k]-a[k] for k in range(3)];length=sum(v*v for v in delta);t=max(0.,min(1.,sum((raw[k]-a[k])*delta[k] for k in range(3))/length))
   distance=sum((raw[k]-a[k]-delta[k]*t)**2 for k in range(3))
   if distance<best:best=distance;parameter=j+t
  blend=smoothstep(.8,2.4,parameter);return [14,15,0,0],[1.-blend,blend,0.,0.]
 if part.startswith('Body /'):return [1,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Neck /','Bib /','Beard /')):return [2,0,0,0],[1.,0.,0.,0.]
 return [3,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species!='entei' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid volcano-beast clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*16
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t);lag=math.sin(math.tau*t-.45)+math.sin(.45)
 if clip=='idle':body=.003*breath;neck=.007*breath;head=.011*breath;turn=.014*wave;flow=.017*lag;leg=.005*breath
 else:
  early=envelope(t,((0.,0.),(.18,1.),(.40,.68),(.67,.10),(1.,0.)))
  late=envelope(t,((0.,0.),(.28,1.),(.53,.60),(.79,-.08),(1.,0.)));sign=1. if clip=='attack' else -.65
  body=sign*.008*early;neck=sign*.017*early;head=sign*.027*early;turn=sign*.014*late;flow=sign*.028*late;leg=sign*.009*early
 return [(0.,0.,0.),(body,0.,0.),(neck,0.,0.),(head,turn,0.),
 (flow*.30,0.,-flow*.15),(flow*.30,0.,flow*.15),(flow*.5,flow*.25,0.),(flow,flow*.35,0.),
 (flow*.25,0.,-flow*.1),(flow*.25,0.,flow*.1),(leg,0.,0.),(leg,0.,0.),(-leg*.8,0.,0.),(-leg*.8,0.,0.),
 (flow*.4,flow*.3,0.),(flow,flow*.6,0.)]


def export_volcano_beast(species,model=None):
 if species not in SPECIES:raise ValueError('unknown volcano_beast species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed volcano_beast neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct volcano_beast anatomy and standard skin v1'
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
   rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(species,clip,i/(count-1))[joint])]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':species+'.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'early torso neck and armored head brace; planted paws; delayed mane blades and smoke; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/entei_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; four planted paws and ankle cuffs; coherent armored head; articulated mane dorsal blades and smoke trails; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('volcano_beast exceeds existing accessor limit')
 return g.bytes()


def decode_volcano_beast(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_volcano_beast(species):raise ValueError('noncanonical volcano_beast GLB')
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


def read_volcano_beast(path,species):
 with Path(path).open('rb') as f:return decode_volcano_beast(f.read(MAX_BYTES+1),species)
def export_entei(model=None):return export_volcano_beast('entei',model)
def read_entei(path):return read_volcano_beast(path,'entei')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/entei.glb'));a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_entei();a.output.write_bytes(blob);print(len(blob),'bytes',len(PARTS['entei']),'parts',len(JOINTS['entei']),'joints',REST_DIGESTS['entei'])
if __name__=='__main__':main()
