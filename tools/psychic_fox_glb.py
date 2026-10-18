#!/usr/bin/env python3
"""Canonical Abra and Alakazam skins with planted claws and articulated psychic props."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from abra_sculpt import sculpture,HEIGHT
SPECIES=('abra','alakazam');HEIGHTS={s:HEIGHT for s in SPECIES}
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'abra': '0f62090ca5311bb42f9f3a8375859f0336df8bfe1e76752e34588c9deb257eb9', 'alakazam': '9798ee114b47db333f11ac0dfddb761373146d62684df9051613f8585fdb4f5c'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def limb_points(species,side):
 adult=species=='alakazam'
 shoulder=(side*(.210 if adult else .163),.592 if adult else .518,-.006)
 elbow=(side*.302,.510,.019) if adult else (side*.213,.357,.028)
 wrist=(side*.413,.524,.186) if adult else (side*.129,.275,.157)
 hip=(side*.114,.279,-.024) if adult else (side*.087,.204,-.024)
 knee=(side*.240,.272,.087) if adult else (side*.220,.139,.100)
 return shoulder,elbow,wrist,hip,knee

def joints(species):
 adult=species=='alakazam';raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.36,0.)),('head',1,(0.,.655 if adult else .600,0.))]
 for side in (-1,1):
  shoulder,elbow,wrist,_,_=limb_points(species,side);j=len(raw);label='left' if side<0 else 'right'
  raw += [('arm_'+label,1,shoulder),('elbow_'+label,j,elbow),('wrist_'+label,j+1,wrist)]
 raw += [('leg_left',0,limb_points(species,-1)[3]),('leg_right',0,limb_points(species,1)[3]),
         ('ear_left',2,(-.150,.854 if adult else .822,0.)),('ear_right',2,(.150,.854 if adult else .822,0.))]
 if adult:
  raw += [('moustache_left',2,(-.005,.665,.315)),('moustache_right',2,(.005,.665,.315))]
  for side in (-1,1):
   wrist=limb_points(species,side)[2];raw += [('spoon_left' if side<0 else 'spoon_right',5 if side<0 else 8,(wrist[0],wrist[1]-.041,wrist[2]+.061))]
 else:raw += [('tail_base',1,(0.,.179,-.100)),('tail_tip',13,(.192,.348,-.507))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
@lru_cache(maxsize=1)
def tail_path():
 from abra_sculpt import TAIL_POINTS,TAIL_RADII
 from bovine_sculpt import smooth
 return smooth(TAIL_POINTS,TAIL_RADII,12)[0]
def projection(point,a,b):
 axis=[b[k]-a[k] for k in range(3)];return sum((point[k]-a[k])*axis[k] for k in range(3))/sum(v*v for v in axis)
def path_fraction(point,path):
 lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.;best=(math.inf,0.)
 for a,b,length in zip(path,path[1:],lengths):
  t=max(0.,min(1.,projection(point,a,b)));best=min(best,(math.dist(point,[a[k]+t*(b[k]-a[k]) for k in range(3)]),(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(species,part,position):
 low,span=authoring_transform(species);point=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position));x,y,z=point;adult=species=='alakazam'
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Foot /'):return rigid(0)
 if part.startswith('Ear /'):return rigid(11 if part.endswith('-1') else 12)
 if part.startswith('Moustache /'):return rigid(13 if part.endswith('-1') else 14)
 if part.startswith('Spoon /'):return rigid(15 if part.endswith('-1') else 16)
 if part.startswith('Armor / shoulder'):return rigid(1)
 if part.startswith(('Body /','Armor / shaped chest')):
  a=smoothstep(.10,.27,y);h=smoothstep(.595 if adult else .535,.670 if adult else .605,y)
  return [0,1,2,0],[1.-a,a*(1.-h),a*h,0.]
 if part.startswith(('Leg /','Armor / fitted knee')):
  side=-1 if part.endswith('-1') else 1;owner=9 if side<0 else 10
  if part.startswith('Armor /'):return rigid(owner)
  if part.startswith('Leg / narrow'):
   a=smoothstep(.065,.20 if adult else .125,y);return [owner,0,0,0],[.01+.99*a,.99*(1.-a),0.,0.]
  hip,knee=limb_points(species,side)[3:];t=projection(point,hip,knee);a=smoothstep(.10,.27,y);b=smoothstep(0.,.65,t)
  return [owner,0,1,0],[.01+.99*b,.99*(1.-b)*(1.-a),.99*(1.-b)*a,0.]
 if part.startswith(('Arm /','Hand /')):
  side=int(part.split()[-2] if part.startswith('Hand / curled finger') else part.split()[-1]);upper=3 if side<0 else 6;elbow=upper+1;hand=upper+2
  if part.startswith('Hand /'):return rigid(hand)
  shoulder,elbow_point,wrist,_,_=limb_points(species,side)
  if part.startswith('Arm / tapered upper'):
   a=smoothstep(0.,.40,projection(point,shoulder,elbow_point));return [1,upper,0,0],[1.-a,a,0.,0.]
  t=projection(point,elbow_point,wrist);e=smoothstep(.05,.40,t);w=smoothstep(.72,.95,t)
  return [upper,elbow,hand,0],[(1.-e)*(1.-w),e*(1.-w),w,0.]
 if part.startswith('Tail /'):
  t=path_fraction(point,tail_path());a=smoothstep(0.,.20,t);b=smoothstep(.50,.85,t)
  return [1,13,14,0],[1.-a,a*(1.-b),a*b,0.]
 return rigid(2)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid psychic clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.008*a,0.),(.010*a,.015*b,0.)]
 for side in (-1,1):out += [(.015*b,side*.018*b,side*.024*a),(.025*a,side*.015*b,0.),(.018*b,0.,side*.018*a)]
 out += [(0.,-.010*a,-.006*b),(0.,.010*a,.006*b),(0.,-.018*a,-.020*b),(0.,.018*a,.020*b)]
 if species=='alakazam':out += [(.010*b,0.,-.018*a),(.010*b,0.,.018*a),(.025*b,.025*a,-.025*b),(.025*b,-.025*a,.025*b)]
 else:out += [(.015*b,.025*a,0.),(.020*a,.045*b,0.)]
 return out

def export_psychic(species,model=None):
 if species not in SPECIES:raise ValueError('unknown psychic species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed psychic neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite psychic fox anatomy, planted claws, articulated tail, moustache and held spoons skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed feet and claws; blended bodies and limbs; coherent faces; independent ears, wrists, moustache and held spoons; curved tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/abra_sculpt.py','geometry_preservation':'original meditating Abra and armored Alakazam; tapered fox skulls, fitted ear interiors, closed or focused gaze, curled fingers, three clawed toes, curved banded juvenile tail, adult serrated moustache and held spoons',
    'animation_contract':'identity floor root; fixed insteps, toes and claws; blended torso, neck and limbs; coherent head and fitted gaze; independent ears and wrists; band follows curved juvenile tail; adult moustache blades pivot at muzzle and spoons at finger grips; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'psychic exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('psychic exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('psychic exceeds existing byte limit')
 return blob


def decode_psychic(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_psychic(species):raise ValueError('noncanonical psychic GLB')
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


def read_psychic(path,species):
 with Path(path).open('rb') as f:return decode_psychic(f.read(MAX_BYTES+1),species)
def export_abra(model=None):return export_psychic('abra',model)
def read_abra(path):return read_psychic(path,'abra')
def export_alakazam(model=None):return export_psychic('alakazam',model)
def read_alakazam(path):return read_psychic(path,'alakazam')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_psychic(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
