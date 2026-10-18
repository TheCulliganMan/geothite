#!/usr/bin/env python3
"""Canonical grounded flowing slimes with coherent faces and independent hands or lobes."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from slime_sculpt import sculpture,SPECIES,HEIGHT
HEIGHTS=HEIGHT
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=3)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'grimer': '01ccaddd323bf25f60a019cbc0642316ec52af5c7cd650ecc625d54d607c3a03', 'muk': '373608073b4024110aaaf4e23d345ca1c07c2fd6a2725f72fe985893224ca8c3', 'ditto': '1a3fc7b87714f7c4c5b82db35cecc4fb7bc3d3679f52f0ccf2336727626327e8'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=3)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def joints(species):
 from slime_sculpt import arm_layout,digit_root
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.260,.0)),('head',1,(0.,.440,.040))]
 if species=='ditto':raw += [('lobe_left',2,(-.170,.340,.0)),('lobe_right',2,(.170,.340,.0))]
 else:
  for side,label,arm in ((-1,'left',3),(1,'right',5)):
   points,radii,center,size=arm_layout(species,side)
   raw += [('arm_'+label,1,points[0]),('hand_'+label,arm,center)]
  for side,label,hand in ((-1,'left',4),(1,'right',6)):
   raw += [('finger_'+label+'_'+str(j),hand,digit_root(species,side,j)) for j in range(3)]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def body_weights(species,x,y):
 a=smoothstep(.075,.28,y)
 if species=='ditto':
  head=smoothstep(.25,.35,y);lobe=smoothstep(.12,.25,abs(x))*smoothstep(.25,.39,y)
  return [0,1,2,3 if x<0 else 4],[1.-a,a*(1.-head),a*head*(1.-lobe),a*head*lobe]
 head=smoothstep(.30,.43,y)
 return [0,1,2,0],[1.-a,a*(1.-head),a*head,0.]
def skin_weights(species,part,position):
 from slime_sculpt import arm_layout,digit_root
 low,span=authoring_transform(species);x,y,z=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position))
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Body /','Body fold /','Body sheen /')):return body_weights(species,x,y)
 if part.startswith(('Arm /','Hand /','Finger /')):
  side=int(part.split()[-2] if part.startswith('Finger /') else part.split()[-1]);arm=3 if side<0 else 5;hand=arm+1
  if part.startswith('Hand /'):return rigid(hand)
  if part.startswith('Finger /'):
   j=int(part.split()[-1]);root=digit_root(species,side,j);blend=smoothstep(root[2]+.020,root[2]+.055,z)
   return [hand,7+(0 if side<0 else 3)+j,0,0],[1.-blend,blend,0.,0.]
  points,_,center,_=arm_layout(species,side);root=abs(points[0][0]);wrist=abs(center[0]);a=smoothstep(root+.025,(root+wrist)*.5,abs(x));b=smoothstep(wrist-.065,wrist+.005,abs(x));head=smoothstep(.30,.43,y)
  return [1,2,arm,hand],[(1.-a)*(1.-b)*(1.-head),(1.-a)*(1.-b)*head,a*(1.-b),b]
 return rigid(2)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid slime clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 if species=='ditto':return [(0.,0.,0.),(0.,.012*b,.012*a),(.018*a,.015*b,0.),(.010*a,0.,-.055*b),(.010*a,0.,.055*b)]
 out=[(0.,0.,0.),(0.,.012*b,.009*a),(.015*a,.012*b,.008*a)]
 for side in (-1,1):out += [(.020*a,side*.018*b,-side*.025*a),(.020*b,side*.014*a,side*.012*b)]
 for side in (-1,1):
  for j in range(3):out += [(-.035*b-.007*a*(j-1),side*.012*a,0.)]
 return out

def export_slime(species,model=None):
 if species not in SPECIES:raise ValueError('unknown slime species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed slime neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite slime continuous flowing bodies, grounded puddles and independent soft appendages skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed broad puddle soles; blended flowing torsos and coherent faces; independent hands and fingers or soft lateral lobes; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/slime_sculpt.py','geometry_preservation':'original continuous flowing sludge and transform contours; true annular lips, fitted faces, rooted palms and digits; distinct asymmetric body profiles',
    'animation_contract':'identity floor root and broad fixed underside; blended flowing body and coherent head; Grimer and Muk blend arms into separate wrists and six finger carriers; Ditto has two independent lateral lobes with matching sheen; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'slime exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('slime exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('slime exceeds existing byte limit')
 return blob


def decode_slime(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_slime(species):raise ValueError('noncanonical slime GLB')
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


def read_slime(path,species):
 with Path(path).open('rb') as f:return decode_slime(f.read(MAX_BYTES+1),species)
def export_grimer(model=None):return export_slime('grimer',model)
def read_grimer(path):return read_slime(path,'grimer')
def export_muk(model=None):return export_slime('muk',model)
def read_muk(path):return read_slime(path,'muk')
def export_ditto(model=None):return export_slime('ditto',model)
def read_ditto(path):return read_slime(path,'ditto')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_slime(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
