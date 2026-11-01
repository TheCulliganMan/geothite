#!/usr/bin/env python3
"""Standard skins preserving the original Zubat/Golbat/Crobat anatomy."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from bat_sculpt import sculpture,HEIGHT,SPECIES
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
REST_DIGESTS={'zubat':'737d2dad1c97b82b0fead248d72ac003e5e759ce90debf0c4cabb06d80c851d0',
 'golbat':'2b4111963c9a7224224eacf4a70ec2359fd0e36d6426c7a992543e66e763379a',
 'crobat':'87f89cd4cc0e7d68b7f29292721ea068514dcdc982722d76fddcb8d23f69ced4'}
PARTS={n:tuple(p['part'] for p in sculpture(n)['primitives']) for n in SPECIES}
MAX_BYTES=1024*1024

@lru_cache(maxsize=3)
def authoring_transform(species):
 raw=sculpture(species,normalize=False)
 low=min(v for p in raw['primitives'] for v in p['positions'][1::3])
 high=max(v for p in raw['primitives'] for v in p['positions'][1::3])
 return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species)
 return tuple((v-(low if i==1 else 0.))*HEIGHT/span for i,v in enumerate(point))

def joints(species):
 def p(point):return normalized(species,point)
 result=[('root',None,(0.,0.,0.)),('body',0,p((0.,.55,-.07))),
  ('wing_left',1,p((-.12,.67,-.073))),('wrist_left',2,p((-.34,.78,-.09))),('tip_left',3,p((-.57,.90,-.10))),
  ('wing_right',1,p((.12,.67,-.073))),('wrist_right',5,p((.34,.78,-.09))),('tip_right',6,p((.57,.90,-.10))),
  ('ear_left',1,p((-.105,.855 if species=='golbat' else .79,-.018))),
  ('ear_right',1,p((.105,.855 if species=='golbat' else .79,-.018)))]
 if species=='crobat':result += [('lower_left',1,p((-.13,.43,-.08))),('lower_tip_left',10,p((-.35,.31,-.108))),
  ('lower_right',1,p((.13,.43,-.08))),('lower_tip_right',12,p((.35,.31,-.108)))]
 return tuple(result)
JOINTS={n:joints(n) for n in SPECIES}
CLIPS=(('idle',2.4,73),('attack',1.,65),('hit',1.,65))

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

def skin_weights(species,part,position):
 x,y,z=position
 if part.startswith(('Leg /','Toe /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Ear /'):return [8 if x<0 else 9,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Wing /','Wing bone /','Wing finger /')):
  # Use one continuous spatial field for skin, leading edge and branching spars.
  # Normalize the authoring X thresholds with the same rest transform.
  extent=abs(x)/normalized(species,(1.,0.,0.))[0]
  if 'lower' in part:
   root,tip=(10,11) if x<0 else (12,13)
   attach=smoothstep(.12,.22,extent);bend=smoothstep(.29,.55,extent)
   return [1,root,tip,0],[1.-attach,attach*(1.-bend),attach*bend,0.]
  root,wrist,tip=(2,3,4) if x<0 else (5,6,7)
  attach=smoothstep(.10,.20,extent);bend=smoothstep(.24,.46,extent);end=smoothstep(.51,.75,extent)
  return [1,root,wrist,tip],[1.-attach,attach*(1.-bend),attach*bend*(1.-end),attach*bend*end]
 return [1,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid bat clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
 lag=math.sin(math.tau*t-.35)+math.sin(.35)
 if clip=='idle':body=.008*breath;flap=.035*wave;trail=.023*lag;ear=.012*wave
 else:
  early=envelope(t,((0.,0.),(.18,1.),(.40,.7),(.67,.12),(1.,0.)))
  late=envelope(t,((0.,0.),(.28,1.),(.52,.65),(.79,-.10),(1.,0.)))
  sign=1. if clip=='attack' else -.7
  body=sign*.024*early;flap=sign*.055*early;trail=sign*.036*late;ear=sign*.023*late
 result=[(0.,0.,0.),(body,0.,0.),
  (.008*wave,0.,-flap),(0.,-trail,0.),(0.,-trail*.65,0.),
  (.008*wave,0.,flap),(0.,trail,0.),(0.,trail*.65,0.),
  (.007*wave,0.,-ear),(.007*wave,0.,ear)]
 if species=='crobat':result += [(0.,0.,-abs(trail)*.65),(0.,-flap*.5,0.),(0.,0.,abs(trail)*.65),(0.,flap*.5,0.)]
 return result

def export_bat(species,model=None):
 if species not in SPECIES:raise ValueError('unknown bat species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed bat neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct bat anatomy and standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'early body brace; connected wing spars and membranes; trailing wing tips and ears; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/bat_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; coherent hollow face; connected multi-joint wing fields; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('bat exceeds existing accessor limit')
 return g.bytes()


def decode_bat(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_bat(species):raise ValueError('noncanonical bat GLB')
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


def read_bat(path,species):
 with Path(path).open('rb') as f:return decode_bat(f.read(MAX_BYTES+1),species)
def read_zubat(path):return read_bat(path,'zubat')
def read_golbat(path):return read_bat(path,'golbat')
def read_crobat(path):return read_bat(path,'crobat')

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--only',choices=SPECIES);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 for n in ([a.only] if a.only else SPECIES):
  blob=export_bat(n);(a.out/(n+'.glb')).write_bytes(blob);print(n,len(blob),'bytes',len(PARTS[n]),'parts',len(JOINTS[n]),'joints',REST_DIGESTS[n])
if __name__=='__main__':main()
