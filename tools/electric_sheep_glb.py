#!/usr/bin/env python3
"""Standard electric sheep skins with coherent fleece, banded ears and tail beacons."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from sheep_sculpt import sculpture as sheep_sculpture,HEIGHT as SHEEP_HEIGHTS
from ampharos_sculpt import sculpture as ampharos_sculpture,HEIGHT as AMPHAROS_HEIGHT
SPECIES=('mareep','flaaffy','ampharos');HEIGHTS={**SHEEP_HEIGHTS,'ampharos':AMPHAROS_HEIGHT}
def sculpture(species,normalize=True):
 if species not in SPECIES:raise ValueError('unknown electric sheep species')
 return ampharos_sculpture(normalize=normalize) if species=='ampharos' else sheep_sculpture(species,normalize=normalize)
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=3)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'mareep': '058c61bce99e43bc24f2bf17a0cabe8bb62a0a6475fcefcd081d34cf9f580c8d', 'flaaffy': 'eaabe519c0cb6edfa85b9b9ddc5a9b1ca5f5a4fa3bbdedfe948dc57bb68bb8d6', 'ampharos': '19d3138402ccf1b0ffed193b47dea3a25ec684a5f40a1593079e3c5fac04527e'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=3)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

TAILS={'mareep':((0.,.28,-.22),(0.,.30,-.39),(.055,.40,-.53),(.085,.54,-.59),(.09,.67,-.58)),
 'flaaffy':((0.,.28,-.22),(0.,.30,-.39),(.055,.40,-.53),(.085,.54,-.59),(.09,.67,-.58)),
 'ampharos':((0.,.225,-.156),(.014,.245,-.29),(.029,.282,-.42),(.042,.35,-.52),(.052,.433,-.58),(.06,.49,-.601))}
def joints(species):
 raw=[('root',None,(0.,0.,0.)),('torso',0,(0.,.405,-.11) if species=='mareep' else (0.,.32,-.026) if species=='flaaffy' else (0.,.40,-.01)),
  ('neck',1,(0.,.45,.24) if species=='mareep' else (0.,.66,.025) if species=='flaaffy' else (0.,.71,-.01)),
  ('head',2,(0.,.56,.24) if species=='mareep' else (0.,.75,.20) if species=='flaaffy' else (0.,1.015,.033))]
 if species!='ampharos':
  for side in (-1,1):raw.append(('ear_'+('left' if side<0 else 'right'),3,(side*.13,.705 if species=='mareep' else .883,.115)))
 if species!='mareep':
  for side in (-1,1):raw.append(('arm_'+('left' if side<0 else 'right'),1,(side*.16,.55,.03) if species=='flaaffy' else (side*.18,.625,-.01)))
 path=TAILS[species];base=len(raw);raw.extend((('tail_base',1,path[0]),('tail_bend',base,path[2] if species!='ampharos' else path[3]),('tail_beacon',base+1,path[-1])))
 for side in (-1,1):
  if species=='mareep':
   for row,z in enumerate((.095,-.30)):raw.append(('leg_'+('left' if side<0 else 'right')+('_front' if row==0 else '_rear'),0,(side*.19,.18,z)))
  elif species=='flaaffy':raw.append(('leg_'+('left' if side<0 else 'right'),0,(side*.13,.145,0.)))
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={species:joints(species) for species in SPECIES}
BANKS={s:{name:i for i,(name,parent,pivot) in enumerate(js)} for s,js in JOINTS.items()}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def path_fraction(point,path):
 best=(math.inf,0.);lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.
 for a,b,length in zip(path,path[1:],lengths):
  delta=[y-x for x,y in zip(a,b)];t=max(0.,min(1.,sum((p-x)*d for p,x,d in zip(point,a,delta))/(length*length)))
  best=min(best,(math.dist(point,[x+t*d for x,d in zip(a,delta)]),(offset+t*length)/total));offset+=length
 return best[1]
def body_weights(species,y):
 if species=='mareep':a=smoothstep(.45,.48,y);b=smoothstep(.48,.53,y)
 elif species=='flaaffy':a=smoothstep(.53,.65,y);b=smoothstep(.70,.77,y)
 else:a=smoothstep(.65,.83,y);b=smoothstep(.94,1.015,y)
 return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
def skin_weights(species,part,position):
 low,span=authoring_transform(species);b=BANKS[species]
 raw=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position));x,y,z=raw
 if part.startswith(('Foot /','Toe /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  if species=='mareep':
   side,row=part.split()[-2:];owner=b['leg_'+('left' if side=='-1' else 'right')+('_front' if row=='0' else '_rear')]
   a=smoothstep(.04,.32,y);g=.45*smoothstep(.04,.12,y)*(1.-smoothstep(.26,.35,y))
  else:
   owner=b['leg_left' if part.endswith('-1') else 'leg_right'];a=smoothstep(.06,.26,y);g=.45*smoothstep(.06,.10,y)*(1.-smoothstep(.18,.267,y))
  return [0,1,owner,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith(('Crown /','Crown groove /','Eye /','Nostril /','Muzzle /','Forehead /','Horn /')):return [3,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Ear /'):
  side=part.split()[-3];owner=b['ear_left' if side=='-1' else 'ear_right'];a=smoothstep(.13,.26,abs(x))
  return [3,owner,0,0],[1.-a,a,0.,0.]
 if part.startswith(('Arm /','Flipper /')):
  owner=b['arm_left' if part.endswith('-1') else 'arm_right'];a=smoothstep(.16,.32,abs(x)) if species=='flaaffy' else smoothstep(0.,1.,(.625-y)/.265)
  return [1,owner,0,0],[1.-a,a,0.,0.]
 if part.startswith(('Tail / round','Tail / ivory','Tail / ruby','Tail / quiet')):return [b['tail_beacon'],0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail /'):
  t=path_fraction(raw,TAILS[species]);a=smoothstep(0.,.25,t);c=smoothstep(.65,.90,t);d=smoothstep(.25,.70,t)
  return [1,b['tail_base'],b['tail_bend'],b['tail_beacon']],[1.-a,a*(1.-d),a*d*(1.-c),a*d*c]
 if species=='mareep' and part.startswith('Fleece /'):return [1,0,0,0],[1.,0.,0.,0.]
 return body_weights(species,y)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid electric sheep clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);d=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  d=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;d*=strength;out=[]
 for name,parent,pivot in JOINTS[species]:
  if name=='root':angle=(0.,0.,0.)
  elif name=='torso':angle=(0.,.012*a,0.)
  elif name=='neck':angle=(0.,.015*a,0.)
  elif name=='head':angle=(.006*a,.020*a,0.)
  elif name.startswith('ear_'):angle=(.012*d,0.,(-1 if name.endswith('left') else 1)*.025*d)
  elif name.startswith('arm_'):angle=(.015*a,.012*d,(-1 if name.endswith('left') else 1)*.025*a)
  elif name=='tail_base':angle=(0.,.020*a,0.)
  elif name=='tail_bend':angle=(.018*d,.025*d,0.)
  elif name=='tail_beacon':angle=(.020*d,.030*d,0.)
  else:angle=(0.,(-1 if 'right' in name else 1)*.018*a,0.)
  out.append(angle)
 return out
def export_electric_sheep(species,model=None):
 if species not in SPECIES:raise ValueError('unknown electric_sheep species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed electric_sheep neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite electric sheep coherent wool and beacon skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted supports; coherent fleece and crown curls; shared ear/tail band seams; fitted belly and neck rings; delayed beacon; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/sheep_sculpt.py and tools/ampharos_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; planted supports; coherent crown curls, fleece, gaze and horn bands; shared spatial belly/neck ring field; articulated ears, arms, leg middles and banded tail; terminal beacon and highlight share one owner; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'electric_sheep exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_electric_sheep(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_electric_sheep(species):raise ValueError('noncanonical electric_sheep GLB')
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


def read_electric_sheep(path,species):
 with Path(path).open('rb') as f:return decode_electric_sheep(f.read(MAX_BYTES+1),species)
def export_mareep(model=None):return export_electric_sheep('mareep',model)
def export_flaaffy(model=None):return export_electric_sheep('flaaffy',model)
def export_ampharos(model=None):return export_electric_sheep('ampharos',model)
def read_mareep(path):return read_electric_sheep(path,'mareep')
def read_flaaffy(path):return read_electric_sheep(path,'flaaffy')
def read_ampharos(path):return read_electric_sheep(path,'ampharos')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_electric_sheep(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
