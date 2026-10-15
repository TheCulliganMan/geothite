#!/usr/bin/env python3
"""Canonical Spinarak and Ariados skins with six articulated limbs."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from spider_sculpt import sculpture,HEIGHT
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('spinarak','ariados')
REST_DIGESTS={'spinarak': 'b88a15aff01d0ca5d5362a8bb3bcfc8d86c389a1bc7c5d15c6d722bb194ee1d6', 'ariados': 'b680cd4729af2101b86594f951817cb56f56a2c4e03b80209c2863345caa57f4'}
PARTS={n:tuple(p['part'] for p in sculpture(n)['primitives']) for n in SPECIES}
MAX_BYTES=768*1024


def normalization(species):
 m=sculpture(species,normalize=False)
 lo=min(v for p in m['primitives'] for v in p['positions'][1::3]);hi=max(v for p in m['primitives'] for v in p['positions'][1::3])
 return lo,HEIGHT/(hi-lo)


def pivot(species,p):
 lo,scale=NORMALIZATION[species];return tuple((v-(lo if i==1 else 0.))*scale for i,v in enumerate(p))


def limb_path(species,side,j,raised=False):
 if raised:return ((side*.17,.47,-.18),(side*.29,.58,-.27),(side*.35,.80,-.31),(side*.39,.98,-.34))
 big=species=='ariados';z=.16-j*(.36 if big else .20);spread=.60 if big else .48
 return ((side*.15,.29,z),(side*spread*.62,.36,z+(.07 if j==0 else -.03)),(side*spread,.032,z+(.23 if j==0 else -.16)))


def joints_for(species):
 js=[('root',None,(0.,0.,0.)),('abdomen',0,pivot(species,(0.,.32,-.19))),('head',1,pivot(species,(0.,.30,.21))),
     ('horn',2,pivot(species,(0.,.426,.29) if species=='ariados' else (0.,.442,.205))),
     ('mandible_left',2,pivot(species,(-.074,.215,.34))),('mandible_right',2,pivot(species,(.074,.215,.34)))]
 for side in (-1,1):
  for j in range(2 if species=='ariados' else 3):
   path=limb_path(species,side,j);label=('left' if side<0 else 'right')+'_'+str(j)
   parent=len(js);js.extend([('leg_'+label+'_upper',1,pivot(species,path[0])),('leg_'+label+'_lower',parent,pivot(species,path[1]))])
 if species=='ariados':
  for side in (-1,1):
   path=limb_path(species,side,0,True);label='left' if side<0 else 'right';parent=len(js)
   js.extend([('raised_'+label+'_base',1,pivot(species,path[0])),('raised_'+label+'_tip',parent,pivot(species,path[1]))])
 return tuple(js)
NORMALIZATION={n:normalization(n) for n in SPECIES}
JOINTS={n:joints_for(n) for n in SPECIES}
CLIPS=(('idle',3.6,73),('attack',1.,65),('hit',1.,65))


def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()


def path_progress(species,position,path):
 # Every vertex at a material seam uses the same spatial field. Band names
 # choose the limb only; they never decide its influence or planted support.
 path=[pivot(species,p) for p in path];lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths)
 candidates=[];distance=0.
 for a,b,length in zip(path,path[1:],lengths):
  d=[v-u for u,v in zip(a,b)];u=max(0.,min(1.,sum((v-w)*q for v,w,q in zip(position,a,d))/(length*length)))
  point=[v+u*q for v,q in zip(a,d)];candidates.append((math.dist(position,point),(distance+u*length)/total));distance+=length
 return min(candidates)[1]


def skin_weights(species,part,position):
 if part.startswith('Walking leg /'):
  side,j=map(int,part.split('/ ')[1].split(' band ')[0].split());count=2 if species=='ariados' else 3
  upper=6+(0 if side<0 else count*2)+j*2;lower=upper+1
  t=path_progress(species,position,limb_path(species,side,j));knee=smoothstep(.22,.56,t);free=1.-smoothstep(.72,.95,t)
  return [0,upper,lower,0],[1.-free,free*(1.-knee),free*knee,0.]
 if part.startswith('Raised leg /'):
  side=int(part.split('/ ')[1].split(' band ')[0]);upper=14 if side<0 else 16;lower=upper+1
  t=path_progress(species,position,limb_path(species,side,0,True));tip=smoothstep(.27,.73,t)
  return [upper,lower,0,0],[1.-tip,tip,0.,0.]
 if part.startswith(('Abdomen /','Abdomen stripe /','False eye /','False mouth /')):owner=1
 elif part.startswith('Horn /'):owner=3
 elif part.startswith('Mandible /'):owner=4 if position[0]<0 else 5
 else:owner=2
 return [owner,0,0,0],[1.,0.,0.,0.]


def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid spider clip/time')
 pose=[(0.,0.,0.)]*len(JOINTS[species])
 if t in (0.,1.):return pose
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
 brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
 lag=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
 recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
 if clip=='idle':
  pose[1]=(.010*breath,.008*wave,0.);pose[2]=(.013*breath,.014*wave,0.);pose[3]=(.012*wave,0.,0.)
  for j in (4,5):pose[j]=(.010*breath,0.,(-1 if j==4 else 1)*.024*breath)
 else:
  sign=1 if clip=='attack' else -1;main=brace if sign>0 else recoil
  pose[1]=(sign*.025*main,0.,0.);pose[2]=(sign*.040*main,.006*main,0.);pose[3]=(sign*.023*lag,0.,0.)
  for j in (4,5):pose[j]=(sign*.028*main,0.,(-1 if j==4 else 1)*.060*main)
 for j,(name,parent,p) in enumerate(JOINTS[species][6:],6):
  side=-1 if 'left' in name else 1
  if name.startswith('raised_'):
   signal=wave if clip=='idle' else lag;amount=.020 if clip=='idle' else .050
   pose[j]=(amount*signal,side*amount*.6*signal,side*amount*.5*signal)
  else:
   signal=breath if clip=='idle' else (brace if clip=='attack' else -recoil);amount=.008 if clip=='idle' else .023
   pose[j]=(amount*signal,side*amount*.4*signal,side*amount*.6*signal)
 return pose


def export_spider(species,model=None):
 if species not in SPECIES:raise ValueError('unknown spider species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed spider neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct spider anatomy and standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'early head and mandible brace; fixed walking tips; coherent striped legs; lagged Ariados dorsal feelers; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/spider_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; fixed walking tips; fitted head and dorsal markings; articulated horn mandibles and six two-joint limbs; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('spider exceeds existing accessor limit')
 return g.bytes()


def decode_spider(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_spider(species):raise ValueError('noncanonical spider GLB')
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


def read_spinarak(path):
 with Path(path).open('rb') as f:return decode_spider(f.read(MAX_BYTES+1),'spinarak')
def read_ariados(path):
 with Path(path).open('rb') as f:return decode_spider(f.read(MAX_BYTES+1),'ariados')


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--only',choices=SPECIES);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 for n in ([a.only] if a.only else SPECIES):
  blob=export_spider(n);(a.out/(n+'.glb')).write_bytes(blob);print(n,len(blob),'bytes',len(PARTS[n]),'parts',len(JOINTS[n]),'joints',REST_DIGESTS[n])
if __name__=='__main__':main()
