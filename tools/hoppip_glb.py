#!/usr/bin/env python3
"""Canonical Hoppip skin with coherent serrated leaves and grounded feet."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from wind_plant_sculpt import sculpture,HEIGHT,crown_point,leaf_midline_point
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('hoppip',)
REST_DIGESTS={'hoppip':'aaa5bb6c980f1ba8d3327f7f611a059193c29ae7092e35f06acab3b9181507b4'}
PARTS={n:tuple(p['part'] for p in sculpture(n)['primitives']) for n in SPECIES}
MAX_BYTES=512000
RAW=sculpture('hoppip',normalize=False)
LOW=min(v for p in RAW['primitives'] for v in p['positions'][1::3])
SCALE=HEIGHT/(max(v for p in RAW['primitives'] for v in p['positions'][1::3])-LOW)
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
JOINTS={'hoppip':tuple((name,parent,pivot(p) if parent is not None else p) for name,parent,p in (
 ('root',None,(0.,0.,0.)),('torso',0,(0.,.25,0.)),
 ('arm_left',1,(-.18,.33,.145)),('arm_right',1,(.18,.33,.145)),
 ('ear_left',1,(-.15,.53,.038)),('ear_right',1,(.15,.53,.038)),
 ('leaf_left',1,(-.027,.604,-.038)),('leaf_tip_left',6,crown_point(leaf_midline_point(.894,-1)[0],-1)),
 ('leaf_right',1,(.027,.604,-.038)),('leaf_tip_right',8,crown_point(leaf_midline_point(.894,1)[0],1)),
 ('tail',1,(0.,.25,-.15))))}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

def skin_weights(species,part,position):
 x,y,z=position;raw_y=y/SCALE+LOW
 if part.startswith('Foot /'):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Leaf /','Leaf vein /')):
  root,tip=(6,7) if x<0 else (8,9)
  bend=smoothstep(.73,.98,raw_y)
  return [root,tip,0,0],[1.-bend,bend,0.,0.]
 if part.startswith('Ear /'):return [4 if x<0 else 5,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Arm /'):return [2 if x<0 else 3,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail /'):return [10,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Body /'):
  upper=smoothstep(.16,.25,raw_y)
  return [0,1,0,0],[1.-upper,upper,0.,0.]
 return [1,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species!='hoppip' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Hoppip clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*11
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
 lag=math.sin(math.tau*t-.55)+math.sin(.55)
 if clip=='idle':
  body=.009*breath;arm=.025*wave;ear=.020*lag;leaf=.038*wave;tip=.050*lag;tail=.038*lag
 else:
  early=envelope(t,((0.,0.),(.18,1.),(.40,.68),(.68,.10),(1.,0.)))
  late=envelope(t,((0.,0.),(.29,1.),(.54,.60),(.79,-.12),(1.,0.)))
  sign=1. if clip=='attack' else -.8
  body=sign*.028*early;arm=sign*.045*early;ear=sign*.040*late
  leaf=sign*.060*early;tip=sign*.085*late;tail=sign*.065*late
 return [(0.,0.,0.),(body,0.,.006*wave),
         (arm,0.,-.012*breath),(arm,0.,.012*breath),
         (ear,0.,-.010*wave),(ear,0.,.010*wave),
         (leaf,0.,-.012*breath),(tip,0.,-.016*breath),
         (leaf,0.,.012*breath),(tip,0.,.016*breath),(0.,tail,.012*wave)]

def export_hoppip(model=None):
 species='hoppip'
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed wind plant neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct wind plant anatomy and standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'body brace; delayed leaf tips and tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/wind_plant_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; fixed supporting feet; fitted face; articulated ears leaves arms and tail; coherent leaf veins; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('wind plant exceeds existing accessor limit')
 return g.bytes()


def decode_hoppip(blob):
 species='hoppip'
 if len(blob)>MAX_BYTES or blob!=export_hoppip():raise ValueError('noncanonical wind plant GLB')
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


def read_hoppip(path):
 with Path(path).open('rb') as f:return decode_hoppip(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--only',choices=SPECIES);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 for n in ([a.only] if a.only else SPECIES):
  blob=export_hoppip();(a.out/(n+'.glb')).write_bytes(blob);print(n,len(blob),'bytes',len(PARTS[n]),'parts',len(JOINTS[n]),'joints',REST_DIGESTS[n])
if __name__=='__main__':main()
