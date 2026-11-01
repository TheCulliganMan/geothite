#!/usr/bin/env python3
"""Sentret standard skin with one upper tail chain above the fixed rounded terminal support."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from sentret_sculpt import sculpture as sculpt,HEIGHT,TAIL,TAIL_KNOTS
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('sentret',)
REST_DIGESTS={'sentret':'ce94b353159bfa530c03d6a0b98c53daeb6c5db3de90602e3735f98b1b19965a'}
PARTS={'sentret':tuple(p['part'] for p in sculpt()['primitives'])};MAX_BYTES=1024*1024

def sculpture(species):
 if species!='sentret':raise ValueError('unsupported lookout')
 return sculpt()
@lru_cache(maxsize=1)
def authoring_transform(species):
 raw=sculpt(normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(point):
 low,span=authoring_transform('sentret');return tuple((v-(low if i==1 else 0.))*HEIGHT/span for i,v in enumerate(point))
JOINTS={'sentret':(('root',None,(0.,0.,0.)),('tail_support',0,normalized(TAIL_KNOTS[3])),
 ('tail_middle',1,normalized(TAIL_KNOTS[2])),('tail_upper',2,normalized(TAIL_KNOTS[1])),('body',3,normalized(TAIL_KNOTS[0])),
 ('ear_left',4,normalized((-.122,.875,-.010))),('ear_right',4,normalized((.122,.875,-.010))),
 ('arm_left',4,normalized((-.190,.650,.012))),('arm_right',4,normalized((.190,.650,.012))),
 ('leg_left',4,normalized((-.096,.394,.012))),('leg_right',4,normalized((.096,.394,.012))))}

CLIPS=(('idle',3.4,85),('attack',1.,65),('hit',1.,65))

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

def tail_parameter(point):
 best=float('inf');value=0.
 for i,(a,b) in enumerate(zip(TAIL,TAIL[1:])):
  delta=[b[k]-a[k] for k in range(3)];length=sum(v*v for v in delta)
  t=max(0.,min(1.,sum((point[k]-a[k])*delta[k] for k in range(3))/length))
  distance=sum((point[k]-a[k]-delta[k]*t)**2 for k in range(3))
  if distance<best:best=distance;value=(i+t)/8
 return value

def skin_weights(species,part,position):
 x,y,z=position
 if part.startswith(('Tail /','Tail band /')):
  low,span=authoring_transform(species);raw=(x*span/HEIGHT,y*span/HEIGHT+low,z*span/HEIGHT);q=tail_parameter(raw)
  knots=((0.,4),(.60,3),(1.45,2),(2.20,1),(2.70,0),(7.,0))
  for (a,j),(b,k) in zip(knots,knots[1:]):
   if a<=q<=b:
    blend=smoothstep(a,b,q);return [j,k,0,0],[1.-blend,blend,0.,0.]
  return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Ear /','Ear inset /')):return [5 if x<0 else 6,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Arm /','Hand /','Finger /')):return [7 if x<0 else 8,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Leg /','Foot /','Toe /')):return [9 if x<0 else 10,0,0,0],[1.,0.,0.,0.]
 return [4,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species!='sentret' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid lookout clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*11
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t);lag=math.sin(math.tau*t-.4)+math.sin(.4)
 if clip=='idle':support=.003*breath;body=.006*breath;ear=.011*lag;arm=.010*wave;leg=.005*wave
 else:
  early=envelope(t,((0.,0.),(.18,1.),(.40,.7),(.67,.12),(1.,0.)));late=envelope(t,((0.,0.),(.28,1.),(.52,.65),(.79,-.10),(1.,0.)));sign=1. if clip=='attack' else -.7
  support=sign*.007*early;body=sign*.016*early;ear=sign*.018*late;arm=sign*.031*early;leg=sign*.009*late
 return [(0.,0.,0.),(support,0.,0.),(support*.8,0.,0.),(support*.65,0.,0.),(body,.004*wave,0.),
  (ear,0.,-ear*.3),(ear,0.,ear*.3),
  (arm,0.,-arm*.2),(arm,0.,arm*.2),(leg,0.,0.),(leg,0.,0.)]

def export_lookout(species,model=None):
 if species not in SPECIES:raise ValueError('unknown lookout species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed lookout neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct lookout anatomy and standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'early tail-supported body brace; coherent belly annulus and striped tail; trailing ears and paws; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/sentret_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; pinned broad rounded terminal support; body follows upper tail chain; coherent face and belly annulus; articulated ears and limbs; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('lookout exceeds existing accessor limit')
 return g.bytes()


def decode_lookout(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_lookout(species):raise ValueError('noncanonical lookout GLB')
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


def read_lookout(path,species):
 with Path(path).open('rb') as f:return decode_lookout(f.read(MAX_BYTES+1),species)
def export_sentret(model=None):return export_lookout('sentret',model)
def read_sentret(path):return read_lookout(path,'sentret')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/sentret.glb'));a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_sentret();a.output.write_bytes(blob);print(len(blob),'bytes',len(PARTS['sentret']),'parts',len(JOINTS['sentret']),'joints',REST_DIGESTS['sentret'])
if __name__=='__main__':main()
