#!/usr/bin/env python3
"""Canonical continuous Caterpie and Weedle bodies with planted contact, coherent faces and articulated tips."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from larva_sculpt import sculpture,SPECIES,HEIGHTS,SPINE,PATH
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'caterpie': '527fcc82fab88435eb538b1b93614b03f896566ecefafc9d0f6e05e5cb419931', 'weedle': 'ef69247a874d0f1cec8f314d3885cd93ddaa5356fd45ce850dda1f75a0a162ea'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def joints(species):
 raw=[('root',None,(0.,0.,0.))]+[('segment_'+str(j),0,p) for j,p in enumerate(SPINE)]+[('head',8,(0.,.570,.132))]
 if species=='caterpie':raw += [('horn_left',9,(0.,.786,.220)),('horn_right',9,(0.,.786,.220))]
 else:raw += [('head_sting',9,(0.,.733,.105)),('tail_sting',1,(0.,.190,-.785))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def path_fraction(point):
 best=(math.inf,0.)
 for j,(a,b) in enumerate(zip(PATH,PATH[1:])):
  delta=[y-x for x,y in zip(a,b)];square=sum(d*d for d in delta);t=max(0.,min(1.,sum((v-x)*d for v,x,d in zip(point,a,delta))/square));distance=sum((v-x-t*d)**2 for v,x,d in zip(point,a,delta))
  if distance<best[0]:best=(distance,(j+t)/(len(PATH)-1))
 return best[1]
def skin_weights(species,part,position):
 low,span=authoring_transform(species);raw=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position));x,y,z=raw
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Proleg / planted'):return rigid(0)
 if part.startswith(('Horn branch /','Horn tip /')):return rigid(10 if part.endswith('-1') else 11)
 if part.startswith('Horn /'):return rigid(10)
 if part.startswith('Tail sting /'):return rigid(11)
 if part.startswith('Tail tip /'):return rigid(1)
 if part.startswith(('Head /','Eye','Glint /','Muzzle','Nose','Horn stem /')):return rigid(9)
 t=path_fraction(raw)*7;j=min(6,int(t));f=smoothstep(0.,1.,t-j);anchor=smoothstep(low+.018,low+.085,y)
 return [0,1+j,2+j,0],[1.-anchor,anchor*(1.-f),anchor*f,0.]
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid larva clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.)]+[(0.,.015*(a*math.cos(j*.43)+b*math.sin(j*.43)),.006*a*math.cos(j*.51)) for j in range(8)]+[(.012*a,.025*b,.008*b)]
 return out+[(.008*a,0.,-.020*b),(.008*a,0.,.020*b)] if species=='caterpie' else out+[(.012*a,.015*b,0.),(.015*b,.025*a,0.)]

def export_larva(species,model=None):
 if species not in SPECIES:raise ValueError('unknown larva species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed larva neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite connected larva bodies, coherent gaze, forked antenna and ivory stings skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted lower belly or pink prolegs; continuous segmented crawling body; coherent fitted gaze; independent red fork or ivory head and tail stings; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/larva_sculpt.py','geometry_preservation':'original connected annulated larva bodies, continuous cream underside with closed material seam, five hollow lateral circles, coherent gaze, red fork or paired ivory stings',
    'animation_contract':'identity floor root; fixed contact points and pink pads; eight blended body carriers; coherent head and gaze; connected red fork or ivory head/tail stings; matched cream/green body seam; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'larva exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('larva exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('larva exceeds existing byte limit')
 return blob


def decode_larva(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_larva(species):raise ValueError('noncanonical larva GLB')
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


def read_larva(path,species):
 with Path(path).open('rb') as f:return decode_larva(f.read(MAX_BYTES+1),species)
def export_caterpie(model=None):return export_larva('caterpie',model)
def read_caterpie(path):return read_larva(path,'caterpie')
def export_weedle(model=None):return export_larva('weedle',model)
def read_weedle(path):return read_larva(path,'weedle')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_larva(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
