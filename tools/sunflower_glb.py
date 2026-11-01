#!/usr/bin/env python3
"""Canonical seedling and sunflower standard skins with coherent leaf veins."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from sunflower_sculpt import sculpture,SPECIES,HEIGHT
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
MAX_BYTES=1024*1024
SOURCE={s:sculpture(s) for s in SPECIES}
RAW={s:sculpture(s,normalize=False) for s in SPECIES}
LOW={s:min(v for p in RAW[s]['primitives'] for v in p['positions'][1::3]) for s in SPECIES}
SCALE={s:HEIGHT/(max(v for p in RAW[s]['primitives'] for v in p['positions'][1::3])-LOW[s]) for s in SPECIES}
PARTS={s:tuple(p['part'] for p in SOURCE[s]['primitives']) for s in SPECIES}
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
REST_DIGESTS={'sunkern': '2c11fc4e36f23be374beb989f959372f218b2cb292dcc91286cb8804df72ea8b', 'sunflora': '3342555c766e836e7ea17ed6fc8a931fb727e29fbb06f97eee739104ea670f70'}
def pivot(species,p):return tuple((v-(LOW[species] if i==1 else 0.))*SCALE[species] for i,v in enumerate(p))
SEED_JOINTS=(('root',None,(0.,0.,0.)),('seed_body',0,(0.,.24,0.)),('sprout',1,(0.,.548,0.)),
 ('leaf_left',2,(0.,.704,-.008)),('leaf_tip_left',3,(-.16,.77,-.008)),('leaf_right',2,(0.,.704,-.008)),('leaf_tip_right',5,(.16,.77,-.008)))
FLOWER_JOINTS=(('root',None,(0.,0.,0.)),('bulb',0,(0.,.28,0.)),('stem_base',1,(0.,.48,0.)),('stem_tip',2,(0.,.66,0.)),('flower_disk',3,(0.,.73,.025)),
 ('leaf_left',1,(-.07,.43,.012)),('leaf_tip_left',5,(-.26,.397,.012)),('leaf_right',1,(.075,.45,.012)),('leaf_tip_right',7,(.36,.441,.012)))
RAYS=tuple(('petal_'+str(i),4,(-.14*math.sin(a),.885+.14*math.cos(a),-.026)) for i,a in enumerate(math.tau*i/8 for i in range(8)))
BRACTS=tuple(('bract_'+str(i),4,(-.12*math.sin(a),.885+.12*math.cos(a),-.041)) for i,a in enumerate(math.tau*i/8+math.pi/8 for i in range(8)))
JOINTS={s:tuple((n,parent,pivot(s,p) if parent is not None else p) for n,parent,p in (SEED_JOINTS if s=='sunkern' else FLOWER_JOINTS+RAYS+BRACTS)) for s in SPECIES}
CLIPS=(('idle',3.6,91),('attack',1.,65),('hit',1.,65))

def skin_weights(species,part,position):
 x,y,z=position;raw_y=y/SCALE[species]+LOW[species];raw_x=abs(x/SCALE[species])
 if species=='sunkern':
  if part.startswith(('Body /','Coat /')):
   upper=smoothstep(.14,.26,raw_y);return [0,1,0,0],[1.-upper,upper,0.,0.]
  if part.startswith('Sprout /'):
   upper=smoothstep(.548,.66,raw_y);return [1,2,0,0],[1.-upper,upper,0.,0.]
  if part.startswith(('Leaf /','Leaf vein /')):
   a,b=(3,4) if x<0 else (5,6);attach=smoothstep(0.,.08,raw_x);tip=smoothstep(.12,.28,raw_x)
   return [2,a,b,0],[1.-attach,attach*(1.-tip),attach*tip,0.]
  return [1,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Foot /'):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Body /','Leg /')):
  if raw_y<.48:
   f=smoothstep(.15,.28,raw_y);return [0,1,0,0],[1.-f,f,0.,0.]
  if raw_y<.62:
   f=smoothstep(.48,.60,raw_y);return [1,2,0,0],[1.-f,f,0.,0.]
  f=smoothstep(.62,.73,raw_y);return [2,3,0,0],[1.-f,f,0.,0.]
 if part.startswith(('Leaf arm /','Leaf vein /')):
  a,b=(5,6) if x<0 else (7,8);attach=smoothstep(.07,.14,raw_x);tip=smoothstep(.24,.43,raw_x)
  return [1,a,b,0],[1.-attach,attach*(1.-tip),attach*tip,0.]
 if part.startswith('Petal /'):return [9+int(part.rsplit(' ',1)[1]),0,0,0],[1.,0.,0.,0.]
 if part.startswith('Bract /'):return [17+int(part.rsplit(' ',1)[1]),0,0,0],[1.,0.,0.,0.]
 return [4,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid sunflower clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
 def lag(phase):return math.sin(math.tau*t-phase)+math.sin(phase)
 if clip=='idle':
  body=.009*breath;stem=.015*wave;head=.016*lag(.55);leaf=.020*wave;tip=.027*lag(.4);petal=.010*breath
 else:
  early=envelope(t,((0.,0.),(.18,1.),(.42,.65),(.73,.08),(1.,0.)))
  late=envelope(t,((0.,0.),(.31,1.),(.56,.55),(.81,-.08),(1.,0.)))
  sign=1. if clip=='attack' else -.8
  body=sign*.020*early;stem=sign*.030*early;head=sign*.035*late;leaf=sign*.045*early;tip=sign*.055*late;petal=sign*.028*late
 if species=='sunkern':return [(0.,0.,0.),(body,.005*wave,0.),(stem,.008*wave,0.),(0.,0.,-leaf),(tip,0.,-.012*breath),(0.,0.,leaf),(tip,0.,.012*breath)]
 result=[(0.,0.,0.),(body,.005*wave,0.),(stem,0.,.005*wave),(head,.007*wave,0.),(head*.6,0.,.006*lag(.5)),
         (.007*wave,0.,-leaf),(tip,0.,-.009*breath),(.007*wave,0.,leaf),(tip,0.,.009*breath)]
 for i in range(16):
  a=math.tau*(i%8)/8+(math.pi/8 if i>=8 else 0.)
  f=petal*(.65 if i>=8 else 1.)+(.004*lag(.2+i*.11) if clip=='idle' else 0.)
  result.append((f*math.cos(a),f*math.sin(a),0.))
 return result

def export_sunflower(species,model=None):
 if model is None:model=SOURCE[species]
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed sunflower family neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct sunflower family anatomy and standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'grounded seed or bulb; coherent leaves; delayed stem disk and rays; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/sunflower_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; grounded base or feet; fitted face; coherent leaves and veins; articulated sprout or stem disk petals and bracts; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('sunflower family exceeds existing accessor limit')
 return g.bytes()


def decode_sunflower(species,blob):
 if len(blob)>MAX_BYTES or blob!=export_sunflower(species):raise ValueError('noncanonical sunflower family GLB')
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


def read_sunflower(species,path):
 with Path(path).open('rb') as f:return decode_sunflower(species,f.read(MAX_BYTES+1))


def read_sunkern(path):return read_sunflower('sunkern',path)
def read_sunflora(path):return read_sunflower('sunflora',path)

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--only',choices=SPECIES);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 for s in ([a.only] if a.only else SPECIES):
  b=export_sunflower(s);(a.out/(s+'.glb')).write_bytes(b);print(s,len(b),'bytes',len(PARTS[s]),'parts',len(JOINTS[s]),'joints',REST_DIGESTS[s])
if __name__=='__main__':main()
