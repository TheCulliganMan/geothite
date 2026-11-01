#!/usr/bin/env python3
"""Canonical Machop family with grounded toes and independently articulated two/four arms."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from machop_sculpt import sculpture,HEIGHTS,arm_paths
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=tuple(HEIGHTS)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=3)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'machop': '8ecc708258f7d11379cbba584594d05d59785a933c7efef426c0533c48d6c362', 'machoke': '899ca475595bcf10908fa43abaec72a3814825e5b33ac57f1821c6377f70bcb1', 'machamp': '71e409803fa06474215ea3bac8c59a53af7199737a3895d11564c03929a0df6d'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024
@lru_cache(maxsize=3)
def authoring_scale(species):
 raw=sculpture(species,normalize=False);return HEIGHTS[species]/max(v for p in raw['primitives'] for v in p['positions'][1::3])
def normalized(species,point):return tuple(v*authoring_scale(species) for v in point)
def arms(species):
 return tuple((row,side,points) for row,side,points,radii in arm_paths(species))
def joints(species):
 young=species=='machop';spread=.13 if young else (.22 if species=='machamp' else .205)
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.48 if young else .60,-.015)),('neck',1,(0.,.79 if young else 1.0,0.)),('head',2,(0.,.84 if young else 1.035,.035)),('leg_left',0,(-spread,.205,0.)),('leg_right',0,(spread,.205,0.))]
 for label,side,points in arms(species):
  base=len(raw);suffix=label+('_left' if side<0 else '_right')
  raw += [('arm_'+suffix,1,points[0]),('elbow_'+suffix,base,points[2]),('wrist_'+suffix,base+1,points[-1])]
 if young:raw += [('tail',1,(0.,.31,-.105))]
 return tuple((name,parent,normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def arm_fraction(species,index,point):
 # A continuous longitudinal field avoids nearest-segment jumps across bent elbows.
 points=arms(species)[index][2];a,b=points[0],points[-1];axis=[b[k]-a[k] for k in range(3)];span=sum(v*v for v in axis)
 return max(0.,min(1.,sum((point[k]-a[k])*axis[k] for k in range(3))/span))
def skin_weights(species,part,position):
 point=tuple(v/authoring_scale(species) for v in position);x,y,z=point;young=species=='machop'
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Toe /')):return rigid(0)
 if part.startswith('Leg /'):
  a=smoothstep(.09,.33 if young else .42,y);b=.01+.50*smoothstep(.08,.16,y)*(1.-smoothstep(.26,.40,y));owner=4 if part.endswith('-1') else 5
  return [owner,0,1,0],[b,(1.-a)*(1.-b),a*(1.-b),0.]
 if part.startswith(('Eye /','Brow /','Mouth /','Crest /')):return rigid(3)
 if part.startswith('Tail /'):return rigid(12)
 if part.startswith(('Arm /','Arm marking /','Hand /','Finger /','Thumb /')):
  label='lower' if species=='machamp' and ' / lower ' in part else 'upper';side=int(part.split()[-2] if part.startswith(('Finger /','Arm marking /')) else part.split()[-1]);idx=(2 if label=='lower' else 0)+int(side>0);base=6+3*idx
  if part.startswith(('Hand /','Finger /','Thumb /')):return rigid(base+2)
  t=arm_fraction(species,idx,point);a=smoothstep(0.,.25,t);b=smoothstep(.32,.62,t);c=smoothstep(.78,1.,t)
  return [1,base,base+1,base+2],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
 # One continuous pelvis/chest/neck surface; garment follows the same low field.
 a=smoothstep(.29 if young else .34,.44 if young else .54,y);b=smoothstep(.76 if young else .92,.82 if young else .99,y);c=smoothstep(.81 if young else .99,.865 if young else 1.025,y)
 if b>0.:return [0,1,2,3],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
 return [0,1,0,0],[1.-a,a,0.,0.]
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid fighting clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.015*a,0.),(.010*b,.010*a,0.),(.008*b,.018*a,0.),(0.,-.018*a,0.),(0.,.018*a,0.)]
 for label,side,points in arms(species):
  k=1. if label=='upper' else -.8
  out += [(-.025*k*a,side*.025*b,-side*.020*k*a),(.055*k*b,side*.020*a,side*.015*b),(.045*k*a,side*.025*b,side*.020*a)]
 if species=='machop':out += [(.015*b,.050*a,0.)]
 return out

def export_fighting(species,model=None):
 if species not in SPECIES:raise ValueError('unknown fighting species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed fighting neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite fighting continuous torso, grounded toes and independently articulated two or four arms skin v1'
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
  # Preserve exact f32 bytes, including signed zero in authored normals.
  key=(packed(values,component),kind,component,target,normalized)
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
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':species,'primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(species,clip,i/(count-1))[joint])]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':species+'.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed foot pads and toes; continuous body and neck; coherent fitted red gaze and ridges; independent shoulders, elbows and wrists; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/machop_sculpt.py','geometry_preservation':'distinct juvenile tail and mature muscular two/four-arm silhouettes; fitted red gaze, supported crests, and surface-fitted Machoke markings',
    'animation_contract':'identity floor root; fixed foot pads and toes; blended pelvis, torso, neck and coherent head; articulated arm chains with shared surface paint; independent juvenile tail; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'fighting exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('fighting exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('fighting exceeds existing byte limit')
 return blob


def decode_fighting(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_fighting(species):raise ValueError('noncanonical fighting GLB')
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


def read_fighting(path,species):
 with Path(path).open('rb') as f:return decode_fighting(f.read(MAX_BYTES+1),species)
def export_machop(model=None):return export_fighting('machop',model)
def read_machop(path):return read_fighting(path,'machop')
def export_machoke(model=None):return export_fighting('machoke',model)
def read_machoke(path):return read_fighting(path,'machoke')
def export_machamp(model=None):return export_fighting('machamp',model)
def read_machamp(path):return read_fighting(path,'machamp')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 for species in SPECIES:
  blob=export_fighting(species);(a.out/(species+'.glb')).write_bytes(blob);print(species,len(blob),'bytes',REST_DIGESTS[species],flush=True)
if __name__=='__main__':main()
