#!/usr/bin/env python3
"""Canonical Magmar skin with supported claws, open bill and articulated flame crowns."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
import magmar_sculpt
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('magmar',);HEIGHTS={'magmar':magmar_sculpt.HEIGHT}
def sculpture(species='magmar',normalize=True):
 if species!='magmar':raise ValueError('invalid spitfire species')
 return magmar_sculpt.sculpture(normalize)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'magmar': 'beed80700a65f3864fa4e3facfffb0c8d6e4aab03e78ec41e1e0228f53b46d50'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,hi-lo
def normalized(species,point):
 lo,span=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))
def joints(species='magmar'):
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.57,-.01)),('neck',1,(0.,.82,.012)),('head',2,(0.,.94,.008))]
 for side,s in (('left',-1),('right',1)):
  base=len(raw);raw += [('arm_'+side,1,(s*.195,.795,-.015)),('elbow_'+side,base,(s*.34,.79,.015)),('wrist_'+side,base+1,(s*.485,.78,.024))]
 raw += [('leg_left',0,(-.19,.30,.02)),('leg_right',0,(.19,.30,.02)),('crown_left',3,(-.115,1.11,.016)),('crown_right',3,(.115,1.11,.016)),('tail_base',1,(0.,.372,-.104)),('tail_tip',14,(-.067,.383,-.432)),('fire_tip',15,(.012,.645,-.556))]
 return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for n,parent,p in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def body_weights(y):
 a=smoothstep(.77,.85,y);b=smoothstep(.87,.94,y)
 return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
def skin_weights(species,part,position):
 lo,span=authoring_transform(species);x,y,z=(v*span/HEIGHTS[species]+(lo if i==1 else 0.) for i,v in enumerate(position))
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Toe claw /')):return rigid(0)
 if part.startswith(('Head /','Forehead lobe /','Eye socket /','Eye /','Pupil /','Bill /','Mouth back /','Tongue /')) or part.startswith('Dorsal point /') and part.endswith(' 2'):return rigid(3)
 # Thin flame banks and their fitted layers share a carrier: differential
 # skin gradients can fold narrow perimeter triangles.
 if part.startswith(('Crown flame /','Crown glow /')):
  side=part.split()[-2] if part.startswith('Crown glow /') else part.split()[-1]
  return rigid(12 if side=='-1' else 13)
 if part.startswith(('Shoulder /','Arm /','Cuff /','Hand /','Hand claw /')):
  side=part.split()[-2] if part.startswith(('Cuff /','Hand claw /')) else part.split()[-1];base=4 if side=='-1' else 7
  if part.startswith(('Hand /','Hand claw /')):return rigid(base+2)
  x=abs(x);upper=smoothstep(.13,.22,x);elbow=smoothstep(.25,.38,x);wrist=smoothstep(.41,.49,x)
  return [1,base,base+1,base+2],[1.-upper,upper*(1.-elbow),upper*elbow*(1.-wrist),upper*elbow*wrist]
 if part.startswith(('Thigh /','Shin band /')):
  a=smoothstep(.07,.32,y);b=.25*math.sin(math.pi*a)**2
  return [0,10 if part.endswith('-1') else 11,1,0],[(1.-a)*(1.-b),b,a*(1.-b),0.]
 if part.startswith(('Tail /','Tail flame /','Tail glow /')):
  a=smoothstep(.12,.28,-z);b=smoothstep(.35,.52,-z)
  if part.startswith('Tail /'):return [1,14,15,0],[1.-a,a*(1.-b),a*b,0.]
  return rigid(16)
 return body_weights(y)
def pose_angles(species,clip,t):
 if species!='magmar' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid spitfire clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*17
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.012*a,0.),(0.,.012*b,0.),(.008*b,.018*a,0.)]
 for s in (-1,1):out += [(.012*b,s*.014*a,-s*.025*b),(.018*a,s*.018*b,s*.012*a),(.014*b,s*.020*a,s*.012*b)]
 out += [(0.,-.024*a,0.),(0.,.024*a,0.),(0.,0.,-.022*b),(0.,0.,.022*b),(0.,.026*a,0.),(0.,.035*b,0.),(0.,.0025*b,.025*b)]
 return out

def pack_parts(species,model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name=source['part'];kind=name.split(' /')[0];words=name.split();key=None
  if kind in ('Hand claw','Toe claw','Cuff'):key=(kind,words[-2])
  elif kind in ('Eye socket','Eye','Pupil','Crown flame','Crown glow','Tail glow'):key=(kind,)
  if key is None:parts.append((source,{'source_order':order}));continue
  if key not in groups:
   joined={'part':kind+' / complete spitfire assembly'+(' '+key[1] if len(key)>1 else ''),'positions':[],'normals':[],'indices':[],'base_color':source['base_color']};metadata={'source_parts':[]};groups[key]=(joined,metadata);parts.append((joined,metadata))
  joined,metadata=groups[key];assert joined['base_color']==source['base_color'];start=len(joined['positions'])//3;first=len(joined['indices'])
  metadata['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])})
  joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGESTS={'magmar': '3463aa43d0d0b9d5d979c0a55071283a3ef1a1893d053a2ffa97e5361dd0c83d'}
def export_spitfire(species,model=None):
 if species not in SPECIES:raise ValueError('unknown spitfire species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed spitfire neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite open-bill spitfire with articulated crown flames skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed spitfire geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed feet and matching toe claws; coherent open bill and fitted flame belly; articulated shoulders, elbows, wrists, twin crown flames and tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/'+species+'_sculpt.py','geometry_preservation':'original twin-crowned spitfire with open bill, tongue, toothed cuffs, flame-shaped belly and long burning tail; adaptive fitted surfaces and focused round-body sampling',
    'animation_contract':'identity floor root; fixed feet and toe claws; coherent bill, tongue, lobes and fitted gaze; rooted cuffs use the forelimb field; independent shoulders, elbows, wrists, supported legs, crown flames and tail carriers; lossless packing preserves every source solid; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'spitfire exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('spitfire exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('spitfire exceeds existing byte limit')
 return blob


def decode_spitfire(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_spitfire(species):raise ValueError('noncanonical spitfire GLB')
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


def read_spitfire(path,species):
 with Path(path).open('rb') as f:return decode_spitfire(f.read(MAX_BYTES+1),species)
def export_magmar(model=None):return export_spitfire('magmar',model)
def decode_magmar(blob):return decode_spitfire(blob,'magmar')
def read_magmar(path):return read_spitfire(path,'magmar')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_magmar();a.output.write_bytes(blob);print(len(blob),'bytes')
if __name__=='__main__':main()
