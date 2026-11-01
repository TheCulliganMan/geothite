#!/usr/bin/env python3
"""Swinub and Piloswine planted hooves, coherent fur and articulated sniffing faces."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from fur_pig_sculpt import sculpture,SPECIES,HEIGHT
HEIGHTS={s:HEIGHT for s in SPECIES}
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'swinub': '99b43c8174c27d71d246f211d8ca6a0ce2764cb9d6e60cc2153b1fe25ffc0591', 'piloswine': 'b6b3ef74640643f89b461d55f276157aad0cfb2f7a39d17214b9a9eb8d3ea0b9'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def joints(species):
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.45 if species=='piloswine' else .20,-.07)),('head',1,(0.,.40 if species=='piloswine' else .20,.18 if species=='piloswine' else .25))]
 if species=='piloswine':raw += [('ear_left',1,(-.30,.60,.10)),('ear_right',1,(.30,.60,.10))]
 else:raw += [('fringe',1,(0.,.143,-.416))]
 for side in (-1,1):
  for row,z in enumerate((.17,-.30)):raw.append(('leg_'+('left' if side<0 else 'right')+'_'+('front' if row==0 else 'rear'),0,(side*.20,.095,z)))
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
BANKS={s:{name:i for i,(name,parent,pivot) in enumerate(js)} for s,js in JOINTS.items()}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 low,span=authoring_transform(species);bank=BANKS[species]
 x,y,z=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position))
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Hoof /'):return rigid(0)
 if part.startswith(('Snout /','Nostril /','Tusk /','Eye /','Fur crease /')):return rigid(2)
 if part.startswith('Ear /'):return rigid(bank['ear_left' if part.endswith('-1') else 'ear_right'])
 if part.startswith('Fur fringe /'):return rigid(bank['fringe'])
 a=smoothstep(.055,.18,z)
 if part.startswith('Leg /'):
  side,row=map(int,part.split()[-2:]);owner=bank['leg_'+('left' if side<0 else 'right')+'_'+('front' if row==0 else 'rear')]
  # A shank follows its centerline: avoid U8 weight steps around tiny pole rings.
  a=smoothstep(.055,.18,.17 if row==0 else -.30)
  b=smoothstep(.025,.17,y);g=.16*smoothstep(.035,.08,y)*(1.-smoothstep(.11,.17,y))
  return [0,1,2,owner],[(1.-b)*(1.-g),b*(1.-a)*(1.-g),b*a*(1.-g),g]
 # The coat and its fitted markings use one spatial body/head field.
 return [1,2,0,0],[1.-a,a,0.,0.]
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid fur pig clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);d=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  d=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;d*=strength;out=[]
 for name,parent,pivot in JOINTS[species]:
  if name=='root':angle=(0.,0.,0.)
  elif name=='body':angle=(0.,.025*a,0.)
  elif name=='head':angle=(.012*a,.025*d,0.)
  elif name.startswith('ear_'):angle=(.015*d,0.,(-1 if name.endswith('left') else 1)*.025*d)
  elif name=='fringe':angle=(.025*d,.035*a,0.)
  else:angle=(0.,(-1 if 'right' in name else 1)*.020*a,0.)
  out.append(angle)
 return out
def export_fur_pig(species,model=None):
 if species not in SPECIES:raise ValueError('unknown fur pig species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed fur pig neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite fur pig planted hooves and coherent sniffing coat skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted hooves; coherent fitted coat; sniffing head, articulated ears and rear wisps; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/fur_pig_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; fixed hooves; shared spatial coat and marking field; coherent snout and tusks; articulated ears and wisps; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'fur pig exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_fur_pig(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_fur_pig(species):raise ValueError('noncanonical fur pig GLB')
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


def read_fur_pig(path,species):
 with Path(path).open('rb') as f:return decode_fur_pig(f.read(MAX_BYTES+1),species)
def export_swinub(model=None):return export_fur_pig('swinub',model)
def read_swinub(path):return read_fur_pig(path,'swinub')
def export_piloswine(model=None):return export_fur_pig('piloswine',model)
def read_piloswine(path):return read_fur_pig(path,'piloswine')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_fur_pig(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
