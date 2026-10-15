#!/usr/bin/env python3
"""Canonical Quagsire with grounded pads and a coherent body-to-tail ridge."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from axolotl_sculpt import sculpture
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('quagsire',)
HEIGHT=1.13
REST_DIGESTS={'quagsire':'d97c02d58f80145da2df67f1902083c13ccc862253c75e9332e5d8b04e5e36ba'}
PARTS={n:tuple(p['part'] for p in sculpture(n)['primitives']) for n in SPECIES}
MAX_BYTES=512000
RAW=sculpture('quagsire',normalize=False)
LOW=min(v for p in RAW['primitives'] for v in p['positions'][1::3])
SCALE=HEIGHT/(max(v for p in RAW['primitives'] for v in p['positions'][1::3])-LOW)
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
JOINTS={'quagsire':tuple((name,parent,pivot(p) if parent is not None else p) for name,parent,p in (
 ('root',None,(0.,0.,0.)),('pelvis',0,(0.,.28,-.01)),('torso',1,(0.,.54,-.01)),('head',2,(0.,.78,.028)),
 ('arm_left',2,(-.27,.70,.01)),('arm_right',2,(.27,.70,.01)),('hand_left',4,(-.320,.46,.089)),('hand_right',5,(.320,.46,.089)),
 ('leg_left',1,(-.192,.155,.035)),('leg_right',1,(.192,.155,.035)),
 ('tail_base',1,(0.,.26,-.18)),('tail_middle',10,(0.,.35,-.48)),('tail_tip',11,(0.,.47,-.72))))}
CLIPS=(('idle',3.6,73),('attack',1.,65),('hit',1.,65))


def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()


def body_field(y):
 active=smoothstep(.075,.22,y);torso=smoothstep(.36,.57,y);head=smoothstep(.61,.75,y)
 return {0:1.-active,1:active*(1.-torso),2:active*torso*(1.-head),3:active*torso*head}


def tail_field(z):
 base=smoothstep(.15,.34,-z);middle=smoothstep(.37,.61,-z);tip=smoothstep(.67,.86,-z)
 return {1:1.-base,10:base*(1.-middle),11:base*middle*(1.-tip),12:base*middle*tip}


def axial_field(position):
 # Body, tail and their dorsal ridge use one spatial field across the join.
 # The transition has at most four active owners: head/root are absent here,
 # and the later tail joints begin after the body-to-tail handoff.
 x,y,z=position;back=smoothstep(.23,.34,-z)
 field={}
 for owner,weight in body_field(y).items():field[owner]=field.get(owner,0.)+weight*(1.-back)
 for owner,weight in tail_field(z).items():field[owner]=field.get(owner,0.)+weight*back
 return field


def skin_weights(species,part,position):
 x,y,z=position;raw=(x/SCALE,(y/SCALE)+LOW,z/SCALE)
 if part.startswith(('Foot /','Toe crease /')):field={0:1.}
 elif part.startswith('Leg /'):
  free=smoothstep(.07,.19,raw[1]);field={0:1.-free,8 if x<0 else 9:free}
 elif part.startswith(('Eye /','Mouth /','Nostril /')):field={3:1.}
 elif part.startswith(('Arm /','Finger crease /')):
  arm,hand=(4,6) if x<0 else (5,7);root=smoothstep(.60,.74,raw[1]);wrist=1.-smoothstep(.43,.55,raw[1])
  field={2:root,arm:(1.-root)*(1.-wrist),hand:(1.-root)*wrist}
 else:field=axial_field(raw)
 used=[(j,w) for j,w in sorted(field.items()) if w>1e-10]
 if len(used)>4:raise ValueError('Quagsire spatial field exceeds four skin owners')
 ids=[j for j,w in used];weights=[w for j,w in used]
 return ids+[0]*(4-len(ids)),weights+[0.]*(4-len(weights))


def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Quagsire clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*13
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
 def lag(phase):return math.sin(math.tau*t-phase)+math.sin(phase)
 if clip=='idle':
  return [(0.,0.,0.),(.009*breath,.008*wave,0.),(.010*breath,.008*wave,0.),(-.011*breath,.021*wave,.007*wave),
          (.016*lag(.3),.006*wave,-.009*breath),(.016*lag(.5),-.006*wave,.009*breath),
          (.018*lag(.6),0.,-.008*breath),(.018*lag(.8),0.,.008*breath),
          (.007*breath,0.,0.),(.007*breath,0.,0.),(.012*wave,.016*wave,0.),(.018*lag(.5),.024*lag(.5),0.),(.023*lag(.8),.032*lag(.8),0.)]
 brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
 trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
 recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
 sign=1. if clip=='attack' else -1.;main=brace if sign>0 else recoil
 return [(0.,0.,0.),(sign*.019*main,0.,0.),(sign*.028*main,0.,0.),(sign*.045*main,.008*main,0.),
         (sign*.038*main,0.,-.020*main),(sign*.038*main,0.,.020*main),
         (sign*.030*trail,0.,-.012*trail),(sign*.030*trail,0.,.012*trail),
         (-sign*.012*main,0.,0.),(-sign*.012*main,0.,0.),
         (-sign*.032*main,.012*main,0.),(-sign*.050*trail,.025*trail,0.),(-sign*.068*trail,.035*trail,0.)]


def export_quagsire(model=None):
 species='quagsire'
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed amphibian neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct amphibian anatomy and standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'early head and arm brace; lagged palms and paddle chain; coherent dorsal ridge; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/axolotl_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; fixed pads and toe folds; fitted rigid face; articulated arms palms and paddle; coherent body-to-tail ridge; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('amphibian exceeds existing accessor limit')
 return g.bytes()


def decode_quagsire(blob):
 species='quagsire'
 if len(blob)>MAX_BYTES or blob!=export_quagsire():raise ValueError('noncanonical amphibian GLB')
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


def read_quagsire(path):
 with Path(path).open('rb') as f:return decode_quagsire(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--only',choices=SPECIES);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 for n in ([a.only] if a.only else SPECIES):
  blob=export_quagsire();(a.out/(n+'.glb')).write_bytes(blob);print(n,len(blob),'bytes',len(PARTS[n]),'parts',len(JOINTS[n]),'joints',REST_DIGESTS[n])
if __name__=='__main__':main()
