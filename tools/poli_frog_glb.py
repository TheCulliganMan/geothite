#!/usr/bin/env python3
"""Original distinct spiral frog creatures with per-limb support packing and fitted markings."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from poli_frog_sculpt import sculpture,SPECIES,HEIGHTS,arm_points,arm_curve
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
@lru_cache(maxsize=3)
def authoring_model(species):return sculpture(species)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
REST_DIGESTS={'poliwhirl': '331c07cd646e2dd21412c2243507420d69862a69a0acaac3086eb3cafbbe4394', 'poliwrath': '3628cbf3a335a904aebdfa48f9ecc48d68033f9449107546b6fe256e7364d09e', 'politoed': 'dce0949835912403468cae4938d451fe36f18b0393a754c5a1108aab46df69ab'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=3)
def authoring_transform(species):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,hi-lo
def normalized(species,point):
 lo,span=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))
def joints(species):
 toad=species=='politoed';wrath=species=='poliwrath'
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.43 if toad else .52 if wrath else .48,0.))]
 if toad:raw += [('head',1,(0.,.70,.04))]
 ex,ey,ez=(.16,.94,.09) if toad else (.195,.87,.015) if wrath else (.175,.81,.015)
 raw += [('eye_left',2 if toad else 1,(-ex,ey,ez)),('eye_right',2 if toad else 1,(ex,ey,ez))]
 for side in (-1,1):
  first=len(raw);suffix='left' if side<0 else 'right'
  points=arm_points(species,side)
  raw += [('arm_'+suffix,1,points[0]),('forearm_'+suffix,first,points[1]),('wrist_'+suffix,first+1,points[2])]
 hx=.18 if toad else .215 if wrath else .18
 raw += [('leg_left',1,(-hx,.225 if toad else .255,-.018)),('leg_right',1,(hx,.225 if toad else .255,-.018))]
 if toad:raw += [('crest_base',2,(0.,.93,-.02)),('crest_tip',13,(.035,1.205,0.))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for name,parent,p in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
@lru_cache(maxsize=2)
def guard_path(side):
 from bovine_sculpt import smooth
 controls,radii=arm_curve('poliwrath',side);points,_=smooth(controls,radii);distance=[0.]
 for p,q in zip(points,points[1:]):distance.append(distance[-1]+math.dist(p,q))
 return points,distance,distance[8]
def skin_weights(species,part,position):
 lo,span=authoring_transform(species);x,y,z=(v*span/HEIGHTS[species]+(lo if i==1 else 0.) for i,v in enumerate(position));toad=species=='politoed'
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Toe /')):return rigid(0)
 if part.startswith(('Body /','Belly /','Spiral /')):return rigid(1)
 if part.startswith(('Head /','Cheek /','Mouth /')):return rigid(2)
 if part.startswith(('Eye socket /','Eye /','Pupil /','Brow /')):return rigid((3 if toad else 2)+(not part.endswith('-1')))
 if part.startswith(('Hand /','Thumb /','Finger /','Glove crease /')):
  side=int(part.split()[-2] if part.startswith(('Finger /','Glove crease /')) else part.split()[-1]);return rigid((7 if toad else 6)+(3 if side>0 else 0))
 if part.startswith('Arm /'):
  side=int(part.split()[-1]);base=(5 if toad else 4)+(3 if side>0 else 0)
  points=arm_points(species,side);point=(x,y,z);candidates=[]
  if species=='poliwrath':points,distance,elbow_distance=guard_path(side)
  for segment,(start,end) in enumerate(zip(points,points[1:])):
   delta=tuple(b-a for a,b in zip(start,end));length=sum(v*v for v in delta);t=max(0.,min(1.,sum((p-a)*d for p,a,d in zip(point,start,delta))/length));d2=sum((p-a-t*d)**2 for p,a,d in zip(point,start,delta))
   along=distance[segment]+t*(distance[segment+1]-distance[segment]) if species=='poliwrath' else segment+t
   candidates.append((d2,along))
  t=min(candidates)[1]
  if species=='poliwrath':t=t/elbow_distance if t<=elbow_distance else 1.+(t-elbow_distance)/(distance[-1]-elbow_distance)
  a=smoothstep(.45,1.45,t);b=smoothstep(1.55,1.95,t)
  return [base,base+1,base+2,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith(('Haunch /','Leg /')):
  owner=(11 if toad else 10)+(int(part.split()[-1])>0)
  if part.startswith('Haunch /'):return rigid(owner)
  a=smoothstep(.08,.20,y);return [0,owner,0,0],[1.-a,a,0.,0.]
 if part.startswith('Crest /'):
  a=smoothstep(1.10,1.19,y);return [13,14,0,0],[1.-a,a,0.,0.]
 raise ValueError('unowned spiral frog anatomy: '+part)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid spiral frog clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.3 if clip=='attack' and species=='poliwrath' else 1.8 if clip=='attack' else -1.3
 a*=strength;b*=strength
 angles=[(0.,0.,0.),((.004 if clip=='idle' else .018)*a,.009*b,0.)]
 if species=='politoed':angles += [(.007*b,.012*a,0.)]
 angles += [(.004*b,-.009*a,0.),(.004*b,.009*a,0.)]
 for side in (-1,1):
  if clip=='idle':upper,lower,wrist=.014*b,.020*a,.016*b
  elif clip=='attack':
   upper=-(.09 if species=='politoed' else .15 if species=='poliwrath' else .12)*a
   lower=-(.07 if species=='politoed' else .12 if species=='poliwrath' else .10)*b
   wrist=.018*b
  else:upper,lower,wrist=-.030*a,-.035*b,-.020*b
  yaw=-side*.35*a if clip=='attack' and species=='poliwrath' else side*.010*a
  angles += [(upper,yaw,side*.008*b),(lower,side*.012*b,0.),(wrist,side*.010*a,side*.008*b)]
 angles += [(.008*b,-.008*a,0.),(-.008*b,.008*a,0.)]
 if species=='politoed':angles += [(.012*a,.018*b,0.),(.018*b,.025*a,0.)]
 return angles

def pack_parts(species,model):
 parts=[];groups={}
 for order,p in enumerate(model['primitives']):
  if not p['part'].startswith('Toe /'):
   parts.append((p,{'source_order':order}));continue
  side=p['part'].split()[-2]
  if side not in groups:
   joined={'part':'Toe / complete planted web digits '+side,'positions':[],'normals':[],'indices':[],'base_color':p['base_color']};meta={'source_parts':[]};groups[side]=(joined,meta);parts.append((joined,meta))
  joined,meta=groups[side];start=len(joined['positions'])//3;first=len(joined['indices']);meta['source_parts'].append({'part':p['part'],'order':order,'vertex_start':start,'vertex_count':len(p['positions'])//3,'index_start':first,'index_count':len(p['indices'])});joined['positions']+=p['positions'];joined['normals']+=p['normals'];joined['indices'] += [i+start for i in p['indices']]
 return parts
PACKED_REST_DIGESTS={'poliwhirl': '331c07cd646e2dd21412c2243507420d69862a69a0acaac3086eb3cafbbe4394', 'poliwrath': '3628cbf3a335a904aebdfa48f9ecc48d68033f9449107546b6fe256e7364d09e', 'politoed': '8d04df99bf27b790d08bd554c5cbc4762bcc345acd6cba1fea600867cc5c8c44'}
def export_poli_frog(species,model=None):
 if species not in SPECIES:raise ValueError('unknown spiral frog species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed spiral frog neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite original distinct spiral frog creatures standard skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed spiral frog geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed complete feet and web toes; coherent belly spiral; supported arm/wrist chains; independent eye turrets; distinct toad head and two crest carriers; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/poli_frog_sculpt.py','geometry_preservation':'original distinct rounded blue spiral frogs with padded gloves, severe blue brow and heavy limbs, or green toad with distinct head, cheeks, web digits and curled crest',
    'animation_contract':'identity floor root; complete fixed battle feet and toes; coherent fitted belly and continuous spiral; supported ankles; independent eye turrets and three arm/wrist controls per limb; toad head and two crest controls; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'spiral frog exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('spiral frog exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('spiral frog exceeds existing byte limit')
 return blob


def decode_poli_frog(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_poli_frog(species):raise ValueError('noncanonical spiral frog GLB')
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


def read_poli_frog(path,species):
 with Path(path).open('rb') as f:return decode_poli_frog(f.read(MAX_BYTES+1),species)
def export_poliwhirl(model=None):return export_poli_frog('poliwhirl',model)
def export_poliwrath(model=None):return export_poli_frog('poliwrath',model)
def export_politoed(model=None):return export_poli_frog('politoed',model)
def read_poliwhirl(path):return read_poli_frog(path,'poliwhirl')
def read_poliwrath(path):return read_poli_frog(path,'poliwrath')
def read_politoed(path):return read_poli_frog(path,'politoed')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--species',choices=SPECIES,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_bytes(export_poli_frog(a.species))
if __name__=='__main__':main()
