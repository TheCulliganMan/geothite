#!/usr/bin/env python3
"""Standard psychic bird skins with planted talons and coherent fitted plumage."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from psychic_bird_sculpt import sculpture,HEIGHT,SPECIES
HEIGHTS={s:HEIGHT for s in SPECIES}
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'natu': '42b91ce42d9cbfb2367f3640cdd1641c5da72aedb65da5cc41b412d9d8b09fd1', 'xatu': '5196df1ef836e954c4904ec06cf3448e3a701eba946050d5a38a477d116e785b'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def joints(species):
 baby=species=='natu'
 raw=[('root',None,(0.,0.,0.)),('torso',0,(0.,.332 if baby else .49,0.)),('head',1,(0.,.35,0.) if baby else (0.,.97,.015)),
  ('wing_left',2 if baby else 1,(-.221,.305,-.021) if baby else (-.14,.70,.07)),('wing_right',2 if baby else 1,(.221,.305,-.021) if baby else (.14,.70,.07))]
 if baby:
  raw.extend((('crest',2,(0.,.510,-.046)),('tail_left',1,(0.,.186,-.133)),('tail_right',1,(0.,.186,-.133))))
 else:
  for side in (-1,1):
   base=len(raw);raw.extend((('streamer_'+('left' if side<0 else 'right'),2,(side*.056,1.111,-.031)),('streamer_'+('left' if side<0 else 'right')+'_tip',base,(side*.113,.682,-.166))))
 for side in (-1,1):raw.append(('ankle_'+('left' if side<0 else 'right'),0,(side*.097,.11 if baby else .12,.01)))
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={species:joints(species) for species in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 baby=species=='natu';low,span=authoring_transform(species)
 raw=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position));x,y,z=raw
 if part.startswith(('Toe /','Claw /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  top=.146 if baby else .168;owner=(8 if baby else 9)+(0 if part.endswith('-1') else 1)
  a=smoothstep(.077,top,y);g=.4*smoothstep(.077,.10,y)*(1.-smoothstep(.13,top,y))
  return [0,1,owner,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith(('Wing /','Wing gold /','Wing red /','Wing badge','Wing hem')):
  name=part.removesuffix(' rear')
  side=name.split()[-2] if part.startswith(('Wing red /','Wing hem')) else name.split()[-1]
  return [3 if side=='-1' else 4,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Body /','Collar /','Chest symbol /','Chest pupil /','Back stripe /')):
  a=smoothstep(.20 if baby else .78,.29 if baby else .90,y)
  return [1,2,0,0],[1.-a,a,0.,0.]
 if part.startswith('Crest /'):
  a=smoothstep(.510,.612,y);return [2,5,0,0],[1.-a,a,0.,0.]
 if part.startswith('Tail /'):
  owner=6 if part.endswith('-1') else 7;a=smoothstep(.133,.280,-z)
  return [1,owner,0,0],[1.-a,a,0.,0.]
 if part.startswith('Streamer /'):
  base=5 if part.endswith('-1') else 7;t=(1.111-y)/.749
  a=smoothstep(0.,.35,t);b=smoothstep(.30,.90,t)
  return [2,base,base+1,0],[1.-a,a*(1.-b),a*b,0.]
 return [2,0,0,0],[1.,0.,0.,0.]
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid psychic bird clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 result=[(0.,0.,0.),(0.,.012*a,0.),(.006*a,.018*a,0.),(.010*a,.012*b,-.012*a),(.010*a,-.012*b,.012*a)]
 if species=='natu':result.extend(((.020*b,0.,-.012*b),(0.,.025*b,-.010*b),(0.,-.025*b,.010*b)))
 else:result.extend(((.010*a,0.,-.010*a),(.020*b,.012*b,-.020*b),(.010*a,0.,.010*a),(.020*b,-.012*b,.020*b)))
 result.extend(((0.,.018*a,0.),(0.,-.018*a,0.)))
 return result
def export_psychic_bird(species,model=None):
 if species not in SPECIES:raise ValueError('unknown psychic_bird species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed psychic_bird neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite psychic bird planted talons and articulated plumage skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted toes and talons; coherent fitted face and wing coatings; articulated crest, tail feathers and streamers; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/psychic_bird_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; fixed toes and claws; blended supporting ankles; coherent face and wing markings; shared torso/collar/chest/rear stripe field; articulated crest, tail feathers and independent two-stage head streamers; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'psychic_bird exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_psychic_bird(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_psychic_bird(species):raise ValueError('noncanonical psychic_bird GLB')
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


def read_psychic_bird(path,species):
 with Path(path).open('rb') as f:return decode_psychic_bird(f.read(MAX_BYTES+1),species)
def export_natu(model=None):return export_psychic_bird('natu',model)
def export_xatu(model=None):return export_psychic_bird('xatu',model)
def read_natu(path):return read_psychic_bird(path,'natu')
def read_xatu(path):return read_psychic_bird(path,'xatu')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_psychic_bird(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
