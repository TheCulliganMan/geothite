#!/usr/bin/env python3
"""Canonical armored-mammal skins with complete supported per-limb digits."""
import hashlib
import json
import math
import struct
from functools import lru_cache
from pathlib import Path
from sand_mammal_sculpt import sculpture, SPECIES, HEIGHTS
from skin_glb import Glb, COORDINATES, packed, quantized_weights, quaternion_xyz, smoothstep, envelope

REST_DIGESTS = {
    'sandshrew': '0b32692642bd87c17470fc0f74ed582ce3947706815903e432f8dee73866f314',
    'sandslash': 'a659e2f579e749032c71e861f3c7400665afa6c779f215b4c117cc28a96f4ccd',
}


def geometry_digest(model):
    h = hashlib.sha256()
    for p in model['primitives']:
        for field, component in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            h.update(packed(p[field],component))
    return h.hexdigest()


@lru_cache(maxsize=2)
def authoring_model(species):
    return sculpture(species)


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
    raw = [('root',None,(0.,0.,0.)), ('body',0,(0.,.43,0.)),
           ('head',1,(0.,.68,.045)), ('ear_left',2,(-.135,.905,.015)),
           ('ear_right',2,(.135,.905,.015))]
    for side in (-1,1):
        first = len(raw); suffix = 'left' if side<0 else 'right'
        raw += [('arm_'+suffix,1,(side*.145,.55,.005)),
                ('forearm_'+suffix,first,(side*.31,.43,.055)),
                ('wrist_'+suffix,first+1,(side*.35,.34,.15))]
    raw += [('leg_left',1,(-.18,.215,-.01)), ('leg_right',1,(.18,.215,-.01)),
            ('tail_base',1,(0.,.245,-.12)), ('tail_tip',13,(0.,.16,-.32))]
    if species=='sandslash':
        raw += [('quills_lower',1,(0.,.34,-.19)),('quills_upper',1,(0.,.62,-.19))]
    return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for n,parent,p in raw)


JOINTS = {s:joints(s) for s in SPECIES}
CLIPS = (('idle',3.2,81),('attack',1.,65),('hit',1.,65))


def skin_weights(species, title, position):
    lo, scale = authoring_transform(species)
    x,y,z = (v/scale+(lo if i==1 else 0.) for i,v in enumerate(position))
    def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
    if title.startswith(('Foot /','Toe /')):return rigid(0)
    if title.startswith(('Body /','Belly /','Shell /','Plate /')):return rigid(1)
    if title.startswith(('Head /','Muzzle /','Nose /','Eye /','Pupil /','Forehead plate /','Bridge plate /')):return rigid(2)
    if title.startswith(('Ear /','Ear inset /')):return rigid(3+(int(title.split()[-1])>0))
    if title.startswith(('Hand /','Claw /')):
        side = int(title.split()[-2] if title.startswith('Claw /') else title.split()[-1])
        return rigid(7+(3 if side>0 else 0))
    if title.startswith('Arm /'):
        base = 5+(3 if int(title.split()[-1])>0 else 0)
        # Arc projection along both authored segments, rather than Y bands.
        side = -1 if base==5 else 1
        points = [(side*.145,.55,.005),(side*.31,.43,.055),(side*.35,.34,.15)]
        candidates = []
        for segment,(a,b) in enumerate(zip(points,points[1:])):
            d = tuple(v-u for u,v in zip(a,b));length = sum(v*v for v in d)
            t = max(0.,min(1.,sum((v-u)*w for v,u,w in zip((x,y,z),a,d))/length))
            candidates.append((sum((v-u-t*w)**2 for v,u,w in zip((x,y,z),a,d)),segment+t))
        t = min(candidates)[1];a = smoothstep(.45,1.45,t);b = smoothstep(1.55,1.95,t)
        return [base,base+1,base+2,0],[1.-a,a*(1.-b),a*b,0.]
    if title.startswith(('Haunch /','Leg /')):
        owner = 11+(int(title.split()[-1])>0)
        if title.startswith('Haunch /'):return rigid(owner)
        a = smoothstep(.075,.18,y)
        return [0,owner,0,0],[1.-a,a,0.,0.]
    if title.startswith('Tail /'):
        a = smoothstep(.24,.43,-z)
        return [13,14,0,0],[1.-a,a,0.,0.]
    if title.startswith('Quill /'):
        row = int(title.split()[-2]);owner = 15 if row==0 else 16
        a = smoothstep(.32,.47,-z)
        return [1,owner,0,0],[1.-a,a,0.,0.]
    raise ValueError('unowned armored mammal anatomy: '+title)


def pose_angles(species, clip, t):
    if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:
        raise ValueError('invalid armored mammal animation')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
    a = math.sin(math.tau*t) if clip=='idle' else envelope(t,((0.,0.),(.18,-.35),(.36,1.),(.65,.15),(1.,0.)))
    b = math.sin(math.tau*t-.4)+math.sin(.4) if clip=='idle' else envelope(t,((0.,0.),(.25,-.25),(.44,1.),(.75,0.),(1.,0.)))
    idle = clip=='idle';hit = clip=='hit'
    angles = [(0.,0.,0.),((.004 if idle else -.012 if hit else .010)*a,.008*b,0.),
              (.009*b,.014*a,0.),(.012*a,0.,-.015*b),(-.012*a,0.,.015*b)]
    for side in (-1,1):
        upper = .012*b if idle else -.055*a if hit else .23*a
        lower = .018*a if idle else -.045*b if hit else .28*b
        angles += [(upper,side*.012*b,side*.008*a),(lower,0.,0.),(.014*b,side*.018*a,0.)]
    angles += [(.008*b,0.,0.),(-.008*b,0.,0.),(.014*b,.025*a,0.),(.020*a,.035*b,0.)]
    if species=='sandslash':angles += [(.008*a,0.,.006*b),(-.008*b,0.,-.006*a)]
    return angles


def pack_parts(model):
    result = [];groups = {}
    for order,p in enumerate(model['primitives']):
        prefix = p['part'].split(' /')[0]
        group = ('armor' if prefix=='Plate' else 'quills' if prefix=='Quill' else
                 prefix+' '+p['part'].split()[-2] if prefix in ('Toe','Claw') else None)
        if group is None:result.append((p,{'source_order':order}));continue
        if group not in groups:
            category = 'Hand claw' if prefix=='Claw' else prefix
            q = dict(part=category+' / complete '+group,positions=[],normals=[],indices=[],base_color=p['base_color'])
            meta = {'source_parts':[]};groups[group] = (q,meta);result.append((q,meta))
        q,meta = groups[group];start = len(q['positions'])//3
        meta['source_parts'].append(dict(part=p['part'],order=order,vertex_start=start,
            vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
        q['positions'] += p['positions'];q['normals'] += p['normals'];q['indices'] += [i+start for i in p['indices']]
    return result


def export_sand_mammal(species, model=None):
    if species not in SPECIES:raise ValueError('unknown armored mammal')
    if model is None:model = authoring_model(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=[p['part'] for p in authoring_model(species)['primitives']]):
        raise ValueError('unreviewed armored mammal geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original armored digging mammals standard skin v1'
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
        'geometry_source':'tools/sand_mammal_sculpt.py','animation_contract':'rotation-only source cues; fixed complete feet; supported digging arms; independent head, ears and two tail carriers'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('armored mammal exceeds existing model limits')
    return blob


def decode_sand_mammal(blob, species):
    if len(blob)>1024*1024 or blob!=export_sand_mammal(species):
        raise ValueError('noncanonical armored mammal GLB')
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


def read_sand_mammal(path, species):
    with Path(path).open('rb') as f:
        return decode_sand_mammal(f.read(1024*1024+1),species)


def export_sandshrew(model=None):return export_sand_mammal('sandshrew',model)
def export_sandslash(model=None):return export_sand_mammal('sandslash',model)
def read_sandshrew(path):return read_sand_mammal(path,'sandshrew')
def read_sandslash(path):return read_sand_mammal(path,'sandslash')
