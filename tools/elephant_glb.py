#!/usr/bin/env python3
"""Standard elephant skins with planted pads, coherent ear markings and articulated trunks."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from phanpy_sculpt import sculpture as phanpy_sculpture
from donphan_sculpt import sculpture as donphan_sculpture,tire_rows
SPECIES=('phanpy','donphan');HEIGHTS={s:1.02 for s in SPECIES}
def sculpture(species,normalize=True):
 if species not in SPECIES:raise ValueError('unknown elephant species')
 return (phanpy_sculpture if species=='phanpy' else donphan_sculpture)(normalize=normalize)
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'phanpy': '4f4d743119d23709e102ad164adb17c52d236d9dca587c4ef10b3176e6fab0f4', 'donphan': '7a95df265d655ae2c8f7f273c5e42040da683449abe331b9c667955a295f70ca'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

TRUNKS={'phanpy':((0.,.720,.333),(0.,.657,.415),(0.,.652,.525),(0.,.706,.598)),
 'donphan':tuple(row[0] for row in tire_rows())}
TAILS={'phanpy':((0.,.420,-.350),(.025,.432,-.431),(.061,.453,-.487)),
 'donphan':((0.,.425,-.405),(.045,.451,-.465),(.096,.461,-.493))}
def joints(species):
 baby=species=='phanpy';trunk=TRUNKS[species];tail=TAILS[species]
 raw=[('root',None,(0.,0.,0.)),('torso',0,(0.,.386 if baby else .400,-.045 if baby else -.057)),
  ('head',1,(0.,.714,.177) if baby else (0.,.554,.249)),
  ('trunk_base',2,trunk[0] if baby else (0.,.650,.460)),('trunk_tip',3,trunk[2] if baby else (0.,.240,.554)),
  ('ear_left',2,(-.21,.78,.132) if baby else (-.16,.70,.148)),('ear_right',2,(.21,.78,.132) if baby else (.16,.70,.148)),
  ('tail_base',1,tail[0]),('tail_tip',7,tail[1])]
 for side in (-1,1):
  for row in (0,1):
   raw.append(('leg_'+('left' if side<0 else 'right')+('_front' if row==0 else '_rear'),1 if baby and side<0 and row==0 else 0,
    (-.183,.361,.165) if baby and side<0 and row==0 else (side*(.178 if baby else .218),.28 if baby else .30,(.205 if row==0 else -.256) if baby else (.183 if row==0 else -.292))))
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={species:joints(species) for species in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def path_fraction(point,path):
 best=(math.inf,0.);lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.
 for a,b,length in zip(path,path[1:],lengths):
  delta=[y-x for x,y in zip(a,b)];t=max(0.,min(1.,sum((p-x)*d for p,x,d in zip(point,a,delta))/(length*length)))
  best=min(best,(math.dist(point,[x+t*d for x,d in zip(a,delta)]),(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(species,part,position):
 baby=species=='phanpy';low,span=authoring_transform(species)
 raw=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position));x,y,z=raw
 if part.startswith(('Foot /','Toenail /','Ankle armor /')):
  owner=9 if baby and part.endswith('-1 front') else 0
  return [owner,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  side,row=part.split()[-2:];owner=9+(0 if side=='-1' else 2)+(0 if row=='front' else 1)
  if baby and owner==9:return [9,0,0,0],[1.,0.,0.,0.]
  a=smoothstep(.05 if baby else .211,.43 if baby else .40,y)
  g=.6*smoothstep(.08 if baby else .211,.18 if baby else .26,y)*(1.-smoothstep(.35 if baby else .32,.46 if baby else .40,y))
  return [0,1,owner,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith(('Ear /','Ear tip /','Ear lining /')):
  side=part.split()[-2] if part.startswith('Ear tip /') else part.split()[-1]
  return [5 if side=='-1' else 6,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Nose /','Nostril /')):return [4,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tread /'):
  plate=int(part.split()[-1]);owner=1 if plate<3 else 2 if plate<5 else 3 if plate<7 else 4
  return [owner,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Trunk /','Tire /')):
  # The baby's thick upturned trunk needs a monotone field across its full
  # cross-section; nearest-segment ownership can jump around the tight bend.
  t=(z-.333)/(.598-.333) if baby else path_fraction(raw,TRUNKS[species])
  if baby:
   a=smoothstep(0.,.4,t);b=smoothstep(.40,.90,t)
   return [2,3,4,0],[1.-a,a*(1.-b),a*b,0.]
  a=smoothstep(.23,.43,t);b=smoothstep(.60,.82,t);c=smoothstep(.82,.99,t)
  return [1,2,3,4],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
 if part.startswith('Tail /'):
  t=path_fraction(raw,TAILS[species]);a=smoothstep(0.,.55,t);b=smoothstep(.35,1.,t)
  return [1,7,8,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith('Body /'):return [1,0,0,0],[1.,0.,0.,0.]
 return [2,0,0,0],[1.,0.,0.,0.]
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid elephant clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 return [(0.,0.,0.),(0.,.012*a,0.),(.006*a,.015*a,0.),(.018*a,.012*a,0.),(.025*b,.015*b,0.),
  (.012*b,0.,-.022*b),(.012*b,0.,.022*b),(0.,.030*a,0.),(.020*b,.025*b,0.),
  (.025*b,0.,.012*b) if species=='phanpy' else (0.,.018*a,0.),(0.,-.018*a,0.),(0.,-.018*a,0.),(0.,.018*a,0.)]
def export_elephant(species,model=None):
 if species not in SPECIES:raise ValueError('unknown elephant species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed elephant neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite elephant trunk and planted pads standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted pads and fitted toenails; coherent fitted face and ear markings; articulated trunk, tail and leg middles; rigid tread plates on articulated tire; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/phanpy_sculpt.py and tools/donphan_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; fixed grounded pads/toes and adult ankle cuffs; coherent head features and ear markings; articulated trunks, tails and leg middles; baby lifted foot/nail share foreleg; adult flexible tire with rigid attached tread plates; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'elephant exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_elephant(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_elephant(species):raise ValueError('noncanonical elephant GLB')
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


def read_elephant(path,species):
 with Path(path).open('rb') as f:return decode_elephant(f.read(MAX_BYTES+1),species)
def export_phanpy(model=None):return export_elephant('phanpy',model)
def export_donphan(model=None):return export_elephant('donphan',model)
def read_phanpy(path):return read_elephant(path,'phanpy')
def read_donphan(path):return read_elephant(path,'donphan')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_elephant(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
