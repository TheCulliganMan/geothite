#!/usr/bin/env python3
"""Nautilus skins with ten grounded tentacle controls and a genuinely open adult beak."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from omanyte_sculpt import sculpture as small,HEIGHT as SMALL_HEIGHT
from omastar_sculpt import sculpture as adult,HEIGHT as ADULT_HEIGHT
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope
SPECIES=('omanyte','omastar')
HEIGHTS={'omanyte':SMALL_HEIGHT,'omastar':ADULT_HEIGHT}

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):
 if species not in SPECIES:raise ValueError('unknown nautilus family')
 return (small if species=='omanyte' else adult)()

REST_DIGESTS={'omanyte': '70c60a2c6dbd169dd870c5af37ef35c2c2c96ecc366cfff95eb12ae93835532d', 'omastar': '598f572bd5aee2e09db4e9b647c6629c3b48a89ba2cf64e9d72e25c8ddb930c4'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}

@lru_cache(maxsize=2)
def raw_model(species):return (small if species=='omanyte' else adult)(False)

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=raw_model(species);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,HEIGHTS[species]/(hi-lo)

def normalized(species,point):
 lo,scale=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*scale for i,v in enumerate(point))

@lru_cache(maxsize=2)
def tentacles(species):return tuple(p['part'] for p in authoring_model(species)['primitives'] if p['part'].startswith('Tentacle /'))

def root_center(part):
 # Each 18-sided closed paddle begins with 16 root-cap triangles. Read that ring,
 # including the juvenile fan's source-authored individual floor translation.
 points=[tuple(part['positions'][i:i+3]) for i in range(0,len(part['positions']),3)]
 ring={points[i] for i in part['indices'][:48]}
 if len(ring)!=18:raise ValueError('unexpected source root cap: '+part['part']+' '+str(len(ring)))
 return tuple(sum(p[i] for p in sorted(ring))/len(ring) for i in range(3))

def joints(species):
 body=(0.,.185,.150) if species=='omanyte' else (0.,.265,.165)
 shell=(0.,.49,-.045) if species=='omanyte' else (0.,.49*1.12+.13,-.045*1.12-.015)
 ey,ez,ex=(.265,.322,.100) if species=='omanyte' else (.410,.348,.115)
 raw=[('root',None,(0.,0.,0.)),('body',0,body),('shell',1,shell),('eye_left',1,(-ex,ey,ez)),('eye_right',1,(ex,ey,ez))]
 for i,part in enumerate(p for p in raw_model(species)['primitives'] if p['part'].startswith('Tentacle /')):raw.append(('crawl_tentacle_'+str(i),0,root_center(part)))
 if species=='omastar':
  for sx in (-1,1):
   for sy in (-1,1):raw.append(('beak_'+('left' if sx<0 else 'right')+'_'+('lower' if sy<0 else 'upper'),1,(sx*.050,.257+sy*.050,.375)))
 return tuple((n,p,(0.,0.,0.) if p is None else normalized(species,v)) for n,p,v in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))

def owner(species,title):
 if title.startswith('Body /'):return 1
 if title.startswith(('Shell /','Shell seam /','Shell spiral /','Spine /')):return 2
 if title.startswith(('Eye /','Iris /','Eye gleam /','Pupil /','Glint /')):return 3 if title.endswith('-1') else 4
 if title.startswith('Tentacle /'):return 5+tentacles(species).index(title)
 if title.startswith('Mouth /'):return 1
 if title.startswith('Beak /'):
  sx,sy=map(int,title.split()[-2:]);return 15+(0 if sx<0 else 2)+(0 if sy<0 else 1)
 raise ValueError('unowned nautilus anatomy: '+title)

def skin_weights(species,title,point):return [owner(species,title),0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid nautilus clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,-.25),(.34,1.),(.63,.18),(1.,0.)));b=envelope(t,((0.,0.),(.26,-.20),(.46,1.),(.76,-.08),(1.,0.)));strength=2.0 if clip=='attack' else -1.7
 a*=strength;b*=strength;out=[]
 for i,(name,parent,pivot) in enumerate(JOINTS[species]):
  if name=='root':angle=(0.,0.,0.)
  elif name=='body':angle=(0.,.014*a,0.)
  elif name=='shell':angle=(0.,.025*b,0.)
  elif name.startswith('eye_'):angle=(0.,(-1 if name.endswith('left') else 1)*.030*b,0.)
  elif name.startswith('crawl_tentacle_'):
   n=i-5;wave=math.sin(math.tau*t-n*.65)+math.sin(n*.65) if clip=='idle' else a*math.cos(n*.65)+b*.4*math.sin(n*.65);angle=(0.,.045*wave,0.)
  else:
   side=-1 if 'left' in name else 1;vertical=-1 if name.endswith('lower') else 1;angle=(0.,0.,side*vertical*.035*b)
  out.append(angle)
 return out

def pack_parts(model):
 result=[];groups={}
 for order,p in enumerate(model['primitives']):
  title=p['part'];prefix=title.split(' /')[0];group=prefix if prefix in ('Shell seam','Shell spiral','Spine') else None
  if group is None:result.append((p,{'source_order':order}));continue
  key=(group,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=prefix+' / complete '+group,positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3
  meta['source_parts'].append(dict(part=title,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
 return result

def export_nautilus(species, model=None):
    if species not in SPECIES:raise ValueError('unknown nautilus family')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed nautilus family geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated nautilus family standard skin v1'
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
        'geometry_source':'tools/omanyte_sculpt.py and tools/omastar_sculpt.py','animation_contract':'rotation-only source cues; authored ground contacts and open beak cross; coherent shell ribs, spirals and spines; ten independent tentacles, shell, eyes and beak quadrants'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('nautilus family exceeds existing model limits')
    return blob


def decode_nautilus(blob, species):
    if len(blob)>1024*1024 or blob!=export_nautilus(species):
        raise ValueError('noncanonical nautilus family GLB')
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
def canonical_blob(species):return export_nautilus(species,authoring_model(species))
def export_omanyte(model=None):return export_nautilus('omanyte',model)
def export_omastar(model=None):return export_nautilus('omastar',model)
def read_nautilus(path,species):
 with Path(path).open('rb') as f:return decode_nautilus(f.read(1024*1024+1),species)
def read_omanyte(path):return read_nautilus(path,'omanyte')
def read_omastar(path):return read_nautilus(path,'omastar')
