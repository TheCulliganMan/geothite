#!/usr/bin/env python3
"""Hoothoot canonical standard skin with fitted owl face and clock-hand brows."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from hoothoot_sculpt import sculpture as sculpt,HEIGHT
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('hoothoot',)
REST_DIGESTS={'hoothoot':'93fbea6a47abff0fff1f6ee6315f7ac09ea26bb1bf60a26b0d293a8ae7784555'}
PARTS={'hoothoot':tuple(p['part'] for p in sculpt()['primitives'])};MAX_BYTES=1024*1024

def sculpture(species):
 if species!='hoothoot':raise ValueError('unsupported owl')
 return sculpt()
@lru_cache(maxsize=1)
def authoring_transform(species):
 raw=sculpt(normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(point):
 low,span=authoring_transform('hoothoot');return tuple((v-(low if i==1 else 0.))*HEIGHT/span for i,v in enumerate(point))
JOINTS={'hoothoot':(('root',None,(0.,0.,0.)),('body',0,normalized((0.,.37,0.))),
 ('brow_left',1,normalized((-.034,.629,.182))),('brow_tip_left',2,normalized((-.17,.745,.075))),
 ('brow_right',1,normalized((.034,.629,.182))),('brow_tip_right',4,normalized((.17,.745,.075))),
 ('wing_left',1,normalized((-.246,.47,-.01))),('wing_tip_left',6,normalized((-.285,.255,-.10))),
 ('wing_right',1,normalized((.246,.47,-.01))),('wing_tip_right',8,normalized((.285,.255,-.10))),
 ('tail',1,normalized((0.,.185,-.196))),('lower_beak',1,normalized((0.,.435,.224))))}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

def skin_weights(species,part,position):
 x,y,z=position;low,span=authoring_transform(species);raw_y=y*span/HEIGHT+low;rear=-z*span/HEIGHT
 if part.startswith(('Leg /','Foot /','Toe /','Claw /','Rear toe /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Body /','Breast mark /')):
  upper=smoothstep(.10,.20,raw_y);return [0,1,0,0],[1.-upper,upper,0.,0.]
 if part.startswith('Clock brow /'):
  root,tip=(2,3) if x<0 else (4,5);attach=smoothstep(.64,.70,raw_y);end=smoothstep(.735,.795,raw_y)
  return [1,root,tip,0],[1.-attach,attach*(1.-end),attach*end,0.]
 if part.startswith(('Wing /','Flight feather /')):
  root,tip=(6,7) if x<0 else (8,9);bend=1.-smoothstep(.19,.34,raw_y)
  return [root,tip,0,0],[1.-bend,bend,0.,0.]
 if part.startswith('Tail /'):
  blend=smoothstep(.20,.29,rear);return [1,10,0,0],[1.-blend,blend,0.,0.]
 if part.startswith('Lower beak /'):return [11,0,0,0],[1.,0.,0.,0.]
 return [1,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species!='hoothoot' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid owl clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*12
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t);lag=math.sin(math.tau*t-.4)+math.sin(.4)
 if clip=='idle':body=.005*breath;brow=.010*lag;wing=.015*wave;tip=.010*lag;tail=.017*wave;jaw=.005*breath
 else:
  early=envelope(t,((0.,0.),(.18,1.),(.40,.7),(.67,.12),(1.,0.)));late=envelope(t,((0.,0.),(.28,1.),(.52,.65),(.79,-.10),(1.,0.)));sign=1. if clip=='attack' else -.7
  body=sign*.021*early;brow=sign*.018*late;wing=sign*.032*early;tip=sign*.023*late;tail=sign*.025*late;jaw=sign*.014*early
 return [(0.,0.,0.),(body,.006*wave,0.),(brow,0.,-brow*.2),(brow*.7,0.,0.),(brow,0.,brow*.2),(brow*.7,0.,0.),(wing,0.,-wing*.2),(tip,0.,0.),(wing,0.,wing*.2),(tip,0.,0.),(tail*.4,tail,0.),(jaw,0.,0.)]

def export_owl(species,model=None):
 if species not in SPECIES:raise ValueError('unknown owl species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed owl neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct owl anatomy and standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'early round body brace; fitted owl face; trailing clock-hand brows folded wings and tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/hoothoot_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; one planted and one tucked foot; coherent fitted owl eyes; articulated clock-hand brows folded wings tail and jaw; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('owl exceeds existing accessor limit')
 return g.bytes()


def decode_owl(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_owl(species):raise ValueError('noncanonical owl GLB')
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


def read_owl(path,species):
 with Path(path).open('rb') as f:return decode_owl(f.read(MAX_BYTES+1),species)
def export_hoothoot(model=None):return export_owl('hoothoot',model)
def read_hoothoot(path):return read_owl(path,'hoothoot')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/hoothoot.glb'));a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_hoothoot();a.output.write_bytes(blob);print(len(blob),'bytes',len(PARTS['hoothoot']),'parts',len(JOINTS['hoothoot']),'joints',REST_DIGESTS['hoothoot'])
if __name__=='__main__':main()
