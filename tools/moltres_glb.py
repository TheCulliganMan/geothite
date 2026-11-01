#!/usr/bin/env python3
"""Canonical Moltres with planted talons, wing strokes and coherent articulated flame banks."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from moltres_sculpt import sculpture,HEIGHT
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
REST_DIGEST='dc2e868b13c6cc3334e76cd38c4089930ff2e2fec9c4935b394793eb6f0bdf4b'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('torso',0,(0.,.40,0.)),('neck',1,(0.,.60,.02)),('head',2,(0.,.84,.034)),
 ('wing_left',1,(-.12,.61,-.05)),('wing_right',1,(.12,.61,-.05)),
 ('fire_left',4,(-.12,.62,-.08)),('fire_right',5,(.12,.62,-.08)),
 ('crown',3,(0.,1.0,.057)),('tail',1,(0.,.385,-.14)),
 ('leg_left',0,(-.10,.16,.02)),('leg_right',0,(.10,.16,.02)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE)
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Toe /','Talon /')):return rigid(0)
 if part.startswith('Leg /'):
  a=smoothstep(.06,.24,y);g=.18*smoothstep(.06,.13,y)*(1.-smoothstep(.20,.275,y))
  return [0,1,10 if part.endswith('-1') else 11,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith('Wing /'):return rigid(4 if part.endswith('-1') else 5)
 if part.startswith(('Wing fire /','Wing glow /','Wing heart /')):
  side=part.split()[-1] if ' group ' in part else part.split()[-3 if part.startswith('Wing heart /') else -2 if part.startswith('Wing glow /') else -1]
  return rigid(6 if side=='-1' else 7)
 if part.startswith('Crown'):return rigid(8)
 if part.startswith('Tail'):return rigid(9)
 if part.startswith('Neck /'):return rigid(2)
 if part.startswith(('Head /','Eye','Pupil /','Bill')):return rigid(3)
 return rigid(1)
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Moltres clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength;c=a*(.50 if clip=='idle' else 1.)
 return [(0.,0.,0.),(0.,.018*a,0.),(.020*a,.020*a,0.),(.035*b,.025*a,0.),
  (.010*b,.025*b,.140*c),(.010*b,-.025*b,-.140*c),
  (0.,-.010*b,0.),(0.,.010*b,0.),(.012*b,.025*b,.015*b),(0.,.055*b,0.),
  (0.,.012*a,0.),(0.,-.012*a,0.)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])
def pack_parts(model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name=source['part'];key=None;label=None
  if name.startswith('Wing heart /'):
   side=name.split()[-3];key=('wing_heart',side);label='Wing heart / coherent hot flame pockets group '+side
  elif name.startswith('Wing glow /'):
   side=name.split()[-2];key=('wing_glow',side);label='Wing glow / coherent broad orange cores group '+side
  else:
   for prefix,label0 in (('Crown glow /','Crown glow / coherent orange core faces'),('Crown heart /','Crown heart / coherent gold core faces'),('Tail glow /','Tail glow / coherent orange flame faces'),('Tail heart /','Tail heart / coherent gold flame faces')):
    if name.startswith(prefix):key=(prefix,None);label=label0;break
  if key is None:
   parts.append((source,{'source_order':order}));continue
  if key not in groups:
   joined={'part':label,'positions':[],'normals':[],'indices':[],'base_color':source['base_color']};metadata={'source_parts':[]};groups[key]=(joined,metadata);parts.append((joined,metadata))
  joined,metadata=groups[key];start=len(joined['positions'])//3;first=len(joined['indices'])
  metadata['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])})
  joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGEST='bc2c5af3c85515bb49c4773acaf16f96232104ac36d08b59681ba4eba064bc33'

def export_moltres(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='moltres' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Moltres anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Moltres wing strokes and coherent flame banks skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'moltres/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'moltres','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'moltres','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'moltres','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 packed_parts=pack_parts(model)
 if geometry_digest({'primitives':[part for part,metadata in packed_parts]})!=PACKED_REST_DIGEST:raise ValueError('unreviewed packed Moltres geometry')
 for part,metadata in packed_parts:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'moltres/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'moltres','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'moltres.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted feet and talons; supported shins; articulated neck and gaze; gold leading wings with coherent delayed fire banks; coherent crown and trailing flame plume; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'moltres','source_sha256':REST_DIGEST,
    'source_part_ranges':[metadata for part,metadata in packed_parts],'packed_sha256':PACKED_REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/moltres_sculpt.py','geometry_preservation':'unchanged gold body, neck, head, bill and full grounded digits; original flame outlines with reduced interior subdivisions and shared smooth corners',
    'animation_contract':'identity floor root; fixed feet, digits and talons; supported shins; rigid gold breast, neck and head owners; gold wing strokes; coherent fire banks with all fitted warm/hot layers; independently seated crown and tail flames; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Moltres exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Moltres exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Moltres exceeds existing byte limit')
 return blob


def decode_moltres(blob):
 if len(blob)>MAX_BYTES or blob!=export_moltres():raise ValueError('noncanonical Moltres GLB')
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
 return {'name':'moltres','version':1,'coordinate_system':COORDINATES,'primitives':[p for order,p in sorted(parts,key=lambda pair:pair[0])]}

def read_moltres(path):
 with Path(path).open('rb') as f:return decode_moltres(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_moltres();(a.out/'moltres.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
