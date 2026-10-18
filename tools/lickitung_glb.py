#!/usr/bin/env python3
"""Original tongue bearer with a physical oral recess and independent tongue/tail chains."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
import lickitung_sculpt as source
from bovine_sculpt import smooth
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('lickitung',);HEIGHTS={'lickitung':source.HEIGHT}
def sculpture(species='lickitung',normalize=True):
 if species!='lickitung':raise ValueError('unknown tongue bearer')
 return source.sculpture(normalize)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=1)
def authoring_model(species='lickitung'):return sculpture(species)
REST_DIGESTS={'lickitung': '3662e8ec4d4b4c20173929a640da889ff077b0f8e57d81d6696df70736825a88'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=1)
def authoring_transform(species='lickitung'):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,hi-lo
def normalized(species,point):
 lo,span=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))
def arms(side):return source.arm_points(side)[:3]
TONGUE_POINTS=source.TONGUE_POINTS
def joints(species='lickitung'):
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.48,0.)),('neck',1,(0.,.91,.06)),('head',2,(0.,1.15,.08))]
 for name,s in (('left',-1),('right',1)):
  points=arms(s);base=len(raw);raw += [('arm_'+name,1,points[0]),('elbow_'+name,base,points[1]),('wrist_'+name,base+1,points[-1])]
 raw += [('leg_left',0,(-.256,.28,.02)),('leg_right',0,(.256,.28,.02)),('tail_base',1,(0,.327,-.206)),('tail_mid',12,(.028,.502,-.480)),('tail_tip',13,(.025,.80,-.615)),('tongue_base',3,TONGUE_POINTS[0]),('tongue_mid',15,TONGUE_POINTS[1]),('tongue_tip',16,TONGUE_POINTS[3])]
 return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for n,parent,p in raw)
JOINTS={'lickitung':joints()};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
@lru_cache(maxsize=1)
def tongue_segments():
 path,_=smooth(TONGUE_POINTS,[1.]*len(TONGUE_POINTS),12);lengths=[0.]
 for a,b in zip(path,path[1:]):lengths.append(lengths[-1]+math.dist(a,b))
 return [(a,b,lo/lengths[-1],hi/lengths[-1]) for a,b,lo,hi in zip(path,path[1:],lengths,lengths[1:])]
def along_tongue(point):
 best=float('inf');parameter=0.
 for a,b,lo,hi in tongue_segments():
  d=tuple(b[i]-a[i] for i in range(3));t=max(0.,min(1.,sum((point[i]-a[i])*d[i] for i in range(3))/sum(v*v for v in d)));distance=sum((point[i]-a[i]-t*d[i])**2 for i in range(3))
  if distance<best:best=distance;parameter=lo+t*(hi-lo)
 return parameter
@lru_cache(maxsize=1)
def tongue_weight_surface():
 # Interpolate the actual stored, quantized parent weights. A fitted marking
 # cannot project its own offset vertices back onto a curved centerline: that
 # gives it a different deformation from the triangles directly underneath.
 parent=next(p for p in authoring_model()['primitives'] if p['part'].startswith('Tongue /'))
 values=struct.unpack('<'+'f'*len(parent['positions']),packed(parent['positions']));points=[values[i:i+3] for i in range(0,len(values),3)];weights=[quantized_weights(skin_weights('lickitung',parent['part'],point)[1]) for point in points];triangles=[]
 for i in range(0,len(parent['indices']),3):
  ids=parent['indices'][i:i+3];a,b,c=[points[j] for j in ids];den=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
  if den>1e-12:triangles.append((ids,a,b,c,den))
 def at(point):
  x,y,z=point;best=None
  for ids,a,b,c,den in triangles:
   u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/den;v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/den;ws=(u,v,1-u-v)
   if min(ws)<-1e-6:continue
   height=sum(w*p[2] for w,p in zip(ws,(a,b,c)))
   if best is None or height>best[0]:best=(height,ids,ws)
  if best is None:raise ValueError('tongue groove lies outside stored tongue')
  _,ids,ws=best;ws=[max(0.,w) for w in ws];total=sum(ws);return [3,15,16,17],[sum(ws[k]*weights[ids[k]][j]/255. for k in range(3))/total for j in range(4)]
 return at

def body_weights(y):
 a=smoothstep(.75,1.03,y);b=smoothstep(1.04,1.17,y)
 return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
def chain_weights(owners,a,b,c):return list(owners),[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
def skin_weights(species,part,position):
 if part.startswith('Tongue groove /'):return tongue_weight_surface()(position)
 lo,span=authoring_transform(species);point=tuple(v*span/HEIGHTS[species]+(lo if i==1 else 0.) for i,v in enumerate(position));x,y,z=point
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Toe /')):return rigid(0)
 if part.startswith(('Head /','Mouth lining /','Eye /','Eye glint /','Nostril /')):return rigid(3)
 if part.startswith('Hand tip /'):return rigid(6 if part.endswith('-1') else 9)
 if part.startswith(('Shoulder /','Arm /')):
  side=-1 if part.endswith('-1') else 1;points=arms(side);axis=[b-a for a,b in zip(points[0],points[-1])];t=sum((point[i]-points[0][i])*axis[i] for i in range(3))/sum(v*v for v in axis)
  base=4 if side<0 else 7;return chain_weights((1,base,base+1,base+2),smoothstep(-.10,.18,t),smoothstep(.28,.62,t),smoothstep(.68,.90,t))
 if part.startswith(('Thigh /','Thigh ring /','Thigh center /')):
  a=smoothstep(.08,.40,y);b=.25*math.sin(math.pi*a)**2
  return [0,10 if part.endswith('-1') else 11,1,0],[(1.-a)*(1.-b),b,a*(1.-b),0.]
 if part.startswith(('Tongue /','Tongue groove /')):
  t=along_tongue(point);return chain_weights((3,15,16,17),smoothstep(.03,.20,t),smoothstep(.30,.58,t),smoothstep(.68,.90,t))
 if part.startswith('Tail /'):return chain_weights((1,12,13,14),smoothstep(.30,.53,y),smoothstep(.55,.83,y),smoothstep(.90,1.05,y))
 return body_weights(y)
def pose_angles(species,clip,t):
 if species!='lickitung' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid tongue-bearer clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*18
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.012*a,0.),(.006*b,.009*a,0.),(.008*b,.015*a,0.)]
 for s in (-1,1):out += [(.018*b,s*.014*a,-s*.018*b),(.022*a,s*.012*b,s*.010*a),(.012*b,s*.018*a,s*.012*b)]
 out += [(0.,-.020*a,0.),(0.,.020*a,0.),(.014*b,.020*a,0.),(.018*a,.026*b,0.),(.020*b,.034*a,0.),(.018*b,.018*a,0.),(.025*a,.022*b,0.),(.035*b,.030*a,0.)]
 return out

def pack_parts(species,model):
 parts=[];groups={}
 for order,part in enumerate(model['primitives']):
  name=part['part'];kind=name.split(' /')[0];key=(kind,) if kind in ('Belly band','Shoulder','Eye','Eye glint','Nostril','Thigh ring','Thigh center') else None
  # Walking IK consumes complete runtime parts; bilateral supports stay separate.
  if kind in ('Thigh ring','Thigh center'):key=None
  if key is None:parts.append((part,{'source_order':order}));continue
  if key not in groups:
   joined={'part':kind+' / complete tongue-bearer assembly','positions':[],'normals':[],'indices':[],'base_color':part['base_color']};meta={'source_parts':[]};groups[key]=(joined,meta);parts.append((joined,meta))
  joined,meta=groups[key];assert joined['base_color']==part['base_color'];start=len(joined['positions'])//3;first=len(joined['indices']);meta['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(part['positions'])//3,'index_start':first,'index_count':len(part['indices'])});joined['positions']+=part['positions'];joined['normals']+=part['normals'];joined['indices'] += [i+start for i in part['indices']]
 return parts
PACKED_REST_DIGESTS={'lickitung': '972f51bfd45847d1dcda324c713ac66537d2eb19a6a9407ae395279d9bc99156'}
def export_tongue_bearer(species,model=None):
 if species not in SPECIES:raise ValueError('unknown tongue_bearer species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed tongue_bearer neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite original tongue bearer with continuous palms and articulated tongue and tail skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed tongue_bearer geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed complete feet and toes; coherent recessed mouth and gaze; articulated arms, wrists, broad tongue and heavy tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/'+species+'_sculpt.py','geometry_preservation':'original stocky pink tongue bearer; physical oral recess, broad tongue and fitted groove, heavy upward tail, continuous palms and cream haunch rings; adaptive fitted surfaces and focused round sampling',
    'animation_contract':'identity floor root; fixed feet and toes; coherent face, gaze and recessed mouth; independent shoulder/elbow/wrist chains with supported legs; three tongue and three tail carriers preserve the actual broad surfaces and fitted markings; lossless source-solid packing; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'tongue_bearer exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('tongue_bearer exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('tongue_bearer exceeds existing byte limit')
 return blob


def decode_tongue_bearer(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_tongue_bearer(species):raise ValueError('noncanonical tongue_bearer GLB')
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


def read_tongue_bearer(path,species):
 with Path(path).open('rb') as f:return decode_tongue_bearer(f.read(MAX_BYTES+1),species)
def export_lickitung(model=None):return export_tongue_bearer('lickitung',model)
def decode_lickitung(blob):return decode_tongue_bearer(blob,'lickitung')
def read_lickitung(path):return read_tongue_bearer(path,'lickitung')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_bytes(export_lickitung())
if __name__=='__main__':main()
