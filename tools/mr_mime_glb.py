#!/usr/bin/env python3
"""Original barrier mime with ten glove digits, rooted fans and fixed curled slippers."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
import mr_mime_sculpt as source
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('mr__mime',);HEIGHTS={'mr__mime':source.HEIGHT}
def sculpture(species='mr__mime',normalize=True):
 if species!='mr__mime':raise ValueError('unknown barrier mime')
 return source.sculpture(normalize)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=1)
def authoring_model(species='mr__mime'):return sculpture(species)
REST_DIGESTS={'mr__mime':'120668b1f7eb02e42729ae3939a5c4e3ff1258f6ec57b837aec6f3fda6f1a8e9'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=1)
def authoring_transform(species='mr__mime'):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,hi-lo
def normalized(species,point):
 lo,span=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))
def joints(species='mr__mime'):
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.65,0.)),('neck',1,(0.,.94,0.)),('head',2,(0.,1.16,0.))]
 for name,s in (('left',-1),('right',1)):
  points=source.arm_points(s);base=len(raw);raw += [('arm_'+name,1,points[0]),('elbow_'+name,base,points[1]),('wrist_'+name,base+1,points[-1])]
 raw += [('leg_left',1,(-.211,.412,0.)),('leg_right',1,(.211,.412,0.)),('knee_left',10,(-.307,.255,.064)),('knee_right',11,(.307,.255,.064)),('fan_left',3,(-.20,1.28,-.025)),('fan_right',3,(.20,1.28,-.025))]
 for s,name,wrist in ((-1,'left',6),(1,'right',9)):
  raw += [(f'finger_{name}_{i}',wrist,source.digit_points(s,i)[0]) for i in range(5)]
 return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for n,parent,p in raw)
JOINTS={'mr__mime':joints()};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def body_weights(y):
 a=smoothstep(.80,.97,y);b=smoothstep(1.00,1.11,y)
 return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
def skin_weights(species,part,position):
 lo,span=authoring_transform(species);point=tuple(v*span/HEIGHTS[species]+(lo if i==1 else 0.) for i,v in enumerate(position));x,y,z=point
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Shoe /','Shoe curl /')):return rigid(0)
 if part.startswith(('Head /','Upper lip /','Lower lip /','Mouth back /','Tongue /','Mouth lining /','Eye /','Pupil /','Eye glint /','Cheek /')):return rigid(3)
 if part.startswith(('Finger /','Fingertip /','Finger pad /')):
  side,index=map(int,part.split()[-2:]);return rigid((16 if side<0 else 21)+index)
 side=-1 if part.endswith('-1') else 1
 if part.startswith('Ear fan /'):return rigid(14 if side<0 else 15)
 if part.startswith('Palm /'):return rigid(6 if side<0 else 9)
 if part.startswith('Shoulder /'):return rigid(4 if side<0 else 7)
 if part.startswith('Arm /'):
  points=source.arm_points(side);axis=[b-a for a,b in zip(points[0],points[-1])];t=sum((point[i]-points[0][i])*axis[i] for i in range(3))/sum(v*v for v in axis)
  base=4 if side<0 else 7;a=smoothstep(.15,.62,t);b=smoothstep(.66,.90,t)
  return [base,base+1,base+2,0],[1.-a,a*(1.-b),a*b,0.]
 hip=10 if side<0 else 11;knee=12 if side<0 else 13
 if part.startswith('Hip /'):return rigid(hip)
 if part.startswith('Knee /'):return rigid(knee)
 if part.startswith('Thigh /'):
  a=smoothstep(.27,.39,y);return [knee,hip,0,0],[1.-a,a,0.,0.]
 if part.startswith('Shin /'):
  a=smoothstep(.10,.24,y);return [0,knee,0,0],[1.-a,a,0.,0.]
 return body_weights(y)
def pose_angles(species,clip,t):
 if species!='mr__mime' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid mime clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*26
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.012*a,0.),(.008*b,.005*a,0.),(.008*b,.014*a,0.)]
 for s in (-1,1):out += [(.012*b,s*.014*a,-s*.012*b),(.018*a,s*.012*b,s*.010*a),(.012*b,s*.018*a,s*.012*b)]
 out += [(0.,-.014*a,-.006*b),(0.,.014*a,.006*b),(.010*a,-.012*b,0.),(.010*a,.012*b,0.),(0.,0.,-.018*a),(0.,0.,.018*a)]
 for s in (-1,1):
  for i,deg in enumerate(source.DIGIT_ANGLES):
   theta=math.radians(deg);curl=(.014+.003*i)*a;spread=s*.012*b*(i-2)/2
   out.append((curl*math.sin(theta),-curl*math.cos(theta),spread))
 return out

def pack_parts(species,model):
 parts=[];groups={}
 for order,part in enumerate(model['primitives']):
  name=part['part'];kind=name.split(' /')[0];key=(kind,) if kind in ('Finger','Fingertip','Finger pad','Eye','Pupil','Eye glint','Cheek','Ear fan','Shoulder','Hip','Knee') else None
  # Walking IK consumes complete runtime parts; bilateral supports stay separate.
  if kind in ('Hip','Knee'):key=None
  if key is None:parts.append((part,{'source_order':order}));continue
  if key not in groups:
   joined={'part':kind+' / complete barrier-mime assembly','positions':[],'normals':[],'indices':[],'base_color':part['base_color']};meta={'source_parts':[]};groups[key]=(joined,meta);parts.append((joined,meta))
  joined,meta=groups[key];assert joined['base_color']==part['base_color'];start=len(joined['positions'])//3;first=len(joined['indices']);meta['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(part['positions'])//3,'index_start':first,'index_count':len(part['indices'])});joined['positions']+=part['positions'];joined['normals']+=part['normals'];joined['indices'] += [i+start for i in part['indices']]
 return parts
PACKED_REST_DIGESTS={'mr__mime': 'd583990f629188f68ad7098a02d6fca3b854f80187ac1bc00ac935bbf9bd1ab2'}
def export_mime(species,model=None):
 if species not in SPECIES:raise ValueError('unknown mime species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed mime neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite original barrier mime with ten independent glove digits and supported slippers skin v1'
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
  # Preserve exact f32 bytes, including signed zero in authored normals.
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 packed_parts=pack_parts(species,model)
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed mime geometry')
 for part,metadata in packed_parts:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':species+'/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  ranges=metadata.get('source_parts',[{'part':part['part'],'vertex_start':0,'vertex_count':len(part['positions'])//3}])
  for source in ranges:
   for i in range(source['vertex_start']*3,(source['vertex_start']+source['vertex_count'])*3,3):
    point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(species,source['part'],point);q=quantized_weights(ws)
    ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':species,'primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(species,clip,i/(count-1))[joint])]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':species+'.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed complete slippers; coherent recessed smile and gaze; articulated arms, wrists, ten glove digits, supported legs and scalloped fans; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/mr_mime_sculpt.py','geometry_preservation':'original asymmetric cream mime; physical smiling recess, five complete digits per glove with fitted pads, scalloped planar fans and curled blue slippers; adaptive fitted surfaces and focused round sampling',
    'animation_contract':'identity floor root; fixed slippers and curls; coherent face, gaze and recessed smile; independent shoulder/elbow/wrist chains with supported legs; ten whole glove digits and two scalloped fan carriers preserve fitted markings and roots; lossless source-solid packing; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'mime exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('mime exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('mime exceeds existing byte limit')
 return blob


def decode_mime(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_mime(species):raise ValueError('noncanonical mime GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 parts=[]
 for p,extra in zip(doc['meshes'][0]['primitives'],doc['extras']['source_part_ranges']):
  positions=values(p['attributes']['POSITION']);normals=values(p['attributes']['NORMAL']);indices=values(p['indices']);color=doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
  sources=extra.get('source_parts',[{'part':p['extras']['part'],'order':extra.get('source_order'),'vertex_start':0,'vertex_count':len(positions)//3,'index_start':0,'index_count':len(indices)}])
  for source in sources:
   a=source['vertex_start'];b=a+source['vertex_count'];i=source['index_start'];j=i+source['index_count']
   parts.append((source['order'],{'part':source['part'],'positions':positions[3*a:3*b],'normals':normals[3*a:3*b],'indices':[k-a for k in indices[i:j]],'base_color':color}))
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':[p for order,p in sorted(parts,key=lambda pair:pair[0])]}


def read_mime(path,species):
 with Path(path).open('rb') as f:return decode_mime(f.read(MAX_BYTES+1),species)
def export_mr_mime(model=None):return export_mime('mr__mime',model)
def decode_mr_mime(blob):return decode_mime(blob,'mr__mime')
def read_mr_mime(path):return read_mime(path,'mr__mime')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_bytes(export_mr_mime())
if __name__=='__main__':main()
