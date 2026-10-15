#!/usr/bin/env python3
"""Staryu standard skin with five radial arm chains and planted lower tips."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from staryu_sculpt import sculpture as sculpt,HEIGHT,CENTER,RADIUS,ANGLES
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

SPECIES=('staryu',)
REST_DIGESTS={'staryu':'c8ef0c854e722452e6850b79cd7df6b66598e7b21488b69ba818c5dc80e87a67'}
PARTS={'staryu':tuple(p['part'] for p in sculpt()['primitives'])};MAX_BYTES=1024*1024

def sculpture(species):
 if species!='staryu':raise ValueError('unsupported starfish')
 return sculpt()
@lru_cache(maxsize=1)
def authoring_transform(species):
 raw=sculpt(normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(point):
 low,span=authoring_transform('staryu');return tuple((v-(low if i==1 else 0.))*HEIGHT/span for i,v in enumerate(point))
JOINTS={'staryu':(('root',None,(0.,0.,0.)),('core',0,normalized(CENTER))) + tuple(
 (name,parent,normalized((r*math.cos(angle),CENTER[1]+r*math.sin(angle),0.)))
 for i,angle in enumerate(ANGLES)
 for name,parent,r in [('arm_'+str(i)+'_base',1,.145),('arm_'+str(i)+'_tip',2+2*i,.305)])}

CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))


def skin_weights(species,part,position):
 if not part.startswith(('Body /','Ridge /')):return [1,0,0,0],[1.,0.,0.,0.]
 low,span=authoring_transform(species)
 x=position[0]*span/HEIGHT;y=position[1]*span/HEIGHT+low-CENTER[1]
 r=math.hypot(x,y)/RADIUS;angle=math.atan2(y,x)
 i=min(range(5),key=lambda i:abs(math.atan2(math.sin(angle-ANGLES[i]),math.cos(angle-ANGLES[i]))))
 base=2+2*i;tip=base+1
 if r<.56:
  blend=smoothstep(.28,.56,r);js=[1,base,0,0];ws=[1.-blend,blend,0.,0.]
 else:
  blend=smoothstep(.56,.84,r);js=[base,tip,0,0];ws=[1.-blend,blend,0.,0.]
 if i in (2,3):
  pin=smoothstep(.57,.85,r);ws=[(1.-pin)*w for w in ws];js[2]=0;ws[2]=pin
 return js,ws

def pose_angles(species,clip,t):
 if species!='staryu' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid starfish clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*12
 wave=math.sin(math.tau*t);lag=math.sin(math.tau*t-.35)+math.sin(.35)
 if clip=='idle':early=.024*wave;late=.033*lag;turn=.006*wave
 else:
  sign=1. if clip=='attack' else -.65
  early=sign*.085*envelope(t,((0.,0.),(.17,1.),(.36,.68),(.65,.10),(1.,0.)))
  late=sign*.105*envelope(t,((0.,0.),(.27,1.),(.48,.65),(.76,-.08),(1.,0.)))
  turn=sign*.011*envelope(t,((0.,0.),(.17,1.),(.40,.5),(1.,0.)))
 angles=[(0.,0.,0.),(0.,turn,0.)]
 for i,a in enumerate(ANGLES):
  lower=i in (2,3);gain=.12 if lower else (1. if i==0 else .82)
  for bend in (early,late):angles.append((-math.sin(a)*bend*gain,math.cos(a)*bend*gain,0.))
 return angles


def export_starfish(species,model=None):
 if species not in SPECIES:raise ValueError('unknown starfish species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed starfish neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct starfish anatomy and standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'early central brace and radial arm flex; lower tips pinned; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/staryu_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; pinned lower arm tips; coherent core and gold mount; five radial arm chains; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('starfish exceeds existing accessor limit')
 return g.bytes()


def decode_starfish(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_starfish(species):raise ValueError('noncanonical starfish GLB')
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


def read_starfish(path,species):
 with Path(path).open('rb') as f:return decode_starfish(f.read(MAX_BYTES+1),species)
def export_staryu(model=None):return export_starfish('staryu',model)
def read_staryu(path):return read_starfish(path,'staryu')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/staryu.glb'));a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_staryu();a.output.write_bytes(blob);print(len(blob),'bytes',len(PARTS['staryu']),'parts',len(JOINTS['staryu']),'joints',REST_DIGESTS['staryu'])
if __name__=='__main__':main()
