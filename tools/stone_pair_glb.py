#!/usr/bin/env python3
"""Articulated Graveler and Golem with coherent rock faces, digits and grounded supports."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from graveler_sculpt import sculpture as graveler,HEIGHT as GRAVELER_HEIGHT
from golem_sculpt import sculpture as golem,HEIGHT as GOLEM_HEIGHT
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep
SPECIES=('graveler','golem')
HEIGHTS={'graveler':GRAVELER_HEIGHT,'golem':GOLEM_HEIGHT}

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):
 if species not in SPECIES:raise ValueError('unknown stone pair')
 return (graveler if species=='graveler' else golem)()

REST_DIGESTS={'graveler': '4734b3c5b5cc465ed932c49ea3e186c7ed55e64ac8d80ccc36ab5a486ea93ea6', 'golem': '553f96c63633520a526ec732e47bce30fd56e62774bae51893be60f6134800de'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=(graveler if species=='graveler' else golem)(False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,HEIGHTS[species]/(hi-lo)

def normalized(species,point):
 lo,scale=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*scale for i,v in enumerate(point))

def joints(species):
 if species=='graveler':
  raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.50,0.)),
       ('arm_upper_left',1,(-.292,.70,0.)),('arm_upper_right',1,(.292,.70,0.)),
       ('arm_lower_left',1,(-.28,.34,.05)),('arm_lower_right',1,(.28,.34,.05)),
       ('wrist_upper_left',2,(-.605,.623,.115)),('wrist_upper_right',3,(.605,.623,.115)),
       ('wrist_lower_left',4,(-.224,.246,.267)),('wrist_lower_right',5,(.224,.246,.267))]
 else:
  raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.66,-.03)),('head',1,(0.,.884,.25)),('jaw',1,(0.,.719,.30)),
       ('arm_left',1,(-.344,.854,.145)),('arm_right',1,(.344,.854,.145)),
       ('forearm_left',4,(-.481,.738,.246)),('forearm_right',5,(.481,.738,.246)),
       ('wrist_left',6,(-.511,.864,.351)),('wrist_right',7,(.511,.864,.351)),
       ('leg_left',0,(-.285,.272,.051)),('leg_right',0,(.285,.272,.051))]
 return tuple((n,p,(0.,0.,0.) if p is None else normalized(species,v)) for n,p,v in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))

def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def skin_weights(species,title,point):
 if title.startswith(('Foot /','Foot claw /','Heel claw /')):return rigid(0)
 if species=='graveler':
  if title.startswith(('Arm /','Forearm /')):return rigid(2+(0 if ' upper ' in title else 2)+(0 if title.endswith('-1') else 1))
  if title.startswith(('Hand /','Finger /')):
   side=int(title.split()[-3] if title.startswith('Finger /') else title.split()[-1]);return rigid(6+(0 if ' upper ' in title else 2)+(0 if side<0 else 1))
  if title.startswith(('Body /','Forehead /','Socket /','Eye /','Pupil /','Mouth /','Mouth inset /','Rubble /')):return rigid(1)
 else:
  if title.startswith(('Shell /','Shell plate /')):return rigid(1)
  if title.startswith(('Head /','Muzzle /','Mouth back /','Socket /','Eye /','Iris /','Pupil /','Fang /')):return rigid(2)
  if title.startswith(('Jaw /','Tongue /','Lower tooth /')):return rigid(3)
  if title.startswith('Arm /'):return rigid(4+(0 if title.endswith('-1') else 1))
  if title.startswith('Forearm /'):return rigid(6+(0 if title.endswith('-1') else 1))
  if title.startswith(('Hand /','Hand claw /')):
   side=int(title.split()[-2] if title.startswith('Hand claw /') else title.split()[-1]);return rigid(8+(0 if side<0 else 1))
  if title.startswith(('Thigh /','Leg /')):
   lo,scale=authoring_transform(species);y=point[1]/scale+lo;amount=smoothstep(.10,.27,y);j=10+(0 if title.endswith('-1') else 1)
   if amount==0:return rigid(0)
   if amount==1:return rigid(j)
   return [0,j,0,0],[1.-amount,amount,0.,0.]
 raise ValueError('unowned stone anatomy: '+title)

def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid stone clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,-.25),(.34,1.),(.63,.18),(1.,0.)));b=envelope(t,((0.,0.),(.26,-.20),(.46,1.),(.76,-.08),(1.,0.)));strength=2.0 if clip=='attack' else -1.7
 a*=strength;b*=strength;out=[]
 for name,parent,pivot in JOINTS[species]:
  side=-1 if name.endswith('left') else 1
  if name=='root':angle=(0.,0.,0.)
  elif name=='body':angle=(0.,.014*a,0.)
  elif name=='head':angle=(.012*a,.025*b,0.)
  elif name=='jaw':angle=(.020*b if clip=='idle' else .055*abs(b),0.,0.)
  elif name.startswith('arm_upper'):angle=(-.080*a,side*.045*b,side*.025*a)
  elif name.startswith('arm_lower'):angle=(.045*b,side*.035*a,-side*.035*b)
  elif name.startswith('arm_'):angle=(-.060*a,side*.035*b,side*.025*a)
  elif name.startswith('forearm_'):angle=(-.055*b,side*.025*a,side*.020*b)
  elif name.startswith('wrist_'):angle=(-.065*b,side*.030*a,side*.020*a) if species=='graveler' else (-.045*a,side*.025*b,side*.020*a)
  else:angle=(0.,side*.020*b,0.)
  out.append(angle)
 return out

def pack_parts(model):
 result=[];groups={};species=model['name']
 for order,p in enumerate(model['primitives']):
  title=p['part'];prefix=title.split(' /')[0]
  group=('body ink' if prefix in ('Socket','Pupil','Mouth') else 'body ivory' if prefix=='Eye' else prefix if prefix in ('Rubble','Finger') else None) if species=='graveler' else ('head ink' if prefix in ('Socket','Pupil','Mouth back') else 'head ivory' if prefix in ('Eye','Fang') else prefix if prefix in ('Shell plate','Iris','Hand claw','Foot claw','Lower tooth') else None)
  if group is None:result.append((p,{'source_order':order}));continue
  owner=skin_weights(species,title,(0.,0.,0.))[0][0];side=title.split()[-2] if prefix=='Foot claw' else ''
  key=(group,owner,side,tuple(p['base_color']))
  if key not in groups:
   label=(prefix+' / complete '+group+' '+str(owner)+' '+side).rstrip()+' '+str(len(groups))
   q=dict(part=label,positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3
  meta['source_parts'].append(dict(part=title,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
 return result

def export_stone_pair(species, model=None):
    if species not in SPECIES:raise ValueError('unknown stone pair')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed stone pair geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated stone pair standard skin v1'
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
        'geometry_source':'tools/graveler_sculpt.py and tools/golem_sculpt.py','animation_contract':'rotation-only source cues; grounded fixed supports; coherent fitted gaze, rock field, polygon plates and digits; four independent Graveler arms and wrists; Golem skull, jaw, shoulders, elbows, wrists and hind limbs'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('stone pair exceeds existing model limits')
    return blob


def decode_stone_pair(blob, species):
    if len(blob)>1024*1024 or blob!=export_stone_pair(species):
        raise ValueError('noncanonical stone pair GLB')
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
def canonical_blob(species):return export_stone_pair(species,authoring_model(species))
def export_graveler(model=None):return export_stone_pair('graveler',model)
def export_golem(model=None):return export_stone_pair('golem',model)
def read_stone_pair(path,species):
 with Path(path).open('rb') as f:return decode_stone_pair(f.read(1024*1024+1),species)
def read_graveler(path):return read_stone_pair(path,'graveler')
def read_golem(path):return read_stone_pair(path,'golem')
