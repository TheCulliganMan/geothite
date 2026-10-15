#!/usr/bin/env python3
"""Unown A..Z standard skins with coherent eyes and delayed glyph rotation."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from unown_sculpt import sculpture as sculpt,HEIGHT,CENTER,STROKES
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

SPECIES=tuple('unown_'+letter for letter in STROKES)
REST_DIGESTS={'unown_a': 'c73dbe28b59ab0a8a8ea462cc763acf05e7ad57d5df10794457c0b26429d08d0', 'unown_b': '709f94e4172e8ecf2c3d05778bc8cf7c69e59cf652cc7906c2e003711f303a3b', 'unown_c': 'ea246f2878eb100b62e1f4aea35ec73dad44f9ad3ec9d1f4ac2ceb294e93ea3d', 'unown_d': 'f8bdf4987fd951ce491301ff87cc18e87029ca0be5f50b55eadaf27367ce1844', 'unown_e': 'b3791a0180b5c07e5386cfd76d68620af062be2dc3173dacd31a2014af0ac6cc', 'unown_f': 'e2796140938ec39927d1b7e28ce5795a960f3fbfcfd3d260362a25e0954fa700', 'unown_g': '1d1c4214f8d3f0863b87939f23bb02c43d0b26b5a159f53fc90663773b3c9244', 'unown_h': 'ac58fb0cc0dd034c8d1f2e620ea7fe6d9a7f09bce702170c0a66605d62a1a0f6', 'unown_i': 'b7de7d05fdb79cc267821edc5a336048341cf8955b9b063172361b9e7d16516f', 'unown_j': 'd2f4bc9389a525876905ab26cec40f557679e1243cd9a7ce2bcbe94ddd27441a', 'unown_k': 'f6864362324b442bf98325cf5687120a4b7a16d9635a01bafe09f15b3265d8fe', 'unown_l': '6fc0930cd4fd5a14c4361c50cafce9bdeb8ae85ba29dae7be405a2cec9b39e31', 'unown_m': 'a0ab086eee8ac95a18c8986b74c1be5482c88a53c21ac6240b1e145b349fc9fa', 'unown_n': '102d136ec2c4047e2cacf80937b16c340229705f0a8231ac1b212bc023fcdff9', 'unown_o': 'afee279d3d7df6bedf7a697ab6e1a7cd79231929c7a92ff900f5f9da423e95aa', 'unown_p': '7a7f956505eae3dafd3b99338c2d7fd4f904f6d6b9bcdb5684a18e02a58f5ec0', 'unown_q': '45af59081b1f4de2664279deb2854cc0d73912cd0fd66456c8684b090f3c9195', 'unown_r': 'b4d44af912dd13e8380f2535a46af8d9e813f79342725b4fc56719d5119a6d77', 'unown_s': 'de9ea07911edecebd23e37e216f2032a05ff56cc341422f6ee5393662604ad7c', 'unown_t': '24f974f50e454a54323368b22fe264a0058a75d4759ecde49cd6a0d01d8617a6', 'unown_u': '6708baf987fcc42d083657b19311a6f755c77b500bb03fa3b3930ac34cdb5b62', 'unown_v': '1026bb702cf9c9f5fa8d1b3f006605390b26dafc4a16943b3031489a529274bb', 'unown_w': 'b62999e5454a604d8a1f68300934f850fff420ecb1ccd906f748a88d1196352c', 'unown_x': '22acc7dc272833b0f1e80ae84c76b538633fa6884b0d42ca8b262d1f77519a02', 'unown_y': '1074672b12d5ef711208bbab10713eb1d6f232092b254c8dafcb0538bfe7d4e7', 'unown_z': 'c47914a33662a3304752409e05e0227748bbc7abf7ba333a1c997aa467174323'}
PARTS={species:tuple(p['part'] for p in sculpt(species[-1])['primitives']) for species in SPECIES};MAX_BYTES=1024*1024

def sculpture(species):
 if species not in SPECIES:raise ValueError('Unown requires an A..Z form')
 return sculpt(species[-1])
@lru_cache(maxsize=26)
def authoring_transform(species):
 raw=sculpt(species[-1],normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHT/span for i,v in enumerate(point))
JOINTS={species:(('root',None,(0.,0.,0.)),('eye',0,normalized(species,CENTER)),('glyph',1,normalized(species,CENTER))) for species in SPECIES}

CLIPS=(('idle',3.6,85),('attack',1.,65),('hit',1.,65))


def skin_weights(species,part,position):
 if not part.startswith('Glyph /'):return [1,0,0,0],[1.,0.,0.,0.]
 low,span=authoring_transform(species)
 x=position[0]*span/HEIGHT;y=position[1]*span/HEIGHT+low-CENTER[1]
 flex=smoothstep(.21,.37,math.hypot(x,y));return [1,2,0,0],[1.-flex,flex,0.,0.]

def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Unown clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*3
 wave=math.sin(math.tau*t);lag=math.sin(math.tau*t-.4)+math.sin(.4)
 if clip=='idle':eye=.055*wave;glyph=.080*lag
 else:
  sign=1. if clip=='attack' else -.65
  eye=sign*.12*envelope(t,((0.,0.),(.18,1.),(.40,.68),(.67,.1),(1.,0.)))
  glyph=sign*.10*envelope(t,((0.,0.),(.28,1.),(.53,.60),(.79,-.08),(1.,0.)))
 return [(0.,0.,0.),(0.,eye,0.),(0.,glyph,0.)]


def export_glyph(species,model=None):
 if species not in SPECIES:raise ValueError('unknown Unown species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed Unown neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct Unown anatomy and standard skin v1'
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
  key=(tuple(values),kind,component,target,normalized)
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
    'NORMAL':g.accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':species,'primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(species,clip,i/(count-1))[joint])]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':species+'.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'coherent eye turn and delayed glyph flex; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/unown_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; coherent eye; 26 authored glyph silhouettes; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Unown exceeds existing accessor limit')
 return g.bytes()


def decode_glyph(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_glyph(species):raise ValueError('noncanonical Unown GLB')
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


def read_glyph(path,species):
 with Path(path).open('rb') as f:return decode_glyph(f.read(MAX_BYTES+1),species)
def export_unown(letter='a',model=None):return export_glyph('unown_'+letter.lower(),model)
def read_unown(path):return read_glyph(path,path.stem.lower())

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--form',choices=tuple(STROKES)+('all',),default='all');p.add_argument('--output',type=Path);p.add_argument('--output-dir',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));args=p.parse_args()
 forms=tuple(STROKES) if args.form=='all' else (args.form,)
 if args.output and len(forms)!=1:p.error('--output requires one form')
 for letter in forms:
  path=args.output or args.output_dir/('unown_'+letter+'.glb');path.parent.mkdir(parents=True,exist_ok=True);blob=export_unown(letter);path.write_bytes(blob);print(path,len(blob),REST_DIGESTS['unown_'+letter])
if __name__=='__main__':main()
