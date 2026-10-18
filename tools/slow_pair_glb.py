#!/usr/bin/env python3
"""Canonical shell-biped skins with coherent mouths, held shells and royal crowns."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
import slowbro_sculpt,slowking_sculpt
from bovine_sculpt import smooth
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('slowbro','slowking');HEIGHTS={'slowbro':slowbro_sculpt.HEIGHT,'slowking':slowking_sculpt.HEIGHT}
def sculpture(species,normalize=True):
 if species not in SPECIES:raise ValueError('unknown shell biped')
 return (slowbro_sculpt if species=='slowbro' else slowking_sculpt).sculpture(normalize)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'slowbro': 'd530ae799750a2e83b3474fc00986df88a53935c12baaf152c3916c5056905b2', 'slowking': '207d131b852e541371aee7b2b863fc9142487a0e5b60a64a86c591a8d34d81d2'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,hi-lo
def normalized(species,point):
 lo,span=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))
def arms(species,side):
 return [(side*.238,.854,.085),(side*.300,.718,.15),(side*.281,.617,.245)] if species=='slowbro' else [(side*.180,.859,.044),(side*.215,.745,.106),(side*.204,.653,.193)]
def tail(species):
 return [(0,.30,-.163),(.17,.245,-.340),(.30,.355,-.440),(.37,.436,-.510)] if species=='slowbro' else [(0,.282,-.158),(.13,.245,-.365),(.285,.29,-.595),(.399,.402,-.784),(.447,.526,-.878)]
def joints(species):
 bro=species=='slowbro';raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.55,0.)),('neck',1,(0.,.94 if bro else 1.0,.02)),('head',2,(0.,1.18 if bro else 1.18,.04))]
 for name,s in (('left',-1),('right',1)):
  points=arms(species,s);base=len(raw);raw += [('arm_'+name,1,points[0]),('elbow_'+name,base,points[1]),('wrist_'+name,base+1,points[-1])]
 raw += [('leg_left',0,(-.291,.25,.04) if bro else (-.236,.235,.038)),('leg_right',0,(.291,.25,.04) if bro else (.236,.235,.038)),('tail_base',1,tail(species)[0]),('tail_tip',12,tail(species)[2])]
 if bro:raw += [('shell',13,(.37,.436,-.510)),('ear_left',3,(-.209,1.444,.022)),('ear_right',3,(.209,1.444,.022)),('shell_jaw',14,(.37,.34,-.60))]
 else:raw += [('crown',3,(0.,1.42,-.024)),('ruff_left',2,(-.08,1.048,.02)),('ruff_right',2,(.08,1.048,.02)),('ruff_center',2,(0.,1.048,.02))]
 return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for n,parent,p in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
@lru_cache(maxsize=6)
def path_segments(species,key):
 points=tail(species) if key=='tail' else arms(species,-1 if key=='left' else 1);path,_=smooth(points,[1.]*len(points),12);lengths=[0.]
 for a,b in zip(path,path[1:]):lengths.append(lengths[-1]+math.dist(a,b))
 return [(a,b,lo/lengths[-1],hi/lengths[-1]) for a,b,lo,hi in zip(path,path[1:],lengths,lengths[1:])]
def along_path(species,point,key):
 best=float('inf');parameter=0.
 for a,b,lo,hi in path_segments(species,key):
  d=tuple(b[i]-a[i] for i in range(3));t=max(0.,min(1.,sum((point[i]-a[i])*d[i] for i in range(3))/sum(v*v for v in d)));distance=sum((point[i]-a[i]-t*d[i])**2 for i in range(3))
  if distance<best:best=distance;parameter=lo+t*(hi-lo)
 return parameter
def body_weights(species,y):
 a=smoothstep(.70 if species=='slowbro' else .75,.94 if species=='slowbro' else .99,y);b=smoothstep(1.04 if species=='slowbro' else 1.05,1.18 if species=='slowbro' else 1.14,y)
 return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
def skin_weights(species,part,position):
 lo,span=authoring_transform(species);point=tuple(v*span/HEIGHTS[species]+(lo if i==1 else 0.) for i,v in enumerate(position));x,y,z=point
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Foot claw /')):return rigid(0)
 if part.startswith(('Shell jaw /','Shell lower tooth /')):return rigid(17)
 if part.startswith(('Shell ','Crown ','Jewel')):return rigid(14)
 if part.startswith(('Ear /','Ear inset /','Ear curl /')):return rigid(15 if part.endswith('-1') else 16)
 if part.startswith('Collar panel /'):
  angle=math.tau*int(part.split()[-1])/10;side=math.sin(angle);return rigid(15 if side<-.3 else 16 if side>.3 else 17)
 if part.startswith(('Head /','Muzzle /','Jaw /','Mouth back /','Mouth seam /','Tongue /','Eye /','Pupil /','Nostril /','Fang /','Tooth /')):return rigid(3)
 if part.startswith(('Shoulder /','Arm /','Hand /','Hand claw /')):
  side=part.split()[-2] if part.startswith('Hand claw /') else part.split()[-1];base=4 if side=='-1' else 7
  if part.startswith(('Hand /','Hand claw /')):return rigid(base+2)
  # A continuous longitudinal coordinate avoids closest-path branch changes
  # across the broad inside of an elbow. Both shoulder and arm use this field.
  points=arms(species,-1 if side=='-1' else 1);axis=[b-a for a,b in zip(points[0],points[-1])]
  t=sum((point[i]-points[0][i])*axis[i] for i in range(3))/sum(v*v for v in axis)
  upper=smoothstep(-.10,.18,t);elbow=smoothstep(.28,.62,t);wrist=smoothstep(.68,.90,t)
  return [1,base,base+1,base+2],[1.-upper,upper*(1.-elbow),upper*elbow*(1.-wrist),upper*elbow*wrist]
 if part.startswith(('Thigh /','Leg /')):
  a=smoothstep(.07,.42,y);b=.25*math.sin(math.pi*a)**2
  return [0,10 if part.endswith('-1') else 11,1,0],[(1.-a)*(1.-b),b,a*(1.-b),0.]
 if part.startswith('Tail /'):
  t=along_path(species,point,'tail');a=smoothstep(.05,.35,t);b=smoothstep(.45,.85,t)
  return [1,12,13,0],[1.-a,a*(1.-b),a*b,0.]
 return body_weights(species,y)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid shell-biped clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*18
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.012*a,0.),(0.,.010*a,0.),(.008*b,.016*a,0.)]
 for s in (-1,1):out += [(.015*b,s*.015*a,-s*.018*b),(.025*a,s*.014*b,s*.012*a),(.014*b,s*.020*a,s*.012*b)]
 out += [(0.,-.022*a,0.),(0.,.022*a,0.),(0.,.028*a,0.),(.018*b,.036*b,0.)]
 out += [(-.008*b,.010*a,0.),(.014*b,0.,-.020*a),(.014*b,0.,.020*a),(.012*b,0.,0.)] if species=='slowbro' else [(.008*b,.012*a,0.),(0.,0.,-.012*b),(0.,0.,.012*b),(.010*b,0.,0.)]
 return out

def pack_parts(species,model):
 parts=[];groups={}
 for order,part in enumerate(model['primitives']):
  name=part['part'];kind=name.split(' /')[0];words=name.split();key=None
  if kind in ('Hand claw','Foot claw'):key=(kind,words[-2])
  elif kind=='Collar panel':key=(kind,tuple(part['base_color']))
  elif kind in ('Belly band','Shoulder','Eye','Pupil','Ear','Ear inset','Ear curl','Nostril','Fang','Tooth','Shell coil','Shell spine','Shell upper tooth','Shell lower tooth','Shell socket','Shell eye','Shell pupil','Crown turn','Crown horn','Crown crest'):key=(kind,)
  if key is None:parts.append((part,{'source_order':order}));continue
  if key not in groups:
   label=kind+' / complete shell-biped assembly'+(' '+words[-2] if kind in ('Hand claw','Foot claw') else ' '+str(len(groups)) if kind=='Collar panel' else '')
   joined={'part':label,'positions':[],'normals':[],'indices':[],'base_color':part['base_color']};meta={'source_parts':[]};groups[key]=(joined,meta);parts.append((joined,meta))
  joined,meta=groups[key];assert joined['base_color']==part['base_color'];start=len(joined['positions'])//3;first=len(joined['indices']);meta['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(part['positions'])//3,'index_start':first,'index_count':len(part['indices'])});joined['positions']+=part['positions'];joined['normals']+=part['normals'];joined['indices'] += [i+start for i in part['indices']]
 return parts
PACKED_REST_DIGESTS={'slowbro': '33741cd0059c20dfaee09820fe9898e00eee22e21fce8e153eda4985ed38fc14', 'slowking': 'c66499f5fae82d1d849da07acc4b7d5e48ce75270cac872bdee5681bb1f5320b'}
def export_shell_biped(species,model=None):
 if species not in SPECIES:raise ValueError('unknown shell_biped species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed shell_biped neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite shell-tailed and royal bipeds with coherent held assemblies skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed shell_biped geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed complete feet and claws; coherent facial layers and open mouth; articulated arms, wrists, tail and held shell/jaw or crown/collar; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/'+species+'_sculpt.py','geometry_preservation':'original pink shell-tailed biped with separate toothed spiral creature and royal biped with horned crown, ruby and flared collar; adaptive fitted surfaces and focused round sampling',
    'animation_contract':'identity floor root; fixed feet and claws; coherent face, fitted gaze and mouth; independent shoulder/elbow/wrist chains and supported legs; shell follows actual tail grip with a distinct jaw, or crown remains complete over independent collar sectors; lossless packing preserves every source solid; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'shell_biped exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('shell_biped exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('shell_biped exceeds existing byte limit')
 return blob


def decode_shell_biped(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_shell_biped(species):raise ValueError('noncanonical shell_biped GLB')
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


def read_shell_biped(path,species):
 with Path(path).open('rb') as f:return decode_shell_biped(f.read(MAX_BYTES+1),species)
def export_slowbro(model=None):return export_shell_biped('slowbro',model)
def read_slowbro(path):return read_shell_biped(path,'slowbro')
def export_slowking(model=None):return export_shell_biped('slowking',model)
def read_slowking(path):return read_shell_biped(path,'slowking')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_bytes(export_shell_biped(a.species))
if __name__=='__main__':main()
