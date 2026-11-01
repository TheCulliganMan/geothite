#!/usr/bin/env python3
"""Standard Teddiursa/Ursaring skins with planted paws, coherent lunar markings and honey contact."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from bear_sculpt import sculpture,HEIGHT,SPECIES
HEIGHTS={s:HEIGHT for s in SPECIES}
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'teddiursa':'38a7cdab920c315dac0608ae661e9f9408e9a3b2709000393b88439a3dafbe3b',
 'ursaring':'3edc0d94b47fe39e7234b958db6c16a2a8254ae548ad21262b4d5994473260a0'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def arm_path(species,side):
 if species=='ursaring':return ((side*.177,.810,.005),(side*.294,.695,.025),(side*.316,.536,.070))
 return ((-.104,.511,.010),(-.224,.549,.090),(-.162,.640,.225),(-.120,.695,.274)) if side<0 else ((.109,.515,.005),(.219,.536,.028),(.286,.502,.091))
def joints(species):
 big=species=='ursaring';left=arm_path(species,-1);right=arm_path(species,1)
 raw=[('root',None,(0.,0.,0.)),('torso',0,(0.,.484 if big else .325,-.012 if big else -.005)),
 ('head',1,(0.,.975 if big else .595,.015 if big else .012)),
 ('arm_left',1 if big else 2,left[0]),('hand_left',3,left[-1]),
 ('arm_right',1,right[0]),('hand_right',5,right[-1]),
 ('tail',1,(0. if big else .027,.354 if big else .253,-.192 if big else -.173))]
 for side in (-1,1):raw.append(('ear_'+('left' if side<0 else 'right'),2,(side*(.129 if big else .198),1.170 if big else .961,-.008)))
 for side in (-1,1):raw.append(('leg_'+('left' if side<0 else 'right'),0,(side*(.228 if big else .133),.217 if big else .128,.003 if big else .005)))
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
 low,span=authoring_transform(species);raw=tuple(v*span/HEIGHT+(low if i==1 else 0.) for i,v in enumerate(position));x,y,z=raw;big=species=='ursaring'
 if part.startswith(('Foot /','Foot claw /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  side=0 if part.endswith('-1') else 1;a=smoothstep(.11 if big else .08,.33 if big else .20,y)
  g=.6*smoothstep(.12 if big else .08,.19 if big else .12,y)*(1.-smoothstep(.26 if big else .15,.34 if big else .22,y))
  return [0,1,10+side,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith(('Arm /','Hand /','Hand claw /')):
  side=int(part.split()[-2] if part.startswith('Hand claw /') else part.split()[-1]);base=3 if side<0 else 5
  if not part.startswith('Arm /'):return [base+1,0,0,0],[1.,0.,0.,0.]
  t=path_fraction(raw,arm_path(species,side));a=smoothstep(0.,.40,t);b=smoothstep(.35,1.,t)
  return [1,base,base+1,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith('Ear /'):return [8 if part.endswith('-1') else 9,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail /'):return [7,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Body /','Ring /','Neck /','Ruff /')):
  f=smoothstep(.88 if big else .53,1.02 if big else .65,y) if big or part.startswith('Neck /') else 0.
  return [1,2,0,0],[1.-f,f,0.,0.]
 return [2,0,0,0],[1.,0.,0.,0.]
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid bear clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength;big=species=='ursaring'
 left=(.035*a,.015*b,-.020*a) if big else (0.,0.,0.);paw=(.025*b,.010*b,0.) if big else (0.,0.,0.)
 return [(0.,0.,0.),(0.,.015*a,0.),(.015*a,.025*a,.005*b),left,paw,
  ((.035 if big else .025)*a,-.015*b,.020*a),(.025*b,-.010*b,0.),
  (.035*b,.025*a,0.),(.015*b,0.,-.025*b),(.015*b,0.,.025*b),
  (0.,.018*a,0.),(0.,-.018*a,0.)]
def export_bear(species,model=None):
 if species not in SPECIES:raise ValueError('unknown bear species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed bear neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite honey cub and ring bear standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted bear paws; honey-paw contact; fitted lunar markings and shoulder fur; articulated arms, ears and tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHTS[species],'body_reference_height':HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/bear_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; fixed paws and foot claws; fitted lunar marks and facial layers; cub honey paw follows head; adult shoulder fur shares chest/neck field; articulated arms, wrists, ears, tail and leg middles; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'bear exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_bear(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_bear(species):raise ValueError('noncanonical bear GLB')
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


def read_bear(path,species):
 with Path(path).open('rb') as f:return decode_bear(f.read(MAX_BYTES+1),species)
def export_teddiursa(model=None):return export_bear('teddiursa',model)
def export_ursaring(model=None):return export_bear('ursaring',model)
def read_teddiursa(path):return read_bear(path,'teddiursa')
def read_ursaring(path):return read_bear(path,'ursaring')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_bear(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
