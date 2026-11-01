#!/usr/bin/env python3
"""Canonical Koffing and Weezing cratered orbs, linked heads and independent drifting vapor."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from gas_pair_sculpt import sculpture,HEIGHTS,CLOUDS,LOBE_SHAPES
SPECIES=tuple(HEIGHTS)
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'koffing': 'ba5c5ad08e422850b832e4cb9412e18bcecbc53b62530186b11fa4f879b957de', 'weezing': 'b655fa7a75abd024c672927149600b45231e833200036b2c7bc97ed9ddba0c70'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def gas_base(species):return 2 if species=='koffing' else 4
def joints(species):
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.47,0.) if species=='koffing' else (-.22,.51,0.))]
 if species=='weezing':raw += [('small_orb',1,(.10,.80,-.05)),('rear_orb',1,(.15,.37,-.13))]
 for cloud,c in enumerate(CLOUDS[species]):
  owner=len(raw);raw += [('gas_cloud_'+str(cloud),0,(c[0],c[1]-.14,c[2]))]
  for k,shape in enumerate(LOBE_SHAPES):raw += [('gas_lobe_'+str(cloud)+'_'+str(k),owner,tuple(c[i]+shape[i]*.5 for i in range(3)))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 low,span=authoring_transform(species);point=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position));x,y,z=point
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Gas cloud /'):return rigid(gas_base(species)+5*int(part.split()[-1]))
 if part.startswith('Gas lobe /'):
  cloud,k=map(int,part.split()[-2:]);core=gas_base(species)+5*cloud;center=CLOUDS[species][cloud];axis=LOBE_SHAPES[k][:3]
  # Tiny authored polar cap rings share one weight field, preventing U8
  # weight steps from pinching their sub-millimeter edges during articulation.
  shape=LOBE_SHAPES[k];lobe=tuple(center[i]+shape[i] for i in range(3))
  if abs((point[1]-lobe[1])/shape[4])>.99:point=(lobe[0],point[1],lobe[2])
  t=sum((point[i]-center[i])*axis[i] for i in range(3))/sum(v*v for v in axis);a=smoothstep(.65,1.65,t)
  return [core,core+1+k,0,0],[1.-a,a,0.,0.]
 if part.startswith('Main stalk /'):
  a=smoothstep(.035,.15,x);return [1,2,0,0],[1.-a,a,0.,0.]
 if part.startswith('Rear stalk /'):
  a=smoothstep(.10,.235,x);return [1,3,0,0],[1.-a,a,0.,0.]
 if part.startswith('Rear node /'):return rigid(3)
 if part.startswith('Small body /') or 'small Weezing' in part:return rigid(2)
 return rigid(1)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid gas clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength;out=[(0.,0.,0.),(0.,.012*a,0.)]
 if species=='weezing':out += [(.015*a,.025*b,.020*a),(.018*b,.025*a,-.018*b)]
 for cloud in range(2):
  phase=.73+cloud*2.11
  if clip=='idle':w=math.sin(math.tau*t+phase)-math.sin(phase)
  else:w=.55*a*math.cos(phase)+.45*b*math.sin(phase)
  out += [(.025*w,.045*b,.055*w)]
  for k in range(4):
   p=phase+k*2.399963229728653
   if clip=='idle':v=math.sin(math.tau*t+p)-math.sin(p)
   else:v=.65*a*math.cos(p)+.35*b*math.sin(p)
   out += [(.022*v,.030*v,.020*v)]
 return out

def export_gas(species,model=None):
 if species not in SPECIES:raise ValueError('unknown gas species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed gas neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite gas cratered orbs, coherent faces, connected Weezing heads and independently phased vapor skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'coherent cratered faces; connected articulated adult orbs; two independent vapor clouds with four rooted lobe carriers each; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/gas_pair_sculpt.py','geometry_preservation':'thirteen actual radial recesses per main orb, recessed expressions, fitted skull and crossbones, rooted canines, three connected adult orbs and two detached closed five-lobed gas clouds',
    'animation_contract':'identity root; main orb yaw preserves authoring floor; rigid coherent fitted faces and fangs; adult small and rear orbs pivot independently through blended stalks; two root-parented vapor clouds have distinct phases and four lobe carriers each, blending from occupied core roots; existing intentional battle/world hover preserved; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'gas exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('gas exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('gas exceeds existing byte limit')
 return blob


def decode_gas(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_gas(species):raise ValueError('noncanonical gas GLB')
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


def read_gas(path,species):
 with Path(path).open('rb') as f:return decode_gas(f.read(MAX_BYTES+1),species)
def export_koffing(model=None):return export_gas('koffing',model)
def read_koffing(path):return read_gas(path,'koffing')
def export_weezing(model=None):return export_gas('weezing',model)
def read_weezing(path):return read_gas(path,'weezing')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_gas(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
