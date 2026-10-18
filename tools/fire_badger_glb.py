#!/usr/bin/env python3
"""Canonical fire-badger skins with coherent coat seams, grounded paws and articulated fire blades."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from fire_badger_sculpt import sculpture,SPECIES,HEIGHTS,arm_path,fire_specs
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'quilava': '3257af9b39de38a6140fe858ea41e7f55a2372a26e2ea46b8dd3e2eadb53096c', 'typhlosion': '4a5f8f7b25c03d42556da0b55a6af3eaad6be1dc77cd684959bd2d11819072fe'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,HEIGHTS[species]/(hi-lo)
def normalized(species,point):
 lo,scale=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*scale for i,v in enumerate(point))

def joints(species):
 if species=='quilava':
  raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.28,-.20)),('head',1,(0.,.43,.15))]
  for row in (0,1):
   for side in (-1,1):raw.append((('foreleg_' if row==0 else 'hindleg_')+('left' if side<0 else 'right'),0,arm_path(species,side,row)[0][0]))
 else:
  raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.50,0.)),('head',1,(0.,1.07,.045)),('jaw',2,(0.,1.07,.055)),
       ('arm_left',1,arm_path(species,-1)[0][0]),('arm_right',1,arm_path(species,1)[0][0]),('wrist_left',4,arm_path(species,-1)[0][-1]),('wrist_right',5,arm_path(species,1)[0][-1]),
       ('leg_left',0,(-.19,.20,0.)),('leg_right',0,(.19,.20,0.))]
 raw += [('fire_'+label.replace(' ','_'),1,root) for label,root,tip,w,twist in fire_specs(species)]
 return tuple((n,p,(0.,0.,0.) if p is None else normalized(species,v)) for n,p,v in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))

def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def influences(pairs):
 pairs=[(j,w) for j,w in pairs if w>1e-12]
 if len(pairs)==1:return rigid(pairs[0][0])
 return [j for j,w in pairs]+[0]*(4-len(pairs)),[w for j,w in pairs]+[0.]*(4-len(pairs))
def skin_weights(species,title,point):
 if title.startswith(('Foot /','Toe /')):return rigid(0)
 if title.startswith(('Body /','Collar /')):return rigid(1)
 if title.startswith(('Head /','Eye socket /','Eye white /','Iris /','Pupil /','Glint /','Nose /','Ear /','Fang /','Lip /')):return rigid(2)
 if title.startswith(('Jaw /','Tongue /')):return rigid(3)
 if title.startswith('Fire /'):
  label=title.split(' / ')[1];return rigid((7 if species=='quilava' else 10)+[v[0] for v in fire_specs(species)].index(label))
 if title.startswith('Leg /'):
  side,row=map(int,title.split()[-2:]);lo,scale=authoring_transform(species);a=smoothstep(.065,.23,point[1]/scale+lo);return influences(((0,1.-a),(3+row*2+(0 if side<0 else 1),a)))
 if title.startswith('Haunch /'):
  lo,scale=authoring_transform(species);a=smoothstep(.06,.19,point[1]/scale+lo);return influences(((0,1.-a),(8 if title.endswith('-1') else 9,a)))
 if title.startswith('Arm /'):return rigid(4 if title.endswith('-1') else 5)
 if title.startswith(('Hand /','Finger /')):
  side=int(title.split()[-2] if title.startswith('Finger /') else title.split()[-1]);return rigid(6 if side<0 else 7)
 raise ValueError('unowned fire badger anatomy: '+title)

def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid fire badger clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,-.25),(.34,1.),(.63,.18),(1.,0.)));b=envelope(t,((0.,0.),(.26,-.20),(.46,1.),(.76,-.08),(1.,0.)));strength=2.0 if clip=='attack' else -1.7
 a*=strength;b*=strength;out=[]
 for name,parent,pivot in JOINTS[species]:
  side=-1 if name.endswith('left') else 1
  if name=='root':angle=(0.,0.,0.)
  elif name=='body':angle=(0.,.010*a,0.)
  elif name=='head':angle=(.010*a,.025*b,0.)
  elif name=='jaw':angle=(.015*b if clip=='idle' else .060*abs(b),0.,0.)
  elif name.startswith('arm_'):angle=(-.070*a,side*.025*b,side*.018*a)
  elif name.startswith('wrist_'):angle=(-.050*b,side*.025*a,side*.018*b)
  elif 'leg_' in name:angle=(0.,side*.022*b,0.)
  elif name.startswith('fire_'):
   j=int(name.split('_')[-1]);phase=1. if j%2 else -1.;angle=(.025*b,phase*.025*a,phase*.040*b)
  else:raise ValueError(name)
  out.append(angle)
 return out

def pack_parts(model):
 result=[];groups={};species=model['name']
 for order,p in enumerate(model['primitives']):
  title=p['part'];prefix=title.split(' /')[0]
  group='head ink' if prefix in ('Eye socket','Pupil','Nose','Lip') else 'head ivory' if prefix in ('Eye white','Glint','Fang') else prefix if prefix in ('Iris','Finger','Toe','Fire') else None
  if group is None:result.append((p,{'source_order':order}));continue
  owner=skin_weights(species,title,(0.,0.,0.))[0][0];side=' '.join(title.split()[-3:-1]) if prefix=='Toe' else '';key=(group,owner,side,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=(prefix+' / complete '+group+' '+str(owner)+' '+side).rstrip()+' '+str(len(groups)),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3
  meta['source_parts'].append(dict(part=title,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
 return result

def export_fire_badger(species, model=None):
    if species not in SPECIES:raise ValueError('unknown fire badger')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed fire badger geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated fire badger standard skin v1'
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
        'geometry_source':'tools/fire_badger_sculpt.py','animation_contract':'rotation-only source cues; fixed soles; shared two-tone coat seams; fitted rigid gaze; complete digits; open adult jaw; independently fluttering coherent fire blades'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('fire badger exceeds existing model limits')
    return blob


def decode_fire_badger(blob, species):
    if len(blob)>1024*1024 or blob!=export_fire_badger(species):
        raise ValueError('noncanonical fire badger GLB')
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
def canonical_blob(species):return export_fire_badger(species,authoring_model(species))
def export_quilava(model=None):return export_fire_badger('quilava',model)
def export_typhlosion(model=None):return export_fire_badger('typhlosion',model)
def read_fire_badger(path,species):
 with Path(path).open('rb') as f:return decode_fire_badger(f.read(1024*1024+1),species)
def read_quilava(path):return read_fire_badger(path,'quilava')
def read_typhlosion(path):return read_fire_badger(path,'typhlosion')
