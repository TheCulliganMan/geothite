#!/usr/bin/env python3
"""Canonical grounded Kabuto and articulated Kabutops standard skins."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from fossil_scythe_sculpt import sculpture,SPECIES,HEIGHTS,small_leg,adult_arm,TAIL
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def influences(pairs):
 pairs=[(j,w) for j,w in pairs if w>1e-12]
 if len(pairs)==1:return rigid(pairs[0][0])
 return [j for j,w in pairs]+[0]*(4-len(pairs)),[w for j,w in pairs]+[0.]*(4-len(pairs))
@lru_cache(maxsize=2)
def authoring_model(s):return sculpture(s)
REST_DIGESTS={'kabuto': 'fcaf686170a83249dfb2edc88f529cbdcd6f8eafb731bcc617dc44f05d6c7be7', 'kabutops': '647542191c7253e934841d51db5bc9b6e60782440f1c39803be0434ee4fafe55'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
@lru_cache(maxsize=2)
def authoring_transform(s):
 raw=sculpture(s,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,HEIGHTS[s]/(hi-lo)
def normalized(s,p):
 lo,k=authoring_transform(s);return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(p))
def joints(s):
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.20 if s=='kabuto' else .53,-.02))]
 if s=='kabuto':
  for side in (-1,1):
   for j in range(2):raw.append((f'leg_{side}_{j}',0,small_leg(side,j)[0]))
 else:
  raw += [('head',1,(0.,.94,.04)),('arm_left',1,adult_arm(-1)[0]),('arm_right',1,adult_arm(1)[0]),('elbow_left',3,adult_arm(-1)[1]),('elbow_right',4,adult_arm(1)[1]),('blade_left',5,adult_arm(-1)[2]),('blade_right',6,adult_arm(1)[2]),('leg_left',0,(-.145,.24,-.035)),('leg_right',0,(.145,.24,-.035)),('tail_base',1,TAIL[0]),('tail_tip',11,(0.,.22,-.45))]
 return tuple((n,p,(0.,0.,0.) if p is None else normalized(s,v)) for n,p,v in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(s,n,point):
 lo,k=authoring_transform(s);y=point[1]/k+lo
 if n.startswith(('Foot /','Claw /','Toe /')):return rigid(0)
 if s=='kabuto':
  if n.startswith('Leg /'):
   side,j=map(int,n.split()[-2:]);a=smoothstep(.04,.13,y);return influences(((0,1.-a),(2+(0 if side<0 else 2)+j,a)))
  return rigid(1)
 if n.startswith(('Body /','Neck /','Rib plate /','Back spur /')):return rigid(1)
 if n.startswith(('Head /','Head wing /','Eye socket /','Eye lens /','Pupil /')):return rigid(2)
 if n.startswith(('Upper arm /','Elbow /')):return rigid(3 if n.endswith('-1') else 4)
 if n.startswith('Forearm /'):return rigid(5 if n.endswith('-1') else 6)
 if n.startswith(('Wrist /','Scythe /')):return rigid(7 if n.endswith('-1') else 8)
 if n.startswith(('Thigh /','Shin /')):
  a=smoothstep(.045,.14,y);return influences(((0,1.-a),(9 if n.endswith('-1') else 10,a)))
 if n.startswith('Tail /'):
  a=smoothstep(.37,.53,-point[2]/k);return influences(((11,1.-a),(12,a)))
 raise ValueError('unowned fossil anatomy '+n)
def pose_angles(s,clip,t):
 if s not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid fossil pose')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[s])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);gain=1.
 else:a=envelope(t,((0.,0.),(.18,-.25),(.34,1.),(.63,.18),(1.,0.)));b=envelope(t,((0.,0.),(.26,-.20),(.46,1.),(.76,-.08),(1.,0.)));gain=2. if clip=='attack' else -1.6
 a*=gain;b*=gain;out=[]
 for name,parent,p in JOINTS[s]:
  side=-1 if name.endswith('left') or '_-1_' in name else 1
  if name=='root':v=(0.,0.,0.)
  elif name=='body':v=(0.,.012*a,0.)
  elif name=='head':v=(.012*a,.025*b,0.)
  elif name.startswith('arm_'):v=(-.065*a,side*.030*b,side*.025*a)
  elif name.startswith('elbow_'):v=(-.08*b,side*.020*a,0.)
  elif name.startswith('blade_'):v=(-.07*a,side*.040*b,side*.025*b)
  elif name.startswith('leg_'):v=(0.,side*.025*b,0.)
  elif name=='tail_base':v=(0.,.025*b,0.)
  elif name=='tail_tip':v=(0.,-.055*a,0.)
  else:raise ValueError(name)
  out.append(v)
 return out

def pack_parts(model):
 result=[];groups={};s=model['name']
 for order,p in enumerate(model['primitives']):
  n=p['part'];prefix=n.split(' /')[0]
  group=prefix if prefix in ('Head wing','Rib plate','Back spur') else 'head ink' if prefix in ('Eye socket','Pupil') else 'gaze' if prefix in ('Eye lens','Iris') else 'toes' if prefix=='Toe' else None
  if group is None:result.append((p,{'source_order':order}));continue
  owner=skin_weights(s,n,(0.,0.,0.))[0][0];side=n.split()[-2] if prefix=='Toe' else '';key=(group,owner,side,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=(prefix+' / complete '+group+' '+str(owner)+' '+side).rstrip()+' '+str(len(groups)),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3;meta['source_parts'].append(dict(part=n,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
 return result

def export_fossil_scythe(species, model=None):
    if species not in SPECIES:raise ValueError('unknown fossil scythe')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed fossil scythe geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated fossil scythe standard skin v1'
    nodes = g.doc['nodes'];inverse = [];js = JOINTS[species]
    for name,parent,pivot in js:
        origin = js[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':species+'/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z = pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
    nodes.append({'name':species,'mesh':0,'skin':0})
    g.doc['scenes'] = [{'name':species,'nodes':[0,len(js)]}]
    g.doc['skins'] = [{'name':species,'skeleton':0,'joints':list(range(len(js))),'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    shared = {};colors = {};primitives = [];parts = pack_parts(model)
    def accessor(values,kind,component=5126,target=None,normalized=False):
        key = (packed(values,component),kind,component,target,normalized)
        if key not in shared:shared[key] = g.accessor(values,kind,component,target,normalized=normalized)
        return shared[key]
    for p,meta in parts:
        color = tuple(p['base_color'])
        if color not in colors:
            colors[color] = len(colors)
            g.doc['materials'].append({'name':species+'/paper_'+str(colors[color]),'alphaMode':'OPAQUE',
                'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.}})
        ids = [];weights = []
        for source in meta.get('source_parts',[dict(part=p['part'],vertex_start=0,vertex_count=len(p['positions'])//3)]):
            for i in range(source['vertex_start']*3,(source['vertex_start']+source['vertex_count'])*3,3):
                point = struct.unpack('<3f',packed(p['positions'][i:i+3]));jj,ww = skin_weights(species,source['part'],point);ww = quantized_weights(ww)
                ids += [j if w else 0 for j,w in zip(jj,ww)];weights += ww
        primitives.append({'attributes':{'POSITION':g.accessor(p['positions'],'VEC3',target=34962,bounds=True),
            'NORMAL':accessor(p['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
            'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},'indices':accessor(p['indices'],'SCALAR',5123,34963),
            'material':colors[color],'mode':4,'extras':{'part':p['part']}})
    g.doc['meshes'] = [{'name':species,'primitives':primitives}]
    for clip,duration,count in CLIPS:
        times = g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels = [];samplers = []
        for joint in range(1,len(js)):
            rotations = [v for i in range(count) for v in quaternion_xyz(*pose_angles(species,clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':species+'.'+clip,'channels':channels,'samplers':samplers,
                                   'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle'}})
    g.doc['extras'] = {'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
        'source_part_ranges':[meta for p,meta in parts],'packed_sha256':geometry_digest({'primitives':[p for p,meta in parts]}),
        'reference_span':HEIGHTS[species],'body_reference_height':HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
        'geometry_source':'tools/fossil_scythe_sculpt.py','animation_contract':'rotation-only source cues; fixed claws/soles; coherent dome and recessed gaze; swept head wings, supported rib plates, articulated scythes and connected tail'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('fossil scythe exceeds existing model limits')
    return blob


def decode_fossil_scythe(blob, species):
    if len(blob)>1024*1024 or blob!=export_fossil_scythe(species):
        raise ValueError('noncanonical fossil scythe GLB')
    length = struct.unpack_from('<I',blob,12)[0]
    doc = json.loads(blob[20:20+length]);binary = blob[28+length:]
    def values(index):
        a = doc['accessors'][index];view = doc['bufferViews'][a['bufferView']]
        width = 3 if a['type']=='VEC3' else 1
        code = 'f' if a['componentType']==5126 else 'H'
        return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
    parts = []
    for p,meta in zip(doc['meshes'][0]['primitives'],doc['extras']['source_part_ranges']):
        positions = values(p['attributes']['POSITION']);normals = values(p['attributes']['NORMAL'])
        indices = values(p['indices']);color = doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
        sources = meta.get('source_parts',[dict(part=p['extras']['part'],order=meta.get('source_order'),
            vertex_start=0,vertex_count=len(positions)//3,index_start=0,index_count=len(indices))])
        for source in sources:
            a = source['vertex_start'];b = a+source['vertex_count']
            i = source['index_start'];j = i+source['index_count']
            parts.append((source['order'],dict(part=source['part'],positions=positions[3*a:3*b],
                normals=normals[3*a:3*b],indices=[k-a for k in indices[i:j]],base_color=color)))
    return dict(name=species,version=1,coordinate_system=COORDINATES,
                primitives=[p for order,p in sorted(parts,key=lambda item:item[0])])



@lru_cache(maxsize=2)
def canonical_blob(species):return export_fossil_scythe(species,authoring_model(species))
def read_fossil_scythe(path,species):
 with Path(path).open('rb') as f:return decode_fossil_scythe(f.read(1024*1024+1),species)
def read_kabuto(path):return read_fossil_scythe(path,'kabuto')
def read_kabutops(path):return read_fossil_scythe(path,'kabutops')
