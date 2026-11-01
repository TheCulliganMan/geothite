#!/usr/bin/env python3
"""Standard Miltank/Tauros skins with planted hooves and coherent bovine anatomy."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from bovine_sculpt import sculpture,HEIGHTS,SPECIES
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'miltank': 'cab41187fb7632ada815e8d5ef241e6ed3e56af5ba964d6195e44cf5a09e851c', 'tauros': '9c7a80af7788de4b71e74f6217e1abc0eced54d06e9a6280dbd6fabd57da2edc'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS={'miltank': 1.12, 'tauros': 1.1175279}
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

MILK_TAIL=((0.,.22,-.143),(.07,.16,-.30),(.18,.16,-.43),(.26,.27,-.455))
BULL_TAILS=(((0.,.49,-.285),(-.09,.65,-.43),(-.25,.89,-.61),(-.29,1.08,-.61),(-.16,1.13,-.58)),
 ((.018,.49,-.29),(.12,.63,-.47),(.32,.67,-.60),(.39,.80,-.63),(.32,.92,-.65)),
 ((-.01,.48,-.28),(-.02,.55,-.47),(.04,.45,-.65),(.14,.41,-.68),(.24,.49,-.65)))
def arm_path(side):return ((side*.17,.55,-.005),(side*.27,.59,.01),(side*.33,.685,.035))
def joints(species):
 if species=='miltank':
  raw=[('root',None,(0.,0.,0.)),('torso',0,(0.,.43,-.015)),('head',1,(0.,.66,0.)),
   ('arm_left',1,arm_path(-1)[0]),('hand_left',3,(-.335,.695,.037)),('arm_right',1,arm_path(1)[0]),('hand_right',5,(.335,.695,.037)),
   ('ear_left',2,(-.14,.90,.018)),('ear_right',2,(.14,.90,.018)),
   ('tail_base',1,MILK_TAIL[0]),('tail_bend',9,MILK_TAIL[2]),('tail_ball',10,MILK_TAIL[-1]),
   ('leg_left',0,(-.17,.155,.035)),('leg_right',0,(.17,.155,.035))]
 else:
  raw=[('root',None,(0.,0.,0.)),('torso',0,(0.,.46,-.10)),('mane',1,(0.,.47,.18)),('head',2,(0.,.70,.32))]
  for side in (-1,1):
   for row in (0,1):raw.append(('leg_'+('left' if side<0 else 'right')+('_front' if row==0 else '_rear'),0,(side*(.145 if row==0 else .16),.27,.22 if row==0 else -.28)))
  for i,path in enumerate(BULL_TAILS):
   base=len(raw);raw.extend((('tail_'+str(i)+'_base',1,path[0]),('tail_'+str(i)+'_bend',base,path[2]),('tail_'+str(i)+'_tuft',base+1,path[-1])))
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
 low,span=authoring_transform(species);raw=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position));x,y,z=raw
 if species=='miltank':
  if part.startswith('Foot /'):return [0,0,0,0],[1.,0.,0.,0.]
  if part.startswith(('Body /','Udder /','Teat /','Spot /')):return [1,0,0,0],[1.,0.,0.,0.]
  if part.startswith('Leg /'):
   side=0 if part.endswith('-1') else 1;a=smoothstep(.08,.23,y);g=.6*smoothstep(.08,.12,y)*(1.-smoothstep(.18,.24,y))
   return [0,1,12+side,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
  if part.startswith(('Arm /','Hand /')):
   side=int(part.split()[-1]);base=3 if side<0 else 5
   if part.startswith('Hand /'):return [base+1,0,0,0],[1.,0.,0.,0.]
   t=path_fraction(raw,arm_path(side));a=smoothstep(0.,.40,t);b=smoothstep(.35,.95,t)
   return [1,base,base+1,0],[1.-a,a*(1.-b),a*b,0.]
  if part.startswith('Ear /'):return [7 if part.endswith('-1') else 8,0,0,0],[1.,0.,0.,0.]
  if part.startswith('Tail / rounded'):return [11,0,0,0],[1.,0.,0.,0.]
  if part.startswith('Tail /'):path=MILK_TAIL;base=9
  else:return [2,0,0,0],[1.,0.,0.,0.]
 else:
  if part.startswith(('Hoof /','Hoof groove /')):return [0,0,0,0],[1.,0.,0.,0.]
  if part.startswith('Body /'):return [1,0,0,0],[1.,0.,0.,0.]
  if part.startswith(('Mane /','Mane crease /')):return [2,0,0,0],[1.,0.,0.,0.]
  if part.startswith('Leg /'):
   side,row=map(int,part.split()[-2:]);owner=4+(0 if side<0 else 2)+row
   a=smoothstep(.20,.46,y);g=.65*smoothstep(.11,.20,y)*(1.-smoothstep(.35,.46,y))
   return [0,1,owner,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
  if part.startswith('Tail /'):
   j=int(part.split()[-1]);base=8+j*3
   if 'tuft' in part:return [base+2,0,0,0],[1.,0.,0.,0.]
   path=BULL_TAILS[j]
  else:return [3,0,0,0],[1.,0.,0.,0.]
 t=path_fraction(raw,path);a=smoothstep(0.,.20,t);b=smoothstep(.25,.70,t);c=smoothstep(.65,.93,t)
 return [1,base,base+1,base+2],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid bovine clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 if species=='miltank':return [(0.,0.,0.),(0.,.015*a,0.),(.018*a,.025*a,.005*b),
  (.030*a,.015*b,-.025*a),(.025*b,.012*b,0.),(.030*a,-.015*b,.025*a),(.025*b,-.012*b,0.),
  (.015*b,0.,-.025*b),(.015*b,0.,.025*b),(0.,.020*a,0.),(.020*b,.025*b,0.),(.020*b,.030*b,0.),
  (0.,.018*a,0.),(0.,-.018*a,0.)]
 result=[(0.,0.,0.),(0.,.015*a,0.),(0.,.015*a,0.),(.020*a,.025*b,.005*b)]
 result.extend((0.,(.018 if i<2 else -.018)*a,0.) for i in range(4))
 for j in range(3):
  sign=1. if j%2==0 else -1.
  result.extend(((0.,sign*.020*a,0.),(.018*b*(j+1)/3,sign*.025*b,0.),(.020*b,sign*.035*b,.012*b*(j-1))))
 return result
def export_bovine(species,model=None):
 if species not in SPECIES:raise ValueError('unknown bovine species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed bovine neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite milk cow and three-tailed bull standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted cloven hooves; coherent fitted face, udder and rear spots; rigid horns and forehead studs; separate three-stage tails; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/bovine_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; fixed hooves and clefts; coherent udder/teats and rear spots; coherent mane/creases; rigid horns and forehead studs; articulated forehooves, ears, leg middles and separate three-stage tails; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'bovine exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_bovine(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_bovine(species):raise ValueError('noncanonical bovine GLB')
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


def read_bovine(path,species):
 with Path(path).open('rb') as f:return decode_bovine(f.read(MAX_BYTES+1),species)
def export_miltank(model=None):return export_bovine('miltank',model)
def export_tauros(model=None):return export_bovine('tauros',model)
def read_miltank(path):return read_bovine(path,'miltank')
def read_tauros(path):return read_bovine(path,'tauros')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_bovine(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
