#!/usr/bin/env python3
"""Canonical Croconaw/Feraligatr continuous-body skins with articulated jaws, arms and tails."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from crocodile_sculpt import sculpture,HEIGHT,posture_point,anatomy_point
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('croconaw','feraligatr');HEIGHTS={s:HEIGHT for s in SPECIES}
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'croconaw': '7edb832ce86d2d9c0e093fbcec14c2b56312f4eb80e98df5bbf08297eec45677', 'feraligatr': 'd7059c539638a31894a34a8560250dc2cf5dbbf8f6905bd9690b7c18c69142ae'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,hi-lo
def normalized(species,point):
 lo,span=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*HEIGHT/span for i,v in enumerate(posture_point(species,point)))
def arms(species):
 adult=species=='feraligatr'
 return tuple(((s*.303,.794,-.012),(s*.400,.630,.060),(s*.470,.660,.250)) if adult else ((s*.239,.690,-.006),(s*.325,.590,.030),(s*.365,.535,.160)) for s in (-1,1))
def joints(species):
 adult=species=='feraligatr'
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.48 if adult else .43,-.02)),('head',1,(0.,.91 if adult else .81,.01)),('jaw',2,(0.,.88 if adult else .80,.04))]
 for side,points in zip(('left','right'),arms(species)):
  base=len(raw);raw += [('arm_'+side,1,points[0]),('elbow_'+side,base,points[1]),('wrist_'+side,base+1,points[2])]
 raw += [('leg_left',0,(-.220,.330,-.033) if adult else (-.225,.276,-.011)),('leg_right',0,(.220,.330,-.033) if adult else (.225,.276,-.011)),('tail',1,(0.,.253,-.160) if adult else (0.,.214,-.179)),('tail_tip',12,(.018,.220,-.384) if adult else (.021,.220,-.371))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def body_weights(species,y):
 a=smoothstep(.16,.34,y);b=smoothstep(.88 if species=='feraligatr' else .76,.98 if species=='feraligatr' else .86,y)
 return [0,1,2,0],[1.-a,a*(1.-b),a*b,0.]
def arm_weights(species,side,point):
 a,b,c=arms(species)[int(side>0)];base=4 if side<0 else 7;d=tuple(c[i]-a[i] for i in range(3));t=sum((point[i]-a[i])*d[i] for i in range(3))/sum(v*v for v in d)
 upper=smoothstep(-.10,.20,t);elbow=smoothstep(.28,.60,t);wrist=smoothstep(.76,.96,t)
 return [1,base,base+1,base+2],[1.-upper,upper*(1.-elbow),upper*elbow*(1.-wrist),upper*elbow*wrist]
def leg_weights(species,side,y):
 a=smoothstep(.08,.30 if species=='feraligatr' else .265,y);b=.25*math.sin(math.pi*a)**2
 return [10 if side<0 else 11,0,1,0],[b,(1.-a)*(1.-b),a*(1.-b),0.]
def tail_weights(species,point):
 x,y,z=point;a=smoothstep(.15,.32,-z);b=smoothstep(.40,.72 if species=='feraligatr' else .60,-z);support=smoothstep(.16,.34,y)
 return [0,1,12,13],[(1.-a)*(1.-support),(1.-a)*support,a*(1.-b),a*b]
def skin_weights(species,part,position):
 lo,span=authoring_transform(species);point=anatomy_point(species,tuple(v*span/HEIGHT+(lo if i==1 else 0.) for i,v in enumerate(position)));x,y,z=point;adult=species=='feraligatr'
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Foot /'):return rigid(0)
 if part.startswith(('Leg /','Armor / limb plate')):
  side=int(part.split()[-2] if part.startswith('Armor /') else part.split()[-1])
  if not part.startswith('Armor /') or part.endswith(' 4'):return leg_weights(species,side,y)
  return arm_weights(species,side,point)
 if part.startswith('Hand /'):return rigid(6 if part.split()[-2 if part.startswith(('Hand / curved','Hand / white')) else -1]=='-1' else 9)
 if part.startswith('Arm /'):return arm_weights(species,int(part.split()[-1]),point)
 if part.startswith(('Tail /','Crest / tail')):return tail_weights(species,point)
 if part.startswith(('Crest / broad jagged cranial','Crest / rear jagged cranial','Crest / three-point cranial','Eye /','Nostril /','Jaw / broad upper','Tooth / upper')):return rigid(2)
 if part.startswith(('Jaw / fitted pale lower','Tooth / lower','Mouth / rounded pink tongue')):return rigid(3)
 if part.startswith(('Mouth /','Jaw / pale mouth corner')):
  # Blend over the entire oral cup. A narrow band through its thin front rim
  # sheared adjacent inner/outer walls when the jaw rotated independently.
  a=smoothstep(.75 if adult else .77,.93 if adult else .89,y);return [3,2,0,0],[1.-a,a,0.,0.]
 if part.startswith('Crest / dorsal'):return rigid(1)
 return body_weights(species,y)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid crocodile clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*14
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.;jaw=.008*(.5-.5*math.cos(math.tau*t))
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3;jaw=(.026 if clip=='attack' else .012)*a
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.016*a,0.),(.010*b,.018*a,0.),(jaw,0.,0.)]
 for side in (-1,1):out += [(-.022*b,side*.025*a,-side*.022*b),(.040*a,side*.020*b,side*.016*a),(.030*b,side*.035*a,side*.020*b)]
 out += [(0.,-.018*a,0.),(0.,.018*a,0.),(.010*b,.040*a,0.),(.010*a,.055*b,0.)]
 return out

def pack_parts(species,model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name=source['part'];key=None;label=None;words=name.split()
  if name.startswith('Hand /'):
   side=words[-2] if name.startswith(('Hand / curved','Hand / white')) else words[-1];kind='white claws' if name.startswith('Hand / white') else 'blue palm and fingers';key=('hand',side,kind);label='Hand / '+kind+' '+side
  elif name.startswith('Foot /'):
   side=words[-1] if name.startswith('Foot / raised') else words[-2];kind='white claws' if name.startswith('Foot / white') else 'blue instep and toes';key=('foot',side,kind);label='Foot / '+kind+' '+side
  elif name.startswith('Tooth /'):
   row='upper' if name.startswith('Tooth / upper') else 'lower';key=('tooth',row);label='Tooth / '+row+' rooted teeth'
  elif name.startswith('Armor / limb plate') and not name.endswith(' 4'):
   side=words[-2];key=('arm_armor',side);label='Armor / four fitted arm plates '+side
  elif name.startswith('Armor / dorsal'):
   key=('back',);label='Armor / three fitted dorsal plates'
  elif name.startswith('Eye /'):
   kind=' '.join(words[2:-1]);key=('eye',kind);label='Eye / paired '+kind
  elif name.startswith('Belly / blue'):
   key=('islands',);label='Belly / four blue islands'
  if key is None:parts.append((source,{'source_order':order}));continue
  if key not in groups:
   joined={'part':label,'positions':[],'normals':[],'indices':[],'base_color':source['base_color']};metadata={'source_parts':[]};groups[key]=(joined,metadata);parts.append((joined,metadata))
  joined,metadata=groups[key];assert joined['base_color']==source['base_color'];start=len(joined['positions'])//3;first=len(joined['indices'])
  metadata['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])})
  joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGESTS={'croconaw': 'e7d36b1fdaa47269e689e0e20eb3af13b4a950875d354e5705039bca66816eba', 'feraligatr': '5115b470f5bc4d168b48e84117b42884e5014c40cb6de97e1ec27dba957b8149'}

def export_crocodile(species,model=None):
 if species not in SPECIES:raise ValueError('unknown crocodile species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed crocodile neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite continuous crocodile body, articulated jaw, arms and tail skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed crocodile geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed splayed feet and toe claws; continuous body and coherent paint; independently hinged lower jaw; articulated upper arms, elbows, wrists and tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/crocodile_sculpt.py','geometry_preservation':'original continuous round juvenile and broad-shouldered adult crocodiles; fitted gazes and bellies; broad jaws and rooted teeth; curved arms, fingers, legs and counterbalancing tails; every armor plate and crest retained',
    'animation_contract':'identity floor root; fixed feet and matching claws; blended pelvis, chest and neck; coherent fitted gaze; independent jaw, upper arms, elbows, wrists and two tail carriers; teeth, fitted plates and tail crests follow their true anatomical owners; lossless packing preserves every original solid and its weights; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'crocodile exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('crocodile exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('crocodile exceeds existing byte limit')
 return blob


def decode_crocodile(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_crocodile(species):raise ValueError('noncanonical crocodile GLB')
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


def read_crocodile(path,species):
 with Path(path).open('rb') as f:return decode_crocodile(f.read(MAX_BYTES+1),species)
def export_croconaw(model=None):return export_crocodile('croconaw',model)
def read_croconaw(path):return read_crocodile(path,'croconaw')
def export_feraligatr(model=None):return export_crocodile('feraligatr',model)
def read_feraligatr(path):return read_crocodile(path,'feraligatr')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_crocodile(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
