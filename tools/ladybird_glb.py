#!/usr/bin/env python3
"""Standard Ledyba/Ledian skins with fitted faces and separately articulated wing cases."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from ladybird_sculpt import sculpture,HEIGHTS,SPECIES
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

REST_DIGESTS={'ledyba': '38b13162e17a6677a6d3c6cc6561f870210a342cbb00e226a8688ee5d5811e2a', 'ledian': 'd8b776f761bf9f6287d975cc158c3ebf2b9f06b2b67479a98ef19e881f202031'}
PARTS={species:tuple(p['part'] for p in sculpture(species)['primitives']) for species in SPECIES}
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def joints(species):
 evolved=species=='ledian'
 raw=[('root',None,(0.,0.,0.)),('abdomen',0,(0.,.48 if evolved else .44,.015)),
 ('thorax',1,(0.,.64 if evolved else .62,.028)),('head',2,(0.,.87 if evolved else .85,.065 if evolved else .04)),
 ('antenna_left',3,(-.076,1.013,.024)),('antenna_right',3,(.076,1.013,.024))]
 for row,(y,x,z) in enumerate(((.655,.132,.028),(.503,.132,.028)) if evolved else ((.659,.187,.047),(.465,.187,.047))):
  for side in (-1,1):raw.append((('upper_arm_' if row==0 else 'lower_arm_')+('left' if side<0 else 'right'),2 if row==0 else 1,(side*x,y,z)))
 for side in (-1,1):raw.append(('leg_'+('left' if side<0 else 'right'),1,(side*(.090 if evolved else .162),.300 if evolved else .242,.006 if evolved else .030)))
 for side in (-1,1):raw.append(('case_'+('left' if side<0 else 'right'),1,(side*(.040 if evolved else .055),.645 if evolved else .660,-.060)))
 if evolved:
  for side in (-1,1):raw.append(('wing_'+('left' if side<0 else 'right'),2,(side*.077,.659,-.101)))
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={species:joints(species) for species in SPECIES}
CLIPS=(('idle',4.,85),('attack',1.,65),('hit',1.,65))

def skin_weights(species,part,position):
 x,y,z=position;left='-1' in part.split();side=0 if left else 1
 if part.startswith('Foot /'):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  low=.078 if species=='ledian' else .063;high=.300 if species=='ledian' else .242
  blend=smoothstep(normalized(species,(0.,low,0.))[1],normalized(species,(0.,high,0.))[1],y)
  return [0,10+side,0,0],[1.-blend,blend,0.,0.]
 if part.startswith(('Arm /','Hand /','Glove seam /')):
  owner=(6 if part.split()[-1]=='0' else 8)+side;return [owner,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Head /','Head spot /','Face /','Eye /','Pupil /','Glint /','Mouth /')):return [3,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Antenna /'):return [4+side,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Shell /','Shell spot /')):return [12+side,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Wing /'):return [14+side,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Abdomen /','Breast seam /')):
  ranges=((.26,.38,0,1),(.56,.69,1,2)) if species=='ledian' else ((.16,.32,0,1),(.52,.68,1,2))
  for a,b,j,k in ranges:
   aa=normalized(species,(0.,a,0.))[1];bb=normalized(species,(0.,b,0.))[1]
   if y<aa:return [j,0,0,0],[1.,0.,0.,0.]
   if y<=bb:
    blend=smoothstep(aa,bb,y);return [j,k,0,0],[1.-blend,blend,0.,0.]
  return [2,0,0,0],[1.,0.,0.,0.]
 raise ValueError('unowned ladybird anatomy: '+part)

def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid ladybird clip/time')
 count=len(JOINTS[species])
 if t in (0.,1.):return [(0.,0.,0.)]*count
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t);lag=math.sin(math.tau*t-.45)+math.sin(.45)
 if clip=='idle':body=.004*breath;head=.012*breath;turn=.013*wave;arm=.017*wave;lower=.013*lag;feeler=.020*lag;case=.025*breath;wing=.018*wave;leg=.005*breath
 else:
  early=envelope(t,((0.,0.),(.18,1.),(.40,.68),(.67,.10),(1.,0.)))
  late=envelope(t,((0.,0.),(.28,1.),(.53,.60),(.79,-.08),(1.,0.)));sign=1. if clip=='attack' else -.65
  body=sign*.010*early;head=sign*.025*early;turn=sign*.018*late;arm=sign*.050*early;lower=sign*.038*late;feeler=sign*.038*late;case=.014*max(0.,late)*(1. if clip=='attack' else .65);wing=sign*.035*late;leg=sign*.010*early
 result=[(0.,0.,0.),(body,0.,0.),(.8*body,0.,0.),(head,turn,0.),
 (feeler,0.,-.25*feeler),(feeler,0.,.25*feeler),
 (arm,0.,-.2*arm),(arm,0.,.2*arm),(lower,0.,-.2*lower),(lower,0.,.2*lower),
 (leg,0.,0.),(leg,0.,0.),(0.,case,-.2*case),(0.,-case,.2*case)]
 if species=='ledian':result.extend(((wing,0.,-.25*wing),(wing,0.,.25*wing)))
 return result

def export_ladybird(species,model=None):
 if species not in SPECIES:raise ValueError('unknown ladybird species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed ladybird neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct ladybird anatomy and standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'body and head brace; planted feet; articulated fists antennae and wing cases; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHTS[species],'body_reference_height':HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/ladybird_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; two planted feet; coherent fitted head gloves and shell spots; six articulated limbs antennae wing cases and evolved flight membranes; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'ladybird exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_ladybird(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_ladybird(species):raise ValueError('noncanonical ladybird GLB')
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


def read_ladybird(path,species):
 with Path(path).open('rb') as f:return decode_ladybird(f.read(MAX_BYTES+1),species)
def export_ledyba(model=None):return export_ladybird('ledyba',model)
def export_ledian(model=None):return export_ladybird('ledian',model)
def read_ledyba(path):return read_ladybird(path,'ledyba')
def read_ledian(path):return read_ladybird(path,'ledian')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_ladybird(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
