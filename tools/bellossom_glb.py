#!/usr/bin/env python3
"""Canonical Bellossom dancer with planted skirt tips and coherent flower crowns."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from bellossom_sculpt import sculpture,HEIGHT
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
REST_DIGEST='21bc079e44b6abd8d1012172d7fb1c909e679d18d143ceafc613241861785f57'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('torso',0,(0.,.30,0.)),('neck',1,(0.,.53,0.)),('head',2,(0.,.59,0.)),
 ('arm_right',1,(.081,.480,.005)),('arm_tip_right',4,(.281,.516,.060)),
 ('arm_left',1,(-.084,.482,.014)),('arm_tip_left',6,(-.152,.415,.164)),
 ('flower_left',3,(-.174,.800,.034)),('flower_right',3,(.174,.800,.034)))
LEAVES=tuple((kind+'_'+str(j),1,(.052*math.sin(a),y,.052*math.cos(a)))
 for j in range(5) for kind,a,y in [('under',j*math.tau/5+.18,.338),('inner',j*math.tau/5+math.pi/5,.368),('outer',j*math.tau/5,.395)])
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE+LEAVES)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
ARMS=(((.081,.480,.005),(.179,.482,.024),(.281,.516,.060),(.337,.550,.074)),
      ((-.084,.482,.014),(-.180,.452,.082),(-.152,.415,.164),(-.059,.414,.184),(.023,.437,.184)))
def path_fraction(point,path):
 best=(math.inf,0.);lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.
 for a,b,length in zip(path,path[1:],lengths):
  delta=[y-x for x,y in zip(a,b)];t=max(0.,min(1.,sum((p-x)*d for p,x,d in zip(point,a,delta))/(length*length)))
  distance=math.dist(point,[x+t*d for x,d in zip(a,delta)])
  best=min(best,(distance,(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(part,point):
 raw=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE);x,y,z=raw
 if part.startswith('Body /'):
  f=smoothstep(.17,.29,y);neck=smoothstep(.45,.53,y);head=smoothstep(.53,.60,y)
  return [0,1,2,3],[1.-f,f*(1.-neck),f*neck*(1.-head),f*neck*head]
 if part.startswith('Head /'):
  return [3,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Arm /'):
  side=0 if 'raised right' in part else 1;a,b=(4,5) if side==0 else (6,7)
  t=path_fraction(raw,ARMS[side]);attach=smoothstep(0.,.22,t);tip=smoothstep(.45,.85,t)
  return [1,a,b,0],[1.-attach,attach*(1.-tip),attach*tip,0.]
 if part.startswith('Skirt /'):
  layer=0 if 'underskirt' in part else 1 if 'inner leaf' in part else 2
  joint=10+int(part.rsplit(' ',1)[1])*3+layer
  support=smoothstep(.04,.13,y);bend=smoothstep(.06,.30,math.hypot(x,z))
  return [0,1,joint,0],[1.-support,support*(1.-bend),support*bend,0.]
 if part.startswith('Flower '):
  sign=int(part.split()[-2] if part.startswith('Flower petal /') else part.split()[-1]);return [8 if sign<0 else 9,0,0,0],[1.,0.,0.,0.]
 return [3,0,0,0],[1.,0.,0.,0.]
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Bellossom clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
 if clip=='idle':early=wave;late=math.sin(math.tau*t-.45)+math.sin(.45);strength=.010
 else:
  early=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  late=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)))
  strength=.026 if clip=='attack' else -.022
 a=strength*early;b=strength*late
 result=[(0.,0.,0.),(.25*a,.45*a,.30*a),(.30*b,0.,-.15*b),(.20*b,.45*b,0.),
         (0.,.60*a,-a),(.55*b,0.,-.35*b),(0.,-.60*a,a),(.55*b,0.,.35*b),
         (.40*b,0.,-.35*b),(.40*b,0.,.35*b)]
 for i in range(15):
  angle=(i//3)*math.tau/5;f=a*.55+b*.25
  result.append((f*math.cos(angle),.20*a,f*math.sin(angle)))
 return result
def export_bellossom(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='bellossom' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Bellossom anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite layered Bellossom dancer and standard skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'bellossom/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'bellossom','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'bellossom','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'bellossom','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(tuple(values),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'bellossom/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':g.accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'bellossom','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'bellossom.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted skirt tips; coherent head face and flowers; delayed arms and layered leaf response; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'bellossom','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/bellossom_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; planted skirt tips; fitted face follows head; coherent crown cups and petals; layered leaf bends; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Bellossom exceeds existing accessor limit')
 return g.bytes()


def decode_bellossom(blob):
 if len(blob)>MAX_BYTES or blob!=export_bellossom():raise ValueError('noncanonical Bellossom GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'bellossom','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_bellossom(path):
 with Path(path).open('rb') as f:return decode_bellossom(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_bellossom();(a.out/'bellossom.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
