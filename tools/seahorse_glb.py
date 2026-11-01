#!/usr/bin/env python3
"""Canonical skin candidates for original hollow-snouted coiled sea creatures."""
import hashlib
import json
import math
import struct
from functools import lru_cache
from pathlib import Path
from seahorse_sculpt import sculpture as small_sculpture, tail_path as small_tail
from kingdra_sculpt import sculpture as dragon_sculpture, tail_path as dragon_tail
from bovine_sculpt import smooth
from skin_glb import Glb, COORDINATES, packed, quantized_weights, quaternion_xyz, smoothstep, envelope

SPECIES = ('horsea','seadra','kingdra')
HEIGHTS = dict.fromkeys(SPECIES,1.07)
REST_DIGESTS = {
    'horsea':'1d71e4150b202fc44087225fd5faee2f7145de1e9908bb0f5300c1069614814c',
    'seadra':'85b7509d919860ae3f0cf71a914251e1aa2b962b9d355286d7f95b24fc60d1a5',
    'kingdra':'4ab6120c7b38e50dbe6f6fb9576d64349656f9c94f9e9cfe495ebadf086642a6',
}


def sculpture(species,normalize=True):
    if species not in SPECIES:raise ValueError('unknown seahorse species')
    return (dragon_sculpture if species=='kingdra' else small_sculpture)(species,normalize=normalize)


def geometry_digest(model):
    h = hashlib.sha256()
    for p in model['primitives']:
        for field,component in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            h.update(packed(p[field],component))
    return h.hexdigest()


@lru_cache(maxsize=3)
def authoring_model(species):return sculpture(species)


PARTS = {s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}


@lru_cache(maxsize=3)
def authoring_transform(species):
    raw = sculpture(species,False)
    lo = min(v for p in raw['primitives'] for v in p['positions'][1::3])
    hi = max(v for p in raw['primitives'] for v in p['positions'][1::3])
    return lo,HEIGHTS[species]/(hi-lo)


def normalized(species,point):
    lo,scale = authoring_transform(species)
    return tuple((v-(lo if i==1 else 0.))*scale for i,v in enumerate(point))


@lru_cache(maxsize=3)
def tail_samples(species):
    points,radii = dragon_tail() if species=='kingdra' else smooth(*small_tail())
    distance = [0.]
    for a,b in zip(points,points[1:]):distance.append(distance[-1]+math.dist(a,b))
    return points,radii,distance


def tail_at(species,fraction):
    points,radii,distance = tail_samples(species);target = distance[-1]*fraction
    for i in range(len(points)-1):
        if distance[i+1]>=target:
            t = (target-distance[i])/(distance[i+1]-distance[i])
            return tuple(a+(b-a)*t for a,b in zip(points[i],points[i+1]))
    return points[-1]


def joints(species):
    dragon = species=='kingdra'
    raw = [('root',None,(0.,0.,0.)),('body',0,(0.,.43,0.)),
           ('head',1,(0.,.79,.03) if dragon else (0.,.80,-.02)),
           ('fin_left',1,(-.10,.54,-.08) if dragon else (-.075,.54,-.08)),
           ('fin_right',1,(.10,.54,-.08) if dragon else (.075,.54,-.08)),
           ('head_fin_left',2,(-.13,.82,.09) if dragon else (-.14,.91,-.045)),
           ('head_fin_right',2,(.13,.82,.09) if dragon else (.14,.91,-.045)),
           ('tail_base',1,tail_at(species,0.)),('tail_middle',7,tail_at(species,.30)),
           ('tail_tip',8,tail_at(species,.70))]
    if dragon:raw += [('crown_left',2,(-.09,.971,-.05)),('crown_right',2,(.09,.971,-.05))]
    return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for n,parent,p in raw)


JOINTS = {s:joints(s) for s in SPECIES}
CLIPS = (('idle',3.2,81),('attack',1.,65),('hit',1.,65))


def skin_weights(species,title,position):
    def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
    if title.startswith(('Body /','Belly /','Belly fold /','Body seam /','Armor /')):return rigid(1)
    if title.startswith(('Head /','Snout /','Eye socket /','Eye white /','Iris /','Pupil /','Glint /','Cheek /')):return rigid(2)
    if title.startswith(('Back fin /','Dorsal fin /','Dorsal horn /','Dorsal fork /','Fin ray /','Fin tip /')):
        side = int(title.split()[-2] if title.startswith(('Fin ray /','Fin tip /')) else title.split()[-1])
        return rigid(3+(side>0))
    if title.startswith(('Head fin /','Neck fin /')):
        side = int(title.split()[-2] if species=='horsea' else title.split()[-1])
        return rigid(5+(side>0))
    if title.startswith(('Crown /','Crown branch /')):
        side = int(title.split()[-2] if title.startswith('Crown branch /') else title.split()[-1])
        return rigid(10+(side>0))
    if title.startswith('Tail /'):
        lo,scale = authoring_transform(species)
        point = tuple(v/scale+(lo if i==1 else 0.) for i,v in enumerate(position))
        points,radii,distance = tail_samples(species);candidates = []
        # Match the actual authored cross-section circle. A nearest-centerline
        # projection can jump to the adjacent turn of a tightly curled tail.
        for i,center in enumerate(points):
            tangent = tuple(b-a for a,b in zip(points[max(0,i-1)],points[min(len(points)-1,i+1)]))
            length = math.sqrt(sum(v*v for v in tangent));tangent = tuple(v/length for v in tangent)
            delta = tuple(p-c for p,c in zip(point,center));axial = sum(d*t for d,t in zip(delta,tangent))
            radial = math.sqrt(max(0.,sum(d*d for d in delta)-axial*axial))
            candidates.append((axial*axial+(radial-radii[i])**2,distance[i]/distance[-1]))
        t = min(candidates)[1];a = smoothstep(.20,.48,t);b = smoothstep(.58,.88,t)
        return [7,8,9,0],[1.-a,a*(1.-b),a*b,0.]
    raise ValueError('unowned seahorse anatomy: '+title)


def pose_angles(species,clip,t):
    if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:
        raise ValueError('invalid seahorse animation')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
    if clip=='idle':
        a = math.sin(math.tau*t);b = math.sin(math.tau*t-.45)+math.sin(.45)
        body,head,fin,headfin,tail,crown = .014,.012,.055,.025,.045,.018
    else:
        a = envelope(t,((0.,0.),(.18,-.30),(.34,1.),(.63,.18),(1.,0.)))
        b = envelope(t,((0.,0.),(.26,-.25),(.46,1.),(.76,-.08),(1.,0.)))
        body,head,fin,headfin,tail,crown = ((.04,-.10,.22,.075,.12,.045) if clip=='attack' else (-.025,.06,-.10,-.05,-.07,-.035))
    # Body/tail yaw preserves the authored waterline without a floor clamp.
    angles = [(0.,0.,0.),(0.,body*a,0.),(head*a,.012*b,.006*b),
              (.008*b,-fin*a,-.025*b),(-.008*a,fin*b,.025*a),
              (.012*b,-headfin*a,-.008*b),(-.012*a,headfin*b,.008*a),
              (0.,tail*a,0.),(0.,-tail*b*1.1,0.),(0.,tail*a*1.25,0.)]
    if species=='kingdra':angles += [(0.,-crown*b,-.018*a),(0.,crown*a,.018*b)]
    return angles


def pack_parts(model):
    result = [];groups = {}
    for order,p in enumerate(model['primitives']):
        title = p['part'];prefix = title.split(' /')[0]
        group = (prefix if prefix in ('Belly fold','Body seam') else
                 prefix+' '+title.split()[-2] if prefix in ('Fin ray','Crown branch') or prefix=='Head fin' and len(title.split())>1 and title.split()[-2] in ('-1','1') else None)
        if group is None:result.append((p,{'source_order':order}));continue
        if group not in groups:
            q = dict(part=prefix+' / complete '+group,positions=[],normals=[],indices=[],base_color=p['base_color'])
            meta = {'source_parts':[]};groups[group] = (q,meta);result.append((q,meta))
        q,meta = groups[group];start = len(q['positions'])//3
        meta['source_parts'].append(dict(part=title,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
        q['positions'] += p['positions'];q['normals'] += p['normals'];q['indices'] += [i+start for i in p['indices']]
    return result


def export_seahorse(species, model=None):
    if species not in SPECIES:raise ValueError('unknown seahorse family')
    if model is None:model = authoring_model(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed seahorse family geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated seahorse family standard skin v1'
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
        'geometry_source':'tools/kingdra_sculpt.py' if species=='kingdra' else 'tools/seahorse_sculpt.py','animation_contract':'rotation-only source cues; preserved authored waterline; coherent hollow snout and fitted face/belly; independent head/fins, three tail carriers and dragon crowns'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('seahorse family exceeds existing model limits')
    return blob


def decode_seahorse(blob, species):
    if len(blob)>1024*1024 or blob!=export_seahorse(species):
        raise ValueError('noncanonical seahorse family GLB')
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


def read_seahorse(path, species):
    with Path(path).open('rb') as f:
        return decode_seahorse(f.read(1024*1024+1),species)


def export_horsea(model=None):return export_seahorse('horsea',model)
def export_seadra(model=None):return export_seahorse('seadra',model)
def export_kingdra(model=None):return export_seahorse('kingdra',model)
def read_horsea(path):return read_seahorse(path,'horsea')
def read_seadra(path):return read_seahorse(path,'seadra')
def read_kingdra(path):return read_seahorse(path,'kingdra')
