#!/usr/bin/env python3
"""Aerodactyl standard skin with coherent toothed jaws, scalloped wings and arrow tail."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from aerodactyl_sculpt import sculpture,HEIGHT
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep
SPECIES=('aerodactyl',)
HEIGHTS={'aerodactyl':HEIGHT}

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()


@lru_cache(maxsize=1)
def authoring_model(species):
 if species not in SPECIES:raise ValueError('unknown fossil flyer')
 return sculpture()

REST_DIGESTS={'aerodactyl':'f49ef0e44a17c7426e61b0c429eb3215a39aa03fd15a48cf57d24a199b29aa0a'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}

@lru_cache(maxsize=1)
def authoring_transform():
 raw=sculpture(False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,HEIGHT/(hi-lo)

def normalized(point):
 lo,scale=authoring_transform();return tuple((v-(lo if i==1 else 0.))*scale for i,v in enumerate(point))

def joints():
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.44,-.06)),('head',1,(0.,.985,.015)),('jaw',2,(0.,.938,.07)),('wing_left',1,(-.105,.685,-.090294375)),('wing_right',1,(.105,.685,-.090294375)),('leg_left',0,(-.11,.31,-.040)),('leg_right',0,(.11,.31,-.040)),('tail',1,(0.,.29,-.13))]
 return tuple((n,p,(0.,0.,0.) if p is None else normalized(v)) for n,p,v in raw)
JOINTS={'aerodactyl':joints()}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))

def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def influences(pairs):
 pairs=[(j,w) for j,w in pairs if w>1e-12]
 if len(pairs)==1:return rigid(pairs[0][0])
 return [j for j,w in pairs]+[0]*(4-len(pairs)),[w for j,w in pairs]+[0.]*(4-len(pairs))

def skin_weights(species,title,point):
 lo,scale=authoring_transform();y=point[1]/scale+lo
 if title.startswith('Body /'):
  a=smoothstep(.20,.44,y);b=smoothstep(.86,.99,y);return influences(((0,1.-a),(1,a*(1.-b)),(2,a*b)))
 if title.startswith('Leg /'):
  j=6 if title.endswith('-1') else 7;a=smoothstep(.10,.22,y);return influences(((0,1.-a),(j,a)))
 if title.startswith(('Foot /','Foot claw /')):return rigid(0)
 if title.startswith(('Skull /','Skull horn /','Jaw hinge /','Eye socket /','Eye white /','Pupil /','Nostril /','Mouth ceiling /')):return rigid(2)
 if title.startswith(('Jaw /','Mouth floor /','Tongue /')):return rigid(3)
 if title.startswith('Tooth /'):return rigid(2 if ' upper ' in title else 3)
 if title.startswith(('Wing /','Wing arm /','Wing finger /','Wing claw /')):
  side=int(title.split()[-2] if title.startswith(('Wing finger /','Wing claw /')) else title.split()[-1]);return rigid(4 if side<0 else 5)
 if title.startswith(('Tail /','Tail tip /')):return rigid(8)
 raise ValueError('unowned fossil anatomy: '+title)

def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid fossil clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*9
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,-.25),(.34,1.),(.63,.18),(1.,0.)));b=envelope(t,((0.,0.),(.26,-.20),(.46,1.),(.76,-.08),(1.,0.)));strength=2.0 if clip=='attack' else -1.7
 a*=strength;b*=strength;out=[]
 for name,parent,pivot in JOINTS[species]:
  side=-1 if name.endswith('left') else 1
  if name=='root':angle=(0.,0.,0.)
  elif name=='body':angle=(0.,.018*a,0.)
  elif name=='head':angle=(.014*a,.035*b,0.)
  # Roars and recoil gasps open the fossil jaw; neither closes its identifying
  # gape below the reviewed source clearance. Idle retains a small jaw sway.
  elif name=='jaw':angle=(.025*b if clip=='idle' else .10*abs(b),0.,0.)
  elif name.startswith('wing_'):
   beat=math.sin(math.tau*t*4) if clip=='idle' else a;angle=(.020*b,side*.018*b,side*(.22*beat if clip=='idle' else .18*a))
  elif name.startswith('leg_'):angle=(0.,side*.025*b,0.)
  else:angle=(0.,-.075*b,0.)
  out.append(angle)
 return out

def pack_parts(model):
 result=[];groups={}
 for order,p in enumerate(model['primitives']):
  title=p['part'];prefix=title.split(' /')[0];owner=skin_weights('aerodactyl',title,(0.,0.,0.))[0][0]
  group=('head ink' if prefix in ('Eye socket','Pupil','Nostril') else prefix) if prefix in ('Jaw hinge','Skull horn','Eye socket','Eye white','Pupil','Nostril','Tooth','Wing finger','Wing claw','Foot claw') else None
  if group is None:result.append((p,{'source_order':order}));continue
  key=(group,owner,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=prefix+' / complete '+group+' '+str(owner),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3
  meta['source_parts'].append(dict(part=title,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
 return result

def export_fossil(species, model=None):
    if species not in SPECIES:raise ValueError('unknown fossil flyer')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed fossil flyer geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated fossil flyer standard skin v1'
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
        'geometry_source':'tools/aerodactyl_sculpt.py','animation_contract':'rotation-only source cues; fixed soles and open toothed jaw; coherent face, upper palate, lower lining and tongue; independent skull, mandible, wings, hind limbs and arrow tail'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('fossil flyer exceeds existing model limits')
    return blob


def decode_fossil(blob, species):
    if len(blob)>1024*1024 or blob!=export_fossil(species):
        raise ValueError('noncanonical fossil flyer GLB')
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



@lru_cache(maxsize=1)
def canonical_blob(species):return export_fossil(species,authoring_model(species))
def export_aerodactyl(model=None):return export_fossil('aerodactyl',model)
def decode_aerodactyl(blob):return decode_fossil(blob,'aerodactyl')
def read_aerodactyl(path):
 with Path(path).open('rb') as f:return decode_aerodactyl(f.read(1024*1024+1))
