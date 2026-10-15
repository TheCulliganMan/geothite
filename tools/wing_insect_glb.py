#!/usr/bin/env python3
"""Distinct Butterfree/Beedrill standard skins over the original insect recipes."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from wing_insect_sculpt import sculpture,HEIGHT,SPECIES
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
REST_DIGESTS={'butterfree':'4b94b9ecfabca7dee88ec0d5b725d50116cb1f2a139d5a2817342e53cd4ee550','beedrill':'998ef3cf0f5f76c6ada93622528f1806dab5361c60d1b8ecb43c991f1979b900'}
PARTS={n:tuple(p['part'] for p in sculpture(n)['primitives']) for n in SPECIES}
MAX_BYTES=1024*1024
@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3])
 return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species)
 return tuple((v-(low if i==1 else 0.))*HEIGHT/span for i,v in enumerate(point))

def joints(species):
 def p(point):return normalized(species,point)
 result=[('root',None,(0.,0.,0.)),('thorax',0,p((0.,.56,0.))),('head',1,p((0.,.735,.055))),('abdomen',0,p((0.,.17,-.04)))]
 for which,y,z in [('upper',.61,-.09),('lower',.55,-.115)]:
  for side,s in [('left',-1),('right',1)]:
   root=len(result);result += [(which+'_'+side,1,p((s*.08,y,z))),(which+'_tip_'+side,root,p((s*.37,.84 if which=='upper' else .47,z-.02)))]
 for side,s in [('left',-1),('right',1)]:
  root=len(result);result += [('antenna_'+side,2,p((s*.064,.93,.019))),('antenna_tip_'+side,root,p((s*.12,1.06,-.01)))]
 result += [('arm_left',1,p((-.094,.60,.03))),('arm_right',1,p((.094,.60,.03)))]
 if species=='beedrill':result += [('drill_left',16,p((-.238,.52,.11))),('drill_right',17,p((.238,.52,.11)))]
 return tuple(result)
JOINTS={n:joints(n) for n in SPECIES}
CLIPS=(('idle',2.8,85),('attack',1.,65),('hit',1.,65))

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

def skin_weights(species,part,position):
 x,y,z=position;left=x<0;low,span=authoring_transform(species);raw_y=y*span/HEIGHT+low
 if part.startswith('Foot /') or (part.startswith('Leg /') and species=='butterfree'):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):return [3,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Wing'):
  lower='lower' in part
  if part.startswith('Wing vein /'):lower=int(part.split(' / ')[1].split()[1])>=(2 if species=='beedrill' else 4)
  root,tip=((8,9) if left else (10,11)) if lower else ((4,5) if left else (6,7))
  extent=abs(x)*span/HEIGHT;attach=smoothstep(.07,.17,extent);bend=smoothstep(.28,.56,extent)
  return [1,root,tip,0],[1.-attach,attach*(1.-bend),attach*bend,0.]
 if part.startswith('Antenna /'):
  root,tip=(12,13) if left else (14,15);bend=smoothstep(1.00,1.075,raw_y)
  return [root,tip,0,0],[1.-bend,bend,0.,0.]
 if part.startswith(('Head /','Eye /','Muzzle /','Mouth /')):return [2,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Drill /'):return [18 if left else 19,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Arm /','Hand /')):
  arm=16 if left else 17
  if species=='beedrill':
   bend=1.-smoothstep(.50,.57,raw_y);return [arm,18 if left else 19,0,0],[1.-bend,bend,0.,0.]
  return [arm,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Waist /'):
  upper=smoothstep(.435,.53,raw_y);return [3,1,0,0],[1.-upper,upper,0.,0.]
 if part.startswith('Stinger /'):
  upper=smoothstep(.015,.11,raw_y);return [0,3,0,0],[1.-upper,upper,0.,0.]
 if part.startswith('Abdomen'):return [3,0,0,0],[1.,0.,0.,0.]
 return [1,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid insect clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t);lag=math.sin(math.tau*t-.4)+math.sin(.4)
 if clip=='idle':brace=.007*breath;flap=.035*wave;trail=.020*lag;arm=.012*wave;feel=.018*lag
 else:
  early=envelope(t,((0.,0.),(.18,1.),(.40,.7),(.67,.12),(1.,0.)));late=envelope(t,((0.,0.),(.28,1.),(.52,.65),(.79,-.10),(1.,0.)));sign=1. if clip=='attack' else -.7
  brace=sign*.022*early;flap=sign*.052*early;trail=sign*.030*late;arm=sign*.044*early;feel=sign*.025*late
 result=[(0.,0.,0.),(brace,0.,0.),(brace*.55,0.,0.),(-brace*.35,.004*wave,0.),(0.,-flap,0.),(0.,-trail,0.),(0.,flap,0.),(0.,trail,0.),(0.,-flap*.65,0.),(0.,-trail*.85,0.),(0.,flap*.65,0.),(0.,trail*.85,0.),(feel,0.,-feel*.3),(feel*.75,0.,0.),(feel,0.,feel*.3),(feel*.75,0.,0.),(arm,0.,-arm*.35),(arm,0.,arm*.35)]
 if species=='beedrill':result += [(arm*.65,0.,-trail*.2),(arm*.65,0.,trail*.2)]
 return result

def export_insect(species,model=None):
 if species not in SPECIES:raise ValueError('unknown wing insect species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed wing insect neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct wing insect anatomy and standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'early thorax/head brace; coherent wing rims membranes veins; trailing tips and antennae; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/wing_insect_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; coherent compound eyes; connected four-wing fields; articulated limbs antennae gaster; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('wing insect exceeds existing accessor limit')
 return g.bytes()


def decode_insect(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_insect(species):raise ValueError('noncanonical wing insect GLB')
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


def read_insect(path,species):
 with Path(path).open('rb') as f:return decode_insect(f.read(MAX_BYTES+1),species)
def read_butterfree(path):return read_insect(path,'butterfree')
def read_beedrill(path):return read_insect(path,'beedrill')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--only',choices=SPECIES);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 for n in ([a.only] if a.only else SPECIES):
  blob=export_insect(n);(a.out/(n+'.glb')).write_bytes(blob);print(n,len(blob),'bytes',len(PARTS[n]),'parts',len(JOINTS[n]),'joints',REST_DIGESTS[n])
if __name__=='__main__':main()
