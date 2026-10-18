#!/usr/bin/env python3
"""Tentacool and Tentacruel standard skins with grounded articulated tentacles."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from tentacle_sculpt import sculpture as sculpt
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

from tentacle_sculpt import SPECIES,HEIGHTS,paths,normalized,transform
from bovine_sculpt import smooth
REST_DIGESTS={'tentacool': 'acb31c9286cd89fc4218aec6bf9706b51fad8a1d0a967ac0e1a158be4b9a1798', 'tentacruel': 'c21b730c9a3ea4b05d3a7031fff2c30ace66889c6948516b36fe2698a225117b'}
PARTS={s:tuple(p['part'] for p in sculpt(s)['primitives']) for s in SPECIES};MAX_BYTES=1024*1024
sculpture=sculpt

def joints(s):
 result=[('root',None,(0.,0.,0.)),('bell',0,normalized(s,(0.,.69,0.)))]
 for j,path in enumerate(paths(s)):
  result.extend(((f'tentacle_{j}_base',1,normalized(s,path[0])),(f'tentacle_{j}_bend',2+2*j,normalized(s,path[2]))))
 if s=='tentacruel':
  for sign in (-1,1):result.append((f'arm_{sign}',1,normalized(s,(sign*.28,.67,.01))))
 return tuple(result)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.4,85),('attack',1.,65),('hit',1.,65))
@lru_cache(maxsize=12)
def path_samples(s,j):
 path=paths(s)[j];pp,rr=smooth(path,[.033,.031,.025,.019,.012,.003]);dist=[0.]
 for a,b in zip(pp,pp[1:]):dist.append(dist[-1]+math.dist(a,b))
 return pp,dist

def path_fraction(s,j,point):
 lo,k=transform(s);point=tuple(v/k+(lo if a==1 else 0.) for a,v in enumerate(point));pp,dist=path_samples(s,j);best=(math.inf,0.)
 for index,(a,b) in enumerate(zip(pp,pp[1:])):
  delta=[v-u for u,v in zip(a,b)];length=sum(v*v for v in delta);t=max(0.,min(1.,sum((v-u)*d for v,u,d in zip(point,a,delta))/length));closest=[v+t*d for v,d in zip(a,delta)]
  best=min(best,(math.dist(point,closest),dist[index]+t*(dist[index+1]-dist[index])))
 return best[1]/dist[-1]

def skin_weights(species,part,position):
 if part.startswith('Tentacle /'):
  j=int(part.split()[-1]);t=path_fraction(species,j,position);base=smoothstep(.02,.25,t);bend=smoothstep(.25,.55,t);floor=smoothstep(.60,.84,t)
  return [1,2+2*j,3+2*j,0],[(1.-base)*(1.-floor),base*(1.-bend)*(1.-floor),base*bend*(1.-floor),floor]
 if part.startswith('Arm /'):
  owner=len(JOINTS[species])-2+(int(part.split()[-1])==1)
  lo,k=transform(species);y=position[1]/k+lo;w=1.-smoothstep(.57,.67,y)
  return [1,owner,0,0],[1.-w,w,0.,0.]
 return [1,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid jelly clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 wave=math.sin(math.tau*t);early=envelope(t,((0.,0.),(.18,1.),(.40,.7),(.67,.10),(1.,0.)));late=envelope(t,((0.,0.),(.28,1.),(.52,.6),(.78,-.08),(1.,0.)))
 sign=1. if clip=='attack' else -.7
 out=[(0.,0.,0.),(.006*wave,.012*wave,0.) if clip=='idle' else (sign*.015*early,sign*.018*late,0.)]
 for j in range(len(paths(species))):
  phase=j*.57;w=math.sin(math.tau*t-phase)+math.sin(phase)
  if clip=='idle':out.extend(((.012*w,.014*w,.011*w),(-.016*w,.009*w,-.015*w)))
  else:out.extend(((sign*.025*early,sign*.020*late,sign*.018*early),(-sign*.032*late,sign*.015*early,-sign*.025*late)))
 if species=='tentacruel':
  for side in (-1,1):out.append((0.,side*(.024*wave if clip=='idle' else sign*.065*early),side*(.012*wave if clip=='idle' else sign*.030*late)))
 return out


def export_tentacle(species,model=None):
 if species not in SPECIES:raise ValueError('unknown jelly species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed jelly anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite original flared jelly bells and rooted tentacle standard skins v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'rooted tentacle flexion; coherent mantle gaze and bell lobes; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHTS[species],'body_reference_height':HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/tentacle_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; pinned distal tentacles; coherent bell lobes and mantle gaze; independent two-joint tendrils and side feelers; rotation-only source-cue clips'}
 if (len(g.doc['accessors'])>256 or len(primitives)>64 or sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices'])//3 for p in model['primitives'])>40000):raise ValueError('jelly exceeds existing limits')
 return g.bytes()


def decode_tentacle(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_tentacle(species):raise ValueError('noncanonical jelly GLB')
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


def read_tentacle(path,species):
 with Path(path).open('rb') as f:return decode_tentacle(f.read(MAX_BYTES+1),species)
def export_tentacool(model=None):return export_tentacle('tentacool',model)
def export_tentacruel(model=None):return export_tentacle('tentacruel',model)
def read_tentacool(path):return read_tentacle(path,'tentacool')
def read_tentacruel(path):return read_tentacle(path,'tentacruel')
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:
  blob=export_tentacle(s);(args.output/(s+'.glb')).write_bytes(blob);print(s,len(blob),'bytes',len(PARTS[s]),'parts',len(JOINTS[s]),'joints',REST_DIGESTS[s])
if __name__=='__main__':main()
