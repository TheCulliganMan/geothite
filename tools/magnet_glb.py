#!/usr/bin/env python3
"""Canonical magnetic steel orbs with open hinged magnets and independently rotating screws."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from magnet_sculpt import sculpture,SPECIES,HEIGHT,UNIT_POSES,unit as magnetic_unit
HEIGHTS={s:HEIGHT for s in SPECIES}
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'magnemite': '616a56a4324733a3df92c7c5c58060ec77941d02839157b72045301df1fe43b3', 'magneton': 'ed29fd704f19641beba12fb100013b102bc69d55a3e8d95e45871ed52c8607e0'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def unit_point(species,index,p):
 cx,cy,cz,angle=UNIT_POSES[species][index];c,t=math.cos(angle),math.sin(angle);x,y,z=p
 return (cx+x*c-y*t,cy+x*t+y*c,cz+z)
@lru_cache(maxsize=1)
def face_centers():
 ps=magnetic_unit();out=[]
 for k in range(2):
  p=next(p for p in ps if p['part'].startswith('Face bolt /') and p['part'].endswith(' '+str(k)))
  out.append(tuple((min(p['positions'][i::3])+max(p['positions'][i::3]))/2 for i in range(3)))
 return tuple(out)
def joints(species):
 raw=[('root',None,(0.,0.,0.))]
 for index,pose in enumerate(UNIT_POSES[species]):
  base=len(raw);raw += [('body_'+str(index),0,unit_point(species,index,(0.,0.,0.)))]
  raw += [('magnet_left_'+str(index),base,unit_point(species,index,(-.170,0.,0.))),('magnet_right_'+str(index),base,unit_point(species,index,(.170,0.,0.))),('crown_'+str(index),base,unit_point(species,index,(0.,.195,0.)))]
  raw += [('face_bolt_'+str(index)+'_'+str(k),base,unit_point(species,index,p)) for k,p in enumerate(face_centers())]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 name,index=part.rsplit(' / unit ',1);base=1+6*int(index)
 if name.startswith(('Magnet /','Pole /')):
  sign=int(name.split()[-2] if name.startswith('Pole /') else name.split()[-1]);owner=base+1 if sign<0 else base+2
 elif name.startswith(('Crown shaft /','Crown head /','Crown slot /','Thread /')):owner=base+3
 elif name.startswith(('Face bolt /','Face slot /')):owner=base+4+int(name.split()[-1])
 else:owner=base
 return [owner,0,0,0],[1.,0.,0.,0.]
def axis_rotation(axis,angle):
 a=math.sin(angle/2);return [v*a for v in axis]+[math.cos(angle/2)]
def pose_rotations(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid magnet clip/time')
 if t in (0.,1.):return [[0.,0.,0.,1.] for _ in JOINTS[species]]
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength;out=[[0.,0.,0.,1.]]
 for index,(_,_,_,angle) in enumerate(UNIT_POSES[species]):
  phase=.65+index*2.11
  if clip=='idle':v=math.sin(math.tau*t+phase)-math.sin(phase);w=math.sin(math.tau*t+phase+.7)-math.sin(phase+.7)
  else:v=.65*a*math.cos(phase)+.35*b*math.sin(phase);w=.45*a*math.sin(phase)+.55*b*math.cos(phase)
  # Global vertical hinges keep all posed authoring floors stable. Crowns
  # twist about each original unit's actual screw axis after its Z turn.
  out += [axis_rotation((0.,1.,0.),.012*v),axis_rotation((0.,1.,0.),.045*v),axis_rotation((0.,1.,0.),-.040*w),axis_rotation((-math.sin(angle),math.cos(angle),0.),.14*v),axis_rotation((0.,0.,1.),.12*w),axis_rotation((0.,0.,1.),-.12*v)]
 return out

def pack_parts(species,model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name,index=source['part'].rsplit(' / unit ',1)
  if name.startswith('Sphere /'):key='body';label='Sphere / blue-silver body'
  elif name.startswith(('Eye border /','Pupil /')):key='eye_dark';label='Eye dark / fitted rim and pupil'
  elif name.startswith('Eye white /'):key='eye_white';label='Eye white / fitted pale eye'
  elif name.startswith('Face bolt /'):key='face_metal';label='Face bolts / two beveled steel discs'
  elif name.startswith(('Face slot /','Crown slot /')):key='slots';label='Slots / three fitted dark screw crosses'
  elif name.startswith(('Crown shaft /','Thread /','Crown head /')):key='crown';label='Crown / threaded steel shaft and cap'
  elif name.startswith('Magnet /'):key='magnets';label='Magnets / two open beveled horseshoes'
  elif name.startswith('Pole /'):
   key='pole_red' if 'flat red ' in name else 'pole_blue';label='Poles / '+('red' if key=='pole_red' else 'blue')+' terminals'
  else:raise ValueError('unreviewed magnetic part')
  key=(int(index),key);label+=' / unit '+index
  if key not in groups:
   joined={'part':label,'positions':[],'normals':[],'indices':[],'base_color':source['base_color']};metadata={'source_parts':[]};groups[key]=(joined,metadata);parts.append((joined,metadata))
  joined,metadata=groups[key];assert joined['base_color']==source['base_color'];start=len(joined['positions'])//3;first=len(joined['indices'])
  metadata['source_parts'].append({'part':source['part'],'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])})
  joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGESTS={'magnemite': 'fcd9ef30d31caba9b51c6fc627b4bf0b68a20edf961777c9477acdb7b96904ce', 'magneton': '432c67fcf83876db5de5b5e9c0dc9ae27df2a6180760e7a0325e394dfccb7f8f'}

def export_magnet(species,model=None):
 if species not in SPECIES:raise ValueError('unknown magnet species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed magnet neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite magnetic steel orbs, coherent eyes, independent crown and face screws, hinged open horseshoes skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed magnet geometry')
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
   rotations=[v for i in range(count) for v in pose_rotations(species,clip,i/(count-1))[joint]]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':species+'.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'coherent orb eyes, attached open magnets and colored poles, crowns twisting on actual axes, independent face bolts and intact three-orb adult cluster; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/magnet_sculpt.py','geometry_preservation':'original blue-silver steel orbs, broad fitted eyes with tiny pupils, open beveled horseshoes with red/blue poles, three slotted screws per unit and a 3.5-turn crown thread; three physically joined adult units',
    'animation_contract':'identity root; independently yawing rigid orbs with coherent eyes; rigid hinged horseshoes and attached poles; crowns twist about each original unit axis with shaft, helix, cap and slot coherent; two independent face-bolt carriers and slots per orb; physical cluster retained; existing intentional battle and world float preserved; lossless packing retains every logical solid and source weights; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'magnet exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('magnet exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('magnet exceeds existing byte limit')
 return blob


def decode_magnet(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_magnet(species):raise ValueError('noncanonical magnet GLB')
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


def read_magnet(path,species):
 with Path(path).open('rb') as f:return decode_magnet(f.read(MAX_BYTES+1),species)
def export_magnemite(model=None):return export_magnet('magnemite',model)
def read_magnemite(path):return read_magnet(path,'magnemite')
def export_magneton(model=None):return export_magnet('magneton',model)
def read_magneton(path):return read_magnet(path,'magneton')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_magnet(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
