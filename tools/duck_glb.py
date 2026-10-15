#!/usr/bin/env python3
"""Canonical Psyduck and Golduck standard skins over their original duck recipes."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from duck_sculpt import sculpture,HEIGHT
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('psyduck','golduck')
REST_DIGESTS={'psyduck':'4d28e9c51e3d5ed94fcccfe356d5cc60dc9ee2d7645332334c7bb7d6ef5b9b4c',
              'golduck':'e7ab59e4de9d3d78f761557cc5b62d6e41442f54c3e25b7fffbecba18892178d'}
PARTS={n:tuple(p['part'] for p in sculpture(n)['primitives']) for n in SPECIES}
MAX_BYTES=1024*1024
JOINTS={
 'psyduck':(('root',None,(0.,0.,0.)),('torso',0,(0.,.40,.02)),('head',1,(0.,.70,.03)),
            ('arm_left',2,(-.18,.52,.02)),('hand_left',3,(-.252,.778,.11)),
            ('arm_right',2,(.18,.52,.02)),('hand_right',5,(.252,.778,.11)),
            ('tail',1,(0.,.22,-.19)),('hair',2,(0.,.916,-.018)),('lower_bill',2,(0.,.63,.27))),
 'golduck':(('root',None,(0.,0.,0.)),('torso',0,(0.,.45,.005)),('head',1,(0.,.765,.01)),
            ('arm_left',1,(-.14,.60,.01)),('hand_left',3,(-.32,.30,.105)),
            ('arm_right',1,(.14,.60,.01)),('hand_right',5,(.32,.30,.105)),
            ('tail',1,(0.,.18,-.10)),('crown',2,(0.,.846,-.01)),('lower_bill',2,(0.,.74,.15)),
            ('tail_tip',7,(0.,.175,-.345))),
}
CLIPS=(('idle',3.6,73),('attack',1.,65),('hit',1.,65))


def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()


def skin_weights(species,part,position):
 x,y,z=position;left=x<0;arm,hand=(3,4) if left else (5,6)
 if part.startswith(('Leg /','Foot /','Foot membrane /','Foot claw /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Body /'):
  lo,hi=(.48,.58) if species=='psyduck' else (.56,.68)
  head=smoothstep(lo,hi,y);return [1,2,0,0],[1.-head,head,0.,0.]
 if part.startswith('Arm /'):
  wrist=smoothstep(.63,.76,y) if species=='psyduck' else 1.-smoothstep(.31,.49,y)
  return [arm,hand,0,0],[1.-wrist,wrist,0.,0.]
 if part.startswith(('Hand /','Hand membrane /','Hand claw /','Finger /')):return [hand,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Hair /','Crown /')):return [8,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Bill / fitted lower jaw','Bill / subtle curved seam')):return [9,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail /'):
  if species=='psyduck':return [7,0,0,0],[1.,0.,0.,0.]
  rear=-z;base=smoothstep(.09,.21,rear);tip=smoothstep(.31,.52,rear)
  return [1,7,10,0],[1.-base,base*(1.-tip),base*tip,0.]
 return [2,0,0,0],[1.,0.,0.,0.]


def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid duck clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
 def lag(p):return math.sin(math.tau*t-p)+math.sin(p)
 if clip=='idle':
  pose=[(0.,0.,0.),(.009*breath,.008*wave,0.),(.009*breath,.010*wave,0.),
        (.013*lag(.2),.006*wave,-.010*breath),(.008*lag(.3),0.,-.006*breath),
        (.012*lag(.5),-.006*wave,.010*breath),(.007*lag(.6),0.,.006*breath),
        (.009*wave,.016*wave,0.),(.015*lag(.7),.011*wave,0.),(.005*breath,0.,0.)]
 else:
  brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
  trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
  if clip=='attack':pose=[(0.,0.,0.),(.028*brace,0.,0.),(.018*brace,0.,0.),
        (.035*brace,.010*brace,-.020*brace),(.019*trail,0.,-.010*trail),
        (.032*brace,-.010*brace,.020*brace),(.017*trail,0.,.010*trail),
        (.024*brace,.034*brace,0.),(.030*trail,.012*trail,0.),(.019*brace,0.,0.)]
  else:
   recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
   pose=[(0.,0.,0.),(-.021*recoil,.006*recoil,0.),(-.014*recoil,0.,0.),
         (-.025*recoil,0.,.015*recoil),(-.014*trail,0.,.008*trail),
         (-.023*recoil,0.,-.015*recoil),(-.012*trail,0.,-.008*trail),
         (-.018*recoil,-.028*recoil,0.),(-.023*trail,0.,0.),(-.012*recoil,0.,0.)]
 if species=='golduck':
  if clip=='idle':pose.append((.014*lag(.7),.029*lag(.7),0.))
  else:pose.append((.035*trail if clip=='attack' else -.028*trail,.056*trail if clip=='attack' else -.043*trail,0.))
 return pose


def export_duck(species,model=None):
 if species not in SPECIES:raise ValueError('unknown duck species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed duck neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct duck anatomy and standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'early duck head/body brace; coherent webbed hands; lagged hair/crown and tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/duck_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; planted webbed feet; coherent fitted head; articulated bill arms hands crest and tail; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('duck exceeds existing accessor limit')
 return g.bytes()


def decode_duck(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_duck(species):raise ValueError('noncanonical duck GLB')
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


def read_psyduck(path):
 with Path(path).open('rb') as f:return decode_duck(f.read(MAX_BYTES+1),'psyduck')
def read_golduck(path):
 with Path(path).open('rb') as f:return decode_duck(f.read(MAX_BYTES+1),'golduck')


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--only',choices=SPECIES);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 for n in ([a.only] if a.only else SPECIES):
  blob=export_duck(n);(a.out/(n+'.glb')).write_bytes(blob);print(n,len(blob),'bytes',len(PARTS[n]),'parts',len(JOINTS[n]),'joints',REST_DIGESTS[n])
if __name__=='__main__':main()
