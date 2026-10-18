#!/usr/bin/env python3
"""Original Clefable and Wigglytuff skins with coherent fitted faces and feet."""
import hashlib
import json
import math
import struct
from functools import lru_cache
from pathlib import Path
from clefable_sculpt import sculpture as clefable_sculpture
from wigglytuff_sculpt import sculpture as wigglytuff_sculpture
from skin_glb import Glb, COORDINATES, packed, quantized_weights, quaternion_xyz, smoothstep, envelope

SPECIES = ('clefable', 'wigglytuff')
HEIGHTS = dict.fromkeys(SPECIES, 1.)
REST_DIGESTS = {
    'clefable': '2dcfdca000ef1ccc8fa751fe5e95753104aa8097685d5200ff18cb6dbcf1c3e8',
    'wigglytuff': 'ba8c7ae0e3fadd9d0a8f91ae55d2ab67ae4e2aed3432ea20df0254265ccf7614',
}


def sculpture(species, normalize=True):
    if species not in SPECIES:raise ValueError('unknown fairy pair species')
    return (clefable_sculpture if species=='clefable' else wigglytuff_sculpture)(normalize=normalize)


def geometry_digest(model):
    h = hashlib.sha256()
    for p in model['primitives']:
        for field, component in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            h.update(packed(p[field],component))
    return h.hexdigest()


@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)


PARTS = {s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}


@lru_cache(maxsize=2)
def authoring_transform(species):
    raw = sculpture(species, False)
    lo = min(v for p in raw['primitives'] for v in p['positions'][1::3])
    hi = max(v for p in raw['primitives'] for v in p['positions'][1::3])
    return lo, HEIGHTS[species]/(hi-lo)


def normalized(species, point):
    lo, scale = authoring_transform(species)
    return tuple((v-(lo if i==1 else 0.))*scale for i,v in enumerate(point))


def joints(species):
    clef = species=='clefable'
    raw = [('root',None,(0.,0.,0.)), ('body',0,(0.,.15,0.)),
           ('ear_left',1,(-.155,.64 if clef else .72,.01)),
           ('ear_right',1,(.155,.64 if clef else .72,.01)),
           ('arm_left',1,(-.23,.34 if clef else .38,.035)),
           ('arm_right',1,(.23,.34 if clef else .38,.035)),
           ('tuft',1,(0.,.645 if clef else .77,.10))]
    if clef:
        raw += [('wing_left',1,(-.17,.43,-.17)),('wing_right',1,(.17,.43,-.17)),
                ('tail_base',1,(0.,.25,-.16)),('tail_tip',9,(.15,.17,-.32))]
    return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for n,parent,p in raw)


JOINTS = {s:joints(s) for s in SPECIES}
CLIPS = (('idle',3.2,81),('attack',1.,65),('hit',1.,65))


def skin_weights(species, title, position):
    def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
    if title.startswith(('Foot /','Toe /')):return rigid(0)
    if title.startswith(('Ear /','Ear tip /','Ear lining /')):
        return rigid(2+(int(title.split()[-1])>0))
    if title.startswith(('Arm /','Hand finger /','Hand claw /')):
        side = int(title.split()[-1] if title.startswith('Arm /') else title.split()[-2])
        return rigid(4+(side>0))
    if title.startswith(('Tuft /','Tuft crease /')):return rigid(6)
    # The existing rear-facing wing recipe rotates X and Z together. Its
    # authored suffix therefore has the opposite sign to the stored X side.
    if title.startswith('Wing /'):return rigid(7+(int(title.split()[-1])<0))
    if title.startswith('Tail /'):
        lo,scale = authoring_transform(species);z = position[2]/scale
        a = smoothstep(.23,.31,-z)
        return [9,10,0,0],[1.-a,a,0.,0.]
    if title.startswith(('Body /','Belly /','Eye /','Eye glint /','Eye border /','Eye white /',
                         'Iris /','Iris sheen /','Cheek /','Mouth /','Tongue /')):
        return rigid(1)
    raise ValueError('unowned fairy pair anatomy: '+title)


def pose_angles(species, clip, t):
    if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:
        raise ValueError('invalid fairy pair animation')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
    if clip=='idle':
        a = math.sin(math.tau*t);b = math.sin(math.tau*t-.45)+math.sin(.45)
        body,ear,arm,tuft,wing,tail = .007,.025,.035,.030,.035,.025
    else:
        a = envelope(t,((0.,0.),(.18,-.30),(.34,1.),(.63,.18),(1.,0.)))
        b = envelope(t,((0.,0.),(.26,-.25),(.46,1.),(.76,-.08),(1.,0.)))
        body,ear,arm,tuft,wing,tail = ((.016,.09,-.35,.075,.19,.08) if clip=='attack'
                                     else (-.012,-.06,.10,-.05,-.09,-.055))
    return ([(0.,0.,0.),(body*a,.009*b,.004*b),
             (ear*b,.012*a,-.035*a),(-ear*a,-.012*b,.035*b),
             (arm*a,.018*b,-.055*b),(arm*a,-.018*b,.055*b),
             (tuft*b,.020*a,0.)] +
            ([(.018*b,-wing*a,-.018*a),(-.018*a,wing*b,.018*b),
              (.012*b,tail*a,0.),(.018*a,tail*b,0.)] if species=='clefable' else []))


def pack_parts(model):
    return [(p,{'source_order':i}) for i,p in enumerate(model['primitives'])]


def export_fairy_pair(species, model=None):
    if species not in SPECIES:raise ValueError('unknown fairy pair')
    if model is None:model = authoring_model(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed fairy pair geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated fairy pair standard skin v1'
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
        'geometry_source':'tools/'+species+'_sculpt.py','animation_contract':'rotation-only source cues; complete fixed feet; coherent fitted face/belly; independent ears, arms and forelock; Clefable wings and two tail carriers'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('fairy pair exceeds existing model limits')
    return blob


def decode_fairy_pair(blob, species):
    if len(blob)>1024*1024 or blob!=export_fairy_pair(species):
        raise ValueError('noncanonical fairy pair GLB')
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


def read_fairy_pair(path, species):
    with Path(path).open('rb') as f:
        return decode_fairy_pair(f.read(1024*1024+1),species)


def export_clefable(model=None):return export_fairy_pair('clefable',model)
def export_wigglytuff(model=None):return export_fairy_pair('wigglytuff',model)
def read_clefable(path):return read_fairy_pair(path,'clefable')
def read_wigglytuff(path):return read_fairy_pair(path,'wigglytuff')
