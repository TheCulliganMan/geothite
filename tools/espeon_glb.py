#!/usr/bin/env python3
"""Canonical slender sun cat with planted paws, broad ears, ruby gem and independent forked tail tips."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from espeon_sculpt import sculpture,HEIGHT,TAIL_POINTS
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
REST_DIGEST='051135bb83bc3ca1a337c80500fa056725f8b3fa59436c1b5267d195d4d5eb4e'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
PATH=TAIL_POINTS
BASE=[('root',None,(0.,0.,0.)),('body',0,(0.,.40,-.07)),
 ('neck',1,(0.,.51,.15)),('head',2,(0.,.67,.21)),
 ('ear_left',3,(-.106,.738,.174)),('ear_right',3,(.106,.738,.174)),
 ('tail_base',1,PATH[0]),('tail_middle',6,PATH[2]),('tail_hub',7,PATH[-1]),
 ('fork_left',8,PATH[-1]),('fork_right',8,PATH[-1])]
for side in (-1,1):
 for row in (0,1):BASE.append(('leg_'+str(side)+'_'+str(row),0,(side*(.080 if row==0 else .115),.220,.137 if row==0 else -.220)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def path_fraction(point):
 lengths=[math.dist(a,b) for a,b in zip(PATH,PATH[1:])];total=sum(lengths);offset=0.;best=(math.inf,0.)
 for a,b,length in zip(PATH,PATH[1:],lengths):
  delta=[b[k]-a[k] for k in range(3)];t=max(0.,min(1.,sum((point[k]-a[k])*delta[k] for k in range(3))/(length*length)))
  best=min(best,(math.dist(point,[a[k]+t*delta[k] for k in range(3)]),(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(part,point):
 x,y,z=[v/SCALE+(LOW if i==1 else 0.) for i,v in enumerate(point)]
 if part.startswith(('Paw /','Toe crease /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Body /','Shoulder /','Haunch /')):return [1,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Ear /','Ear inset /')):return [4 if part.endswith('-1') else 5,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail hub /'):return [8,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail fork /'):
  owner=9 if part.endswith('-1') else 10;a=smoothstep(.70,.79,y)
  return [8,owner,0,0],[1.-a,a,0.,0.]
 if part.startswith('Neck /'):
  a=smoothstep(.42,.50,y);b=smoothstep(.52,.59,y);return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith('Leg /'):
  side,row=map(int,part.split()[-2:]);owner=11+(0 if side<0 else 2)+row
  a=smoothstep(.13,.32,y);g=.70*smoothstep(.054,.130,y)*(1.-smoothstep(.280,.380,y))
  return [0,1,owner,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith('Tail /'):
  t=path_fraction((x,y,z));a=smoothstep(0.,.20,t);b=smoothstep(.15,.50,t);c=smoothstep(.45,.82,t)
  return [1,6,7,8],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
 return [3,0,0,0],[1.,0.,0.,0.]

def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Espeon clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);body=.002*a;neck=.005*b;head=.010*b;turn=.014*a;fan=.022*b;tail=.035*a;leg=.006*a;flap=.025*b
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));gain=1. if clip=='attack' else -.65
  body=.006*a*gain;neck=.015*a*gain;head=.035*b*gain;turn=.028*b*gain;fan=.055*b*gain;tail=.100*b*gain;leg=.070*a*gain;flap=.080*b*gain
 return [(0.,0.,0.),(body,0.,0.),(neck,0.,0.),(head,turn,0.),
  (fan*.2,fan*.35,-fan),(fan*.2,-fan*.35,fan),
  (0.,tail*.25,0.),(0.,tail*.55,0.),(0.,tail,tail*.15),
  (0.,-tail*.35,flap),(0.,tail*.35,-flap),
  (leg,0.,0.),(-leg*.75,0.,0.),(-leg*.85,0.,0.),(leg*.65,0.,0.)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])

def export_espeon(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='espeon' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Espeon anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Espeon slender sun cat, wide ears, ruby forehead lens and bifurcated tail skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'espeon/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'espeon','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'espeon','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'espeon','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'espeon/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'espeon','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'espeon.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'coherent feline gaze and forehead gem; broad independent ears; softly articulated limbs with planted paws; three-carrier tail with independent fork tips; source cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'espeon','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/espeon_sculpt.py','geometry_preservation':'original slender lilac sun cat, broad triangular ears with blue insets, silver violet gaze, ruby forehead lens, swept cheek wisps and a long forked tail',
    'animation_contract':'identity authoring root; four fixed ground paws; blended slender limbs; coherent head, muzzle, eyes, gem and cheek wisps; independent ears; continuously weighted three-carrier tail and independently blended fork tips; rotation-only source-cue clips'}

 if len(g.doc['accessors'])>256:raise ValueError('Espeon exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Espeon exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Espeon exceeds existing byte limit')
 return blob


def decode_espeon(blob):
 if len(blob)>MAX_BYTES or blob!=export_espeon():raise ValueError('noncanonical Espeon GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'espeon','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_espeon(path):
 with Path(path).open('rb') as f:return decode_espeon(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_espeon();(a.out/'espeon.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
