#!/usr/bin/env python3
"""Standard Snubbull/Granbull skins with planted paws, coherent spots, jaws and tusks."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from bulldog_sculpt import sculpture,HEIGHT,SPECIES
HEIGHTS={s:HEIGHT for s in SPECIES}
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'snubbull': '70065daa04c182094aef38bc6772adf416da67efaf666cc9bffe28b7b2854412', 'granbull': 'f724a10e0de09e5ecb92c5943b24bb23a1dacd35fa1909f888d8ee9142f4f344'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def arm_path(species,side):
 return ((side*.19,.68,.005),(side*.30,.55,.035),(side*.33,.56,.18)) if species=='granbull' else ((side*.099,.495,0.),(side*.193,.565,.021),(side*.278,.637,.078))
def joints(species):
 big=species=='granbull';left=arm_path(species,-1);right=arm_path(species,1)
 raw=[('root',None,(0.,0.,0.)),('torso',0,(0.,.399 if big else .286,-.012 if big else -.006)),
 ('head',1,(0.,.732 if big else .54,.015 if big else .001)),
 ('jaw',2,(0.,.75 if big else .640,.08 if big else .17)),
 ('arm_left',1,left[0]),('hand_left',4,left[-1]),('arm_right',1,right[0]),('hand_right',6,right[-1]),
 ('tail',1,(0.,.332 if big else .293,-.105))]
 for side in (-1,1):raw.append(('ear_'+('left' if side<0 else 'right'),2,(side*.18,1.015 if big else .891,.001)))
 for side in (-1,1):raw.append(('leg_'+('left' if side<0 else 'right'),0,(side*(.225 if big else .132),.18 if big else .124,-.015 if big else .009)))
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
 low,span=authoring_transform(species);raw=tuple(v*span/HEIGHT+(low if i==1 else 0.) for i,v in enumerate(position));x,y,z=raw;big=species=='granbull'
 if part.startswith(('Foot /','Toe groove /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  side=0 if part.endswith('-1') else 1;a=smoothstep(.10 if big else .08,.31 if big else .22,y)
  g=.6*smoothstep(.11 if big else .08,.18 if big else .12,y)*(1.-smoothstep(.25 if big else .16,.33 if big else .23,y))
  return [0,1,11+side,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith(('Arm /','Hand /','Hand fold /','Wrist /')):
  side=int(part.split()[-2] if part.startswith('Hand fold /') else part.split()[-1]);base=4 if side<0 else 6
  if not part.startswith('Arm /'):return [base+1,0,0,0],[1.,0.,0.,0.]
  if big:
   # A continuous chord field avoids nearest-segment switches across the
   # inside of the bent elbow, which compressed neighboring skin edges.
   path=arm_path(species,side);start,end=path[0],path[-1];delta=[b-a for a,b in zip(start,end)]
   t=max(0.,min(1.,sum((p-a)*d for p,a,d in zip(raw,start,delta))/sum(d*d for d in delta)))
  else:t=path_fraction(raw,arm_path(species,side))
  a=smoothstep(0.,.35,t);b=smoothstep(.25,.75,t)
  return [1,base,base+1,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith(('Ear /','Ear marking /')):return [9 if part.endswith('-1') else 10,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail /'):return [8,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Body /','Neck /','Collar /','Spot /')):
  f=smoothstep(.67 if big else .50,.80 if big else .59,y)
  return [1,2,0,0],[1.-f,f,0.,0.]
 if part.startswith(('Jaw /','Tusk /','Tongue /')):return [3,0,0,0],[1.,0.,0.,0.]
 return [2,0,0,0],[1.,0.,0.,0.]
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid bulldog clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 return [(0.,0.,0.),(0.,.015*a,0.),(.018*a,.025*a,.005*b),(.025*a,0.,0.),
  (.030*a,.015*b,-.025*a),(.025*b,.015*b,0.),(.030*a,-.015*b,.025*a),(.025*b,-.015*b,0.),
  (.030*b,.025*a,0.),(.015*b,0.,-.030*b),(.015*b,0.,.030*b),
  (0.,.018*a,0.),(0.,-.018*a,0.)]
def export_bulldog(species,model=None):
 if species not in SPECIES:raise ValueError('unknown bulldog species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed bulldog neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct bulldog jaws, folded ears and planted paws skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted paws and toe clefts; fitted faces and dress spots; coherent jaws and tusks; folded ears and articulated fists; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHTS[species],'body_reference_height':HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/bulldog_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; fixed paws and toe clefts; coherent fitted face, spots and ear tips; rigid lower jaw and tusks; cuffs and knuckle clefts share fists; articulated arms, wrists, ears, tail and leg middles; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'bulldog exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_bulldog(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_bulldog(species):raise ValueError('noncanonical bulldog GLB')
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


def read_bulldog(path,species):
 with Path(path).open('rb') as f:return decode_bulldog(f.read(MAX_BYTES+1),species)
def export_snubbull(model=None):return export_bulldog('snubbull',model)
def export_granbull(model=None):return export_bulldog('granbull',model)
def read_snubbull(path):return read_bulldog(path,'snubbull')
def read_granbull(path):return read_bulldog(path,'granbull')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_bulldog(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
