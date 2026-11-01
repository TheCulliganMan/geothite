#!/usr/bin/env python3
"""Onix standard skin with a rooted tail, connected rock-chain joints and a coherent head."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from onix_sculpt import sculpture as onix_sculpture,HEIGHT,CHAIN,anatomy,body_reference_span
def sculpt(*,normalize=True):
 if not normalize:return {'primitives':anatomy()}
 model=onix_sculpture();model['name']='onix';return model
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

SPECIES=('onix',)
REST_DIGESTS={'onix':'c36c6f0a7b6d955e1f5937d4862220768229c6ac7e661abc7b00fd354622fa10'}
PARTS={'onix':tuple(p['part'] for p in sculpt()['primitives'])};MAX_BYTES=1024*1024

def sculpture(species):
 if species!='onix':raise ValueError('unsupported rock_chain')
 return sculpt()
@lru_cache(maxsize=1)
def authoring_transform(species):
 raw=sculpt(normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(point):
 low,span=authoring_transform('onix');return tuple((v-(low if i==1 else 0.))*HEIGHT/span for i,v in enumerate(point))
JOINTS={'onix':(('root',None,(0.,0.,0.)),) + tuple(
 ('segment_'+str(j),j-3,normalized(CHAIN[j-1][:3])) for j in range(3,13)) +
 (('head',10,normalized(CHAIN[12][:3])),)}

CLIPS=(('idle',4.,85),('attack',1.,65),('hit',1.,65))


def rock_owner(j):return max(0,j-2)

def skin_weights(species,part,position):
 if part.startswith('Rock segment /'):
  return [rock_owner(int(part.split()[-1])),0,0,0],[1.,0.,0.,0.]
 if part.startswith('Joint /'):
  j=int(part.split()[-1]);a=normalized(CHAIN[j][:3]);b=normalized(CHAIN[j+1][:3])
  delta=[b[k]-a[k] for k in range(3)];length=sum(v*v for v in delta)
  t=max(0.,min(1.,sum((position[k]-a[k])*delta[k] for k in range(3))/length));blend=smoothstep(.10,.90,t)
  return [rock_owner(j),rock_owner(j+1),0,0],[1.-blend,blend,0.,0.]
 return [11,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species!='onix' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid rock-chain clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*12
 angles=[(0.,0.,0.)]
 for k in range(10):
  phase=k*.13;wave=math.sin(math.tau*t-phase)+math.sin(phase);breath=.5-.5*math.cos(math.tau*t)
  if clip=='idle':x=.0015*breath;y=.0025*wave;z=.0020*wave
  else:
   early=envelope(t,((0.,0.),(.18,1.),(.40,.70),(.67,.10),(1.,0.)))
   late=envelope(t,((0.,0.),(.27,1.),(.52,.65),(.78,-.08),(1.,0.)))
   sign=1. if clip=='attack' else -.65
   gain=.65+.035*k;x=sign*.0035*early*gain;y=sign*.0050*late*gain;z=sign*.0040*early*gain
  angles.append((x,y,z))
 if clip=='idle':angles.append((.004*breath,.011*wave,0.))
 else:angles.append((sign*.017*late,sign*.014*late,sign*.005*late))
 return angles


def export_rock_chain(species,model=None):
 if species not in SPECIES:raise ValueError('unknown rock_chain species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed rock_chain neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct rock_chain anatomy and standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'early connected rising rock-chain brace; pinned tail supports; coherent head and fitted face; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_centerline_length':body_reference_span(),
    'body_centerline_points':[normalized(p[:3]) for p in CHAIN]+[normalized((-.306,1.12,.10))],'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/onix_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; three fixed tail rocks; rigid boulders and blended buried joints; coherent head, face and horn; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('rock_chain exceeds existing accessor limit')
 return g.bytes()


def decode_rock_chain(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_rock_chain(species):raise ValueError('noncanonical rock_chain GLB')
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


def read_rock_chain(path,species):
 with Path(path).open('rb') as f:return decode_rock_chain(f.read(MAX_BYTES+1),species)
def export_onix(model=None):return export_rock_chain('onix',model)
def read_onix(path):return read_rock_chain(path,'onix')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/onix.glb'));a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_onix();a.output.write_bytes(blob);print(len(blob),'bytes',len(PARTS['onix']),'parts',len(JOINTS['onix']),'joints',REST_DIGESTS['onix'])
if __name__=='__main__':main()
