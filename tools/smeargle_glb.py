#!/usr/bin/env python3
"""Canonical Smeargle skin with planted paws, a coherent held brush-tail and articulated painter beret."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from smeargle_sculpt import sculpture,HEIGHT
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
REST_DIGEST='41339016b1387248e3407fbba1d1412e7fee742f47744f29bf3f6dff2211a753'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('torso',0,(0.,.42,0.)),('neck',1,(0.,.646,.002)),('head',2,(0.,.70,.12)),
 ('ear_left',3,(-.15,.87,.012)),('ear_right',3,(.15,.87,.012)),('beret',3,(0.,.957,-.02)),
 ('arm_left',1,(-.10,.565,0.)),('wrist_left',7,(-.235,.43,.07)),
 ('arm_right',1,(.10,.588,0.)),('brush_hold',9,(.285,.61,.22)),
 ('leg_left',0,(-.11,.20,.010)),('leg_right',0,(.11,.20,.010)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE)
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Toe groove /','Cuff /')):return rigid(0)
 if part.startswith('Leg /'):
  a=smoothstep(.16,.285,y);g=.18*smoothstep(.16,.20,y)*(1.-smoothstep(.25,.30,y))
  return [0,1,11 if part.endswith('-1') else 12,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith('Ear /'):return rigid(4 if part.endswith('-1') else 5)
 if part.startswith('Beret /'):return rigid(6)
 if part.startswith('Brush /'):return rigid(10)
 if part.startswith('Tail /'):
  a=smoothstep(.02,.27,x)*smoothstep(.35,.48,y);return [1,10,0,0],[1.-a,a,0.,0.]
 if part.startswith('Arm /'):
  side=part.split()[-3];owner=7 if side=='-1' else 9
  a=smoothstep(.16,.235,-x) if side=='-1' else smoothstep(.14,.29,x)
  return [owner,owner+1,0,0],[1.-a,a,0.,0.]
 if part.startswith(('Hand /','Finger /')):
  side=part.split()[-1] if part.startswith('Hand /') else part.split()[-2]
  return rigid(8 if side=='-1' else 10)
 if part.startswith(('Head /','Eye /','Muzzle /','Nostril /','Mouth /','Tongue /')):return rigid(3)
 a=smoothstep(.60,.66,y);b=smoothstep(.66,.715,y)
 return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Smeargle clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength;c=a*(.20 if clip=='idle' else 1.);d=b*(.20 if clip=='idle' else 1.)
 return [(0.,0.,0.),(0.,.014*a,0.),(0.,.014*a,0.),(.008*a,.020*a,0.),
  (.018*b,0.,-.025*b),(.018*b,0.,.025*b),(.012*b,.012*a,.015*b),
  (.015*c,.025*d,-.020*c),(.025*c,.035*d,.025*c),
  (0.,-.020*c,.010*d),(.060*c,-.035*d,0.),
  (0.,.015*a,0.),(0.,-.015*a,0.)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])
def pack_parts(model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name=source['part']
  if not name.startswith('Arm /'):
   parts.append((source,{'source_order':order}));continue
  side=name.split()[-3];color=tuple(source['base_color']);key=(side,color)
  if key not in groups:
   label='cream' if color==tuple(model['primitives'][0]['base_color']) else 'brown'
   joined={'part':'Arm / cream forelimb and brown wrist bands '+side+' group '+label,'positions':[],'normals':[],'indices':[],'base_color':source['base_color']}
   metadata={'source_parts':[]};groups[key]=(joined,metadata);parts.append((joined,metadata))
  joined,metadata=groups[key];start=len(joined['positions'])//3;first=len(joined['indices'])
  metadata['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])})
  joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGEST='e87a6aa122e9560a34741eb17c164e9ae34f2da00990129878ef18676460b595'

def export_smeargle(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='smeargle' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Smeargle anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Smeargle coherent held brush-tail and planted paws skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'smeargle/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'smeargle','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'smeargle','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'smeargle','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 packed_parts=pack_parts(model)
 if geometry_digest({'primitives':[p for p,metadata in packed_parts]})!=PACKED_REST_DIGEST:raise ValueError('unreviewed packed Smeargle geometry')
 for part,metadata in packed_parts:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'smeargle/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'smeargle','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'smeargle.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted paws, grooves and ankle markings; coherent canine face and beret; shared arm-band rings; held tail and brush stroke; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'smeargle','source_sha256':REST_DIGEST,
    'source_part_ranges':[metadata for part,metadata in packed_parts],'packed_sha256':PACKED_REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/smeargle_sculpt.py','geometry_preservation':'39 non-arm parts retain exact original f32 geometry; 26 arm bands use current shared continuous two-span frames and normals',
    'animation_contract':'identity floor root; fixed paws, grooves and ankle cuffs; spatial neck field and coherent face; articulated ears and beret; shared banded-arm fields; held hand, distal tail and both scalloped brush layers share the painting owner; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Smeargle exceeds existing accessor limit')
 return g.bytes()


def decode_smeargle(blob):
 if len(blob)>MAX_BYTES or blob!=export_smeargle():raise ValueError('noncanonical Smeargle GLB')
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
 return {'name':'smeargle','version':1,'coordinate_system':COORDINATES,'primitives':[p for order,p in sorted(parts,key=lambda pair:pair[0])]}


def read_smeargle(path):
 with Path(path).open('rb') as f:return decode_smeargle(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_smeargle();(a.out/'smeargle.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
