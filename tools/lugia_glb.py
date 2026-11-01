#!/usr/bin/env python3
"""Canonical Lugia with five-digit hand wings, a live mandible and planted pads."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from lugia_sculpt import sculpture,HEIGHT
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
REST_DIGEST='00a71d50c07f0307e3638ddd0bfe10bbc8b839f24c4061020d41f4029cc7afa9'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('torso',0,(0.,.46,0.)),('neck',1,(0.,.76,.03)),('head',2,(0.,1.18,.24)),
 ('wing_left',1,(-.17,.72,-.015)),('palm_left',4,(-.72,1.10,-.08)),
 ('wing_right',1,(.17,.72,-.015)),('palm_right',6,(.72,1.10,-.08)),
 ('mandible',3,(0.,1.21,.33)),('tail_root',1,(0.,.314,-.139)),('tail_tip',9,(.26,.153,-.87)),
 ('leg_left',0,(-.21,.20,.05)),('leg_right',0,(.21,.20,.05)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE)
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Toe /','Toes /')):return rigid(0)
 if part.startswith(('Leg /','Haunch /')):
  a=smoothstep(.06,.24,y);g=.18*smoothstep(.06,.13,y)*(1.-smoothstep(.20,.275,y))
  return [0,1,11 if part.endswith('-1') else 12,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith('Wing arm /'):
  a=smoothstep(.52,.79,abs(x));return [4 if x<0 else 6,5 if x<0 else 7,0,0],[1.-a,a,0.,0.]
 if part.startswith(('Wing palm /','Wing digit /','Wing digits /')):
  side=part.split()[-2 if part.startswith('Wing digit /') else -1];return rigid(5 if side=='-1' else 7)
 if part.startswith('Tail fin /'):return rigid(10)
 if part.startswith('Tail /'):
  a=smoothstep(.46,.91,-z);return [9,10,0,0],[1.-a,a,0.,0.]
 if part.startswith(('Jaw / open','Tongue /')):return rigid(8)
 if part.startswith(('Neck /',)):return rigid(2)
 if part.startswith(('Head /','Head crest /','Eye','Mask /','Pupil /','Jaw /','Mouth /')):return rigid(3)
 return rigid(1)
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Lugia clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength;c=a*(.50 if clip=='idle' else 1.);jaw=.020*(max(0.,a) if clip=='attack' else .4*abs(b))
 return [(0.,0.,0.),(0.,.018*a,0.),(.015*a,.020*a,0.),(.020*b,.025*a,0.),
  (.010*b,.020*b,.110*c),(.020*b,.035*b,0.),(.010*b,-.020*b,-.110*c),(.020*b,-.035*b,0.),
  (jaw,0.,0.),(0.,.025*b,0.),(0.,-.035*b,0.),(0.,.012*a,0.),(0.,-.012*a,0.)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])
def pack_parts(model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name=source['part'];key=None;label=None
  if name.startswith('Wing digit /'):
   side=name.split()[-2];key=('digits',side);label='Wing digits / coherent five-finger hand group '+side
  elif name.startswith('Back plate /'):key=('plates',None);label='Back plates / coherent paired navy dorsal paddles'
  elif name.startswith('Toe /'):key=('toes',None);label='Toes / coherent planted guardian digits'
  if key is None:parts.append((source,{'source_order':order}));continue
  if key not in groups:
   joined={'part':label,'positions':[],'normals':[],'indices':[],'base_color':source['base_color']};metadata={'source_parts':[]};groups[key]=(joined,metadata);parts.append((joined,metadata))
  joined,metadata=groups[key];assert joined['base_color']==source['base_color'];start=len(joined['positions'])//3;first=len(joined['indices'])
  metadata['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])})
  joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGEST='8bfe309e8e03304e4547f34346df87add36f0706bed92b5cc7b589b57cf4001b'

def export_lugia(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='lugia' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Lugia anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Lugia five-digit hand wings and open roaring mandible skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'lugia/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'lugia','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'lugia','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'lugia','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 packed_parts=pack_parts(model)
 if geometry_digest({'primitives':[part for part,metadata in packed_parts]})!=PACKED_REST_DIGEST:raise ValueError('unreviewed packed Lugia geometry')
 for part,metadata in packed_parts:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'lugia/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'lugia','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'lugia.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted pads and blunt toes; coherent five-digit palms; supported bending wing arms; articulated neck, gaze and open mandible; yawing tail and paired fins; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'lugia','source_sha256':REST_DIGEST,
    'source_part_ranges':[metadata for part,metadata in packed_parts],'packed_sha256':PACKED_REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/lugia_sculpt.py','geometry_preservation':'25 retained core meshes with unchanged topology, normals and colors; normalization drift below 3.75e-7; reduced paddle and ellipsoid sampling; locally refined fitted belly',
    'animation_contract':'identity floor root; planted pads and toes; supported haunches and ankles; coherent belly and dorsal plates; neck and head articulation; independently carried five-digit palms; opening mandible with coherent tongue; two yawing tail joints and paired navy fins; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Lugia exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Lugia exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Lugia exceeds existing byte limit')
 return blob


def decode_lugia(blob):
 if len(blob)>MAX_BYTES or blob!=export_lugia():raise ValueError('noncanonical Lugia GLB')
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
 return {'name':'lugia','version':1,'coordinate_system':COORDINATES,'primitives':[p for order,p in sorted(parts,key=lambda pair:pair[0])]}

def read_lugia(path):
 with Path(path).open('rb') as f:return decode_lugia(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_lugia();(a.out/'lugia.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
