#!/usr/bin/env python3
"""Canonical crab shells, planted walking toes and independently closing orange pincers."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from crab_sculpt import sculpture,SPECIES,HEIGHT
HEIGHTS={s:HEIGHT for s in SPECIES}
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'krabby': '5f8f12d336b23cecec7ad64a1ef4ab89f387324289654f1fdbc722df900ff5ef', 'kingler': '77ef337fdce8469cd785876b536b3c92953ebd6a1b026f7b12001c2a592cfc36'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def joints(species):
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.365,0.))]
 for side,label,arm in ((-1,'left',2),(1,'right',5)):
  size=1.85 if species=='kingler' and side<0 else (.88 if species=='kingler' else .96)
  raw += [('arm_'+label,1,(side*.234,.347,.092)),('wrist_'+label,arm,(side*.467,.573,.145)),
   ('jaw_'+label,arm+1,(side*(.485+.006*size),.584-.020*size,.153+.010*size))]
 raw += [('leg_'+str(side)+'_'+str(j),0,(side*.373,.238,z-.025)) for side in (-1,1) for j,z in enumerate((-.060,.065))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 low,span=authoring_transform(species);x,y,z=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position))
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Foot /'):return rigid(0)
 if part.startswith(('Walking leg /','Leg knee /')):
  side,j=map(int,part.split()[-2:]);owner=8+(0 if side<0 else 2)+j
  a=smoothstep(.055,.275,y);b=.01+.64*smoothstep(.07,.16,y)*(1.-smoothstep(.24,.29,y))
  return [owner,0,1,0],[b,(1.-a)*(1.-b),a*(1.-b),0.]
 if part.startswith(('Pincer arm /','Wrist /','Claw palm /','Claw jaw /')):
  side=int(part.split()[-1]);arm=2 if side<0 else 5
  if part.startswith('Wrist /') or part.startswith('Claw palm /'):return rigid(arm+1)
  if part.startswith('Claw jaw /'):return rigid(arm+2)
  a=smoothstep(.355,.410,y);b=smoothstep(.475,.550,y)
  return [1,arm,arm+1,0],[(1.-a)*(1.-b),a*(1.-b),b,0.]
 return rigid(1)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid crab clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*12
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.012*a,0.)]
 for side in (-1,1):
  out += [(.012*a,side*.024*b,-side*.022*a),(.009*a,side*.035*b,-side*.012*b),(0.,0.,side*.095*b)]
 out += [(0.,side*.018*a,side*.008*b) for side in (-1,1) for j in range(2)]
 return out

def export_crab(species,model=None):
 if species not in SPECIES:raise ValueError('unknown crab species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed crab neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite crab broad pincers, four planted legs and independent jaws skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'four fixed walking toes; blended braced legs and arms; independent wrists and closing pincer jaws; coherent shell and gaze; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/crab_sculpt.py','geometry_preservation':'broad orange and ivory crab shell; four braced walking legs; tapered hooked pincers; Kingler has a dominant larger claw and five crown points',
    'animation_contract':'identity floor root; four fixed toes and blended braced walking legs; yaw-only shell and coherent eyes pupils cheek peaks and mouth; blended arms; separate wrist and jaw carriers; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'crab exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('crab exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('crab exceeds existing byte limit')
 return blob


def decode_crab(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_crab(species):raise ValueError('noncanonical crab GLB')
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


def read_crab(path,species):
 with Path(path).open('rb') as f:return decode_crab(f.read(MAX_BYTES+1),species)
def export_krabby(model=None):return export_crab('krabby',model)
def read_krabby(path):return read_crab(path,'krabby')
def export_kingler(model=None):return export_crab('kingler',model)
def read_kingler(path):return read_crab(path,'kingler')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_crab(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
