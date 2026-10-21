#!/usr/bin/env python3
"""Canonical green mantis with curved scythes, fitted wing membranes and planted claws."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from scyther_sculpt import sculpture,HEIGHT
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
MAX_BYTES=1024*1024
SOURCE=sculpture();RAW=sculpture(normalize=False)
LOW=min(v for p in RAW['primitives'] for v in p['positions'][1::3])
SCALE=HEIGHT/(max(v for p in RAW['primitives'] for v in p['positions'][1::3])-LOW)
PARTS=tuple(p['part'] for p in SOURCE['primitives'])
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
REST_DIGEST='e7c84bdfd2b8b6f63bd9281c1fd0a717a12a8ee4cc208d409e6b4707dc87576b'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('thorax',0,(0.,.78,.010)),('abdomen',1,(0.,.62,-.025)),
 ('neck',1,(0.,.93,.028)),('head',3,(0.,1.015,.072)),
 ('arm_left',1,(-.17,.84,.03)),('scythe_left',5,(-.20,.81,.27)),
 ('arm_right',1,(.17,.84,.03)),('scythe_right',7,(.20,.81,.27)),
 ('wing_left',1,(-.071,.85,-.055)),('wing_right',1,(.071,.85,-.055)),
 ('leg_left',2,(-.142,.453,-.040)),('leg_right',2,(.142,.453,-.040)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE)
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 side=part.split()[-2] if part.startswith(('Wing /','Wing case /','Wing vein /','Foot claw /')) else part.split()[-1];left=side=='-1'
 if part.startswith(('Greave /','Foot /','Foot claw /')):return rigid(0)
 if part.startswith('Shank /'):
  a=smoothstep(.245,.37,y)
  return [0,11 if left else 12,0,0],[1.-a,a,0.,0.]
 if part.startswith(('Haunch /','Haunch thorn /')):return rigid(11 if left else 12)
 if part.startswith('Upper arm /'):return rigid(5 if left else 7)
 if part.startswith(('Scythe /','Scythe rim /')):return rigid(6 if left else 8)
 if part.startswith(('Wing /','Wing case /','Wing vein /')):return rigid(9 if left else 10)
 if part.startswith(('Head /','Head marking /','Crest /','Eye','Pupil /','Mouth /','Nostril /')):return rigid(4)
 if part.startswith('Neck /'):
  a=smoothstep(.93,1.01,y);b=smoothstep(.98,1.02,y)
  return [1,3,4,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith(('Abdomen /','Abdomen seam /')):return rigid(2)
 return rigid(1)
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Scyther clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.;flutter=math.sin(5.*math.tau*t);slash=.025*b
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.;flutter=b*strength;slash=(.18 if clip=='attack' else -.035)*b
 a*=strength;b*=strength
 return [(0.,0.,0.),(0.,.012*a,0.),(0.,-.016*b,0.),(.010*a,.012*b,0.),(.016*b,.022*a,.008*b),
  (.018*a,.020*b,-.040*b),(-slash,.025*a,-.012*b),
  (.018*a,-.020*b,.040*b),(-slash,-.025*a,.012*b),
  (.015*a,.085*flutter,.018*b),(.015*a,-.085*flutter,-.018*b),
  (0.,.020*a,0.),(0.,-.020*a,0.)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])

def pack_parts(model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name=source['part'];key=None;label=None
  # Shin armor and pads retain separate roles; nail groups stay within one foot.
  if name.startswith(('Greave /','Foot /')):pass
  elif name.startswith('Foot claw /'):
   side=name.split()[-2];key=('ground_pale',side);label='Foot claw / coherent ivory mantis nails '+side
  elif name.startswith('Crest /'):key=('crest',tuple(source['base_color']));label='Crest / coherent mantis cranial points '+str(len(groups))
  elif name.startswith(('Eye socket /','Nostril /','Mouth /')):key=('face_dark',);label='Head marking / coherent dark mantis brow muzzle and pores'
  elif name.startswith(('Wing /','Wing case /','Wing vein /')):
   side=name.split()[-2];kind='case' if name.startswith('Wing case /') else 'vein' if name.startswith('Wing vein /') else 'wing'
   key=(kind,side);prefix={'case':'Wing case /','vein':'Wing vein /','wing':'Wing /'}[kind];label=prefix+' coherent paired mantis leaves '+side+' 0'
  elif name.startswith('Abdomen seam /'):key=('seams',);label='Abdomen seam / coherent three mantis shell divisions'
  if key is None:parts.append((source,{'source_order':order}));continue
  if key not in groups:
   joined={'part':label,'positions':[],'normals':[],'indices':[],'base_color':source['base_color']};metadata={'source_parts':[]};groups[key]=(joined,metadata);parts.append((joined,metadata))
  joined,metadata=groups[key];assert joined['base_color']==source['base_color'];start=len(joined['positions'])//3;first=len(joined['indices'])
  metadata['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])})
  joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGEST='3636af80cadc38d3c0c3de404cbbc0448e6d897f82d590f080f7a8ceb7ec1472'

def export_scyther(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='scyther' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Scyther anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Scyther green mantis, curved scythes and four-wing skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'scyther/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'scyther','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'scyther','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'scyther','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 packed_parts=pack_parts(model)
 if geometry_digest({'primitives':[part for part,metadata in packed_parts]})!=PACKED_REST_DIGEST:raise ValueError('unreviewed packed Scyther geometry')
 for part,metadata in packed_parts:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'scyther/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'scyther','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'scyther.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted clawed pads and green greaves; fitted ivory gaze and muzzle seam; articulated curved scythes with coherent green spines; four fitted wing membranes on paired carriers; source cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'scyther','source_sha256':REST_DIGEST,
    'source_part_ranges':[metadata for part,metadata in packed_parts],'packed_sha256':PACKED_REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/scyther_sculpt.py','geometry_preservation':'original rounded green thorax, pale waist, segmented tapered abdomen, angular green head and ivory gaze, curved ivory scythes with fitted green spines, four pale veined membranes and planted clawed greaves',
    'animation_contract':'identity floor root; fixed greaves pads and ivory nails; blended pale shanks and independent armored coxae; separate thorax abdomen neck head; coherent fitted gaze muzzle pores and cranial points; articulated elbows and curved scythes with fitted spines; four fitted membranes and ribs on paired flutter carriers; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Scyther exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Scyther exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Scyther exceeds existing byte limit')
 return blob


def decode_scyther(blob):
 if len(blob)>MAX_BYTES or blob!=export_scyther():raise ValueError('noncanonical Scyther GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 parts=[]
 for p,extra in zip(doc['meshes'][0]['primitives'],doc['extras']['source_part_ranges']):
  positions=values(p['attributes']['POSITION']);normals=values(p['attributes']['NORMAL']);indices=values(p['indices']);color=doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
  sources=extra.get('source_parts',[{'part':p['extras']['part'],'order':extra.get('source_order'),'vertex_start':0,'vertex_count':len(positions)//3,'index_start':0,'index_count':len(indices)}])
  for src in sources:
   a=src['vertex_start'];b=a+src['vertex_count'];i=src['index_start'];j=i+src['index_count']
   parts.append((src['order'],{'part':src['part'],'positions':positions[3*a:3*b],'normals':normals[3*a:3*b],'indices':[k-a for k in indices[i:j]],'base_color':color}))
 return {'name':'scyther','version':1,'coordinate_system':COORDINATES,'primitives':[p for order,p in sorted(parts,key=lambda pair:pair[0])]}

def read_scyther(path):
 with Path(path).open('rb') as f:return decode_scyther(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_scyther();(a.out/'scyther.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
