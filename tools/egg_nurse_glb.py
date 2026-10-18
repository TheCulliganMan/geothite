#!/usr/bin/env python3
"""Original egg-nurse pair with contained eggs, open pouches and rooted flexible appendages."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
import chansey_sculpt,blissey_sculpt
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SOURCES={'chansey':chansey_sculpt,'blissey':blissey_sculpt};SPECIES=tuple(SOURCES);HEIGHTS={s:m.HEIGHT for s,m in SOURCES.items()}
def sculpture(species,normalize=True):
 if species not in SOURCES:raise ValueError('unknown egg nurse')
 return SOURCES[species].sculpture(normalize)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'chansey': 'a5b1ee98864b13d123d3f59ff5c7f2b81b530d746157a0fc8bd397a14f387a27', 'blissey': 'd48c3eeb58816ec7caa4200536f9e065544a3306a8f48f69c04e29f114ef8a80'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,hi-lo
def normalized(species,point):
 lo,span=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))
def joints(species):
 source=SOURCES[species];baby=species=='chansey'
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.50,0.)),('head',1,(0.,.88 if baby else .96,0.))]
 for name,s in (('left',-1),('right',1)):
  points=source.arm_points(s);base=len(raw);raw += [('arm_'+name,1,points[0]),('elbow_'+name,base,points[1])]
 raw += [('leg_left',0,(-.239 if baby else -.174,.163 if baby else .153,.023 if baby else .015)),('leg_right',0,(.239 if baby else .174,.163 if baby else .153,.023 if baby else .015))]
 if baby:
  for s,name in ((-1,'left'),(1,'right')):raw += [(f'ribbon_{name}_{i}',2,source.ribbon_root(s,i)) for i in range(3)]
  raw += [('tail_base',1,source.TAIL_POINTS[0]),('tail_tip',15,source.TAIL_POINTS[1])]
 else:
  raw += [('crown_left',2,(-.220,1.105,-.017)),('curl_left',9,source.curl_points(-1)[0]),('crown_right',2,(.220,1.105,-.017)),('curl_right',11,source.curl_points(1)[0]),('frill_left',3,(-.346,.760,.006)),('frill_right',5,(.346,.760,.006))]
  raw += [(f'skirt_{i}',1,source.skirt_root(i)) for i in range(10)]
 return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for n,parent,p in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def body_weights(species,y):
 a=smoothstep(.70,.82,y) if species=='chansey' else smoothstep(.78,.94,y)
 return [1,2,0,0],[1.-a,a,0.,0.]
def parameter(point,start,end):
 axis=[b-a for a,b in zip(start,end)];return sum((point[i]-start[i])*axis[i] for i in range(3))/sum(v*v for v in axis)
def skin_weights(species,part,position):
 lo,span=authoring_transform(species);point=tuple(v*span/HEIGHTS[species]+(lo if i==1 else 0.) for i,v in enumerate(position));x,y,z=point;baby=species=='chansey'
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Foot /'):return rigid(0)
 if part.startswith(('Pouch /','Egg /','Lower body /')):return rigid(1)
 if part.startswith(('Eye /','Eye glint /','Smile /','Chin /','Mouth lining /','Tongue /')):return rigid(2)
 side=-1 if part.endswith('-1') else 1;base=3 if side<0 else 5
 if part.startswith('Shoulder /'):return rigid(base)
 if part.startswith('Hand /'):return rigid(1 if baby else base+1)
 if part.startswith('Arm /'):
  points=SOURCES[species].arm_points(side);t=parameter(point,points[0],points[-1]);a=smoothstep(.10,.50,t) if baby else smoothstep(.15,.85,t)
  if not baby:return [base,base+1,0,0],[1.-a,a,0.,0.]
  b=smoothstep(.62,.88,t);return [base,base+1,1,0],[(1.-a)*(1.-b),a*(1.-b),b,0.]
 if part.startswith(('Ankle /','Leg /')):
  a=smoothstep(.07,.27,y);b=.30*math.sin(math.pi*a)**2
  return [0,7 if side<0 else 8,1,0],[(1.-a)*(1.-b),b,a*(1.-b),0.]
 if part.startswith('Ribbon /'):
  s,i=map(int,part.split()[-2:]);start=chansey_sculpt.ribbon_root(s,i);t=(s*x-abs(start[0]))/((.60,.645,.635)[i]-abs(start[0]));a=smoothstep(.03,.30,t);return [2,(9 if s<0 else 12)+i,0,0],[1.-a,a,0.,0.]
 if part.startswith('Tail /'):
  t=parameter(point,chansey_sculpt.TAIL_POINTS[0],chansey_sculpt.TAIL_POINTS[-1]);a=smoothstep(.05,.28,t);b=smoothstep(.48,.80,t);return [1,15,16,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith('Crown /'):return rigid(9 if side<0 else 11)
 if part.startswith('Curl /'):return rigid(10 if side<0 else 12)
 if part.startswith('Arm frill /'):return rigid(13 if part.split()[-2]=='-1' else 14)
 if part.startswith('Skirt /'):return rigid(15+int(part.split()[-1]))
 return body_weights(species,y)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid egg-nurse clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength;baby=species=='chansey'
 out=[(0.,0.,0.),(0.,.012*a,0.),(.008*b,.012*a,0.)]
 for s in (-1,1):out += [(.010*b,s*.010*a,-s*.010*b),(.012*a,s*.010*b,s*.010*a)] if baby else [(.018*b,s*.016*a,-s*.014*b),(.022*a,s*.012*b,s*.012*a)]
 out += [(0.,-.020*a,0.),(0.,.020*a,0.)]
 if baby:
  for s in (-1,1):
   for i in range(3):out.append((.010*b,s*(.012+.003*i)*a,s*(.014+.003*i)*b))
  out += [(.015*b,.018*a,0.),(.020*a,.025*b,0.)]
 else:
  out += [(0.,-.010*a,-.008*b),(.012*b,-.014*a,-.012*b),(0.,.010*a,.008*b),(.012*b,.014*a,.012*b),(.012*b,-.014*a,-.016*b),(.012*b,.014*a,.016*b)]
  for i in range(10):
   theta=(i+.5)*math.tau/10;flutter=(.014*a+.006*b*math.cos(theta));out.append((flutter*math.cos(theta),.010*b, -flutter*math.sin(theta)))
 return out

def pack_parts(species,model):
 parts=[];groups={}
 for order,part in enumerate(model['primitives']):
  name=part['part'];kind=name.split(' /')[0];key=(kind,) if kind in ('Ribbon','Eye','Eye glint','Crown','Curl','Shoulder','Arm frill','Skirt') else None
  if key is None:parts.append((part,{'source_order':order}));continue
  if key not in groups:
   joined={'part':kind+' / complete egg-nurse assembly','positions':[],'normals':[],'indices':[],'base_color':part['base_color']};meta={'source_parts':[]};groups[key]=(joined,meta);parts.append((joined,meta))
  joined,meta=groups[key];assert joined['base_color']==part['base_color'];start=len(joined['positions'])//3;first=len(joined['indices']);meta['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(part['positions'])//3,'index_start':first,'index_count':len(part['indices'])});joined['positions']+=part['positions'];joined['normals']+=part['normals'];joined['indices'] += [i+start for i in part['indices']]
 return parts
PACKED_REST_DIGESTS={'chansey': '9b0d87c52e08fad76e7ee66f2983e075382f8a3da37b989e4cc025be0061021e', 'blissey': '316186bb6e6a3a5c2eac8d4040d19e52536c3d642bdb23d6dba0288056f62a0e'}
def export_egg_nurse(species,model=None):
 if species not in SPECIES:raise ValueError('unknown egg-nurse species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed egg-nurse neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite original egg nurses with supported open pouches, gathered hands and rooted ribbons, curls and frills skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed egg-nurse geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed feet; coherent egg and open pouch; supported gathered hands or independent arm tips; articulated ribbons, curls and frills; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/'+species+'_sculpt.py','geometry_preservation':'original soft pink Chansey and pink/white Blissey; contained eggs in actual open pink cups, gathered hands, rounded side ribbons or curled locks and rooted white frills; adaptive fitted surfaces and focused round sampling',
    'animation_contract':'identity floor root; fixed complete feet; coherent face and mouth; egg and pouch share the body carrier; Chansey hands keep their grip while arms flex; supported legs and independent ribbons, tail, crown locks, shoulder frills and skirt petals; lossless source-solid packing; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'egg-nurse exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('egg-nurse exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('egg-nurse exceeds existing byte limit')
 return blob


def decode_egg_nurse(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_egg_nurse(species):raise ValueError('noncanonical egg-nurse GLB')
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


def read_egg_nurse(path,species):
 with Path(path).open('rb') as f:return decode_egg_nurse(f.read(MAX_BYTES+1),species)
def export_chansey(model=None):return export_egg_nurse('chansey',model)
def export_blissey(model=None):return export_egg_nurse('blissey',model)
def decode_chansey(blob):return decode_egg_nurse(blob,'chansey')
def decode_blissey(blob):return decode_egg_nurse(blob,'blissey')
def read_chansey(path):return read_egg_nurse(path,'chansey')
def read_blissey(path):return read_egg_nurse(path,'blissey')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--species',choices=SPECIES,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_bytes(export_egg_nurse(a.species))
if __name__=='__main__':main()
