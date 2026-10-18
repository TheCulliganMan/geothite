#!/usr/bin/env python3
"""Original dragon skins with grounded coils, coherent ventral ribbons and complete digits."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from dragon_family_sculpt import sculpture,spine_path,arm_path,arm_samples,SPECIES,HEIGHTS
from bovine_sculpt import smooth
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep


def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()


REST_DIGESTS={'dratini': '1ba39a11cb3500c09cdac49d48156bdaa58811b71ed00d436398137c62d68b8a', 'dragonair': 'd1025e2f13dc1785387b2f13a19ffb704a6eb3d617724aac1d49fd608f99cf3f', 'dragonite': '519e2bdb4227b1705a81f67b1a2b632d5007433449243dd69f79aee2d16eac60'}
@lru_cache(maxsize=3)
def authoring_model(species):return sculpture(species)
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}


@lru_cache(maxsize=3)
def authoring_transform(species):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,HEIGHTS[species]/(hi-lo)


def normalized(species,point):
 lo,scale=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*scale for i,v in enumerate(point))


@lru_cache(maxsize=3)
def spine_samples(species):
 points,radii,hint,segments,steps=spine_path(species);points,radii=smooth(points,radii,steps);distance=[0.]
 for p,q in zip(points,points[1:]):distance.append(distance[-1]+math.dist(p,q))
 return points,radii,distance,steps


def joints(species):
 if species=='dragonite':
  left,_=arm_path(-1);right,_=arm_path(1)
  raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.50,0.)),('head',1,(0.,1.01,.03)),
       ('antenna_left',2,(-.074,1.218,-.014)),('antenna_right',2,(.074,1.218,-.014)),
       ('wing_left',1,(-.13,.73,-.11)),('wing_right',1,(.13,.73,-.11)),
       ('arm_left',1,left[0]),('arm_right',1,right[0]),
       ('wrist_left',7,left[2]),('wrist_right',8,right[2]),
       ('leg_left',0,(-.23,.20,.04)),('leg_right',0,(.23,.20,.04)),
       ('tail_base',1,(0.,.28,-.12)),('tail_tip',13,(.38,.33,-.75))]
 else:
  points,radii,distance,steps=spine_samples(species);big=species=='dragonair';start=7 if big else 6;tip=int(7.5*steps) if big else 7*steps
  raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.54,.15)),('head',1,(0.,.86,.13)),
       ('ear_left',2,(-.076,1.014,.10) if big else (-.12,1.0,.10)),
       ('ear_right',2,(.076,1.014,.10) if big else (.12,1.0,.10)),
       ('tail_base',0,points[start*steps]),('tail_tip',5,points[tip])]
 return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for n,parent,p in raw)


JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))

def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def influences(pairs):
 pairs=[(j,w) for j,w in pairs if w>1e-12]
 if len(pairs)==1:return rigid(pairs[0][0])
 return [j for j,w in pairs]+[0]*(4-len(pairs)),[w for j,w in pairs]+[0.]*(4-len(pairs))


def raw_point(species,point):
 lo,scale=authoring_transform(species);return tuple(v/scale+(lo if i==1 else 0.) for i,v in enumerate(point))


def ring_distance(point,points,radii,distance,ribbon=False):
 candidates=[]
 for i,center in enumerate(points):
  tangent=tuple(b-a for a,b in zip(points[max(0,i-1)],points[min(len(points)-1,i+1)]));length=math.sqrt(sum(v*v for v in tangent));tangent=tuple(v/length for v in tangent)
  delta=tuple(p-c for p,c in zip(point,center));axial=sum(v*t for v,t in zip(delta,tangent));radial=math.sqrt(max(0.,sum(v*v for v in delta)-axial*axial))
  expected=(radii[i]+min(.002,radii[i]*.12),radii[i]-min(.001,radii[i]*.07)) if ribbon else (radii[i],)
  candidates.append((axial*axial+min((radial-r)**2 for r in expected),distance[i]))
 return min(candidates)[1]


def spine_distance(species,title,point):
 points,radii,distance,steps=spine_samples(species)
 return ring_distance(raw_point(species,point),points,radii,distance,title.startswith(('Belly /','Tail underside /')))

@lru_cache(maxsize=2)
def forelimb_samples(side):
 points,radii=arm_samples(side);distance=[0.]
 for p,q in zip(points,points[1:]):distance.append(distance[-1]+math.dist(p,q))
 return points,radii,distance


def skin_weights(species,title,point):
 if species!='dragonite':
  if title.startswith(('Body /','Belly /')):
   d=spine_distance(species,title,point);points,radii,dist,steps=spine_samples(species);start=7 if species=='dragonair' else 6;begin=dist[start*steps]
   if d>=begin:
    a=smoothstep(begin,begin+(.06 if species=='dragonair' else .12),d)
    lo,hi=(dist[int(7.25*steps)],dist[int(7.5*steps)]) if species=='dragonair' else (dist[7*steps]-.08,dist[7*steps]+.06)
    b=smoothstep(lo,hi,d);return influences(((0,1.-a),(5,a*(1.-b)),(6,a*b)))
   a=smoothstep(.20,.36,d);b=smoothstep(.44,.70,d);return influences(((0,b),(1,a*(1.-b)),(2,(1.-a)*(1.-b))))
  if title.startswith(('Ear fin /','Ear fold /')):return rigid(3 if title.endswith('-1') else 4)
  if title.startswith(('Orb /','Orb glint /')):return rigid(2 if title.endswith('0') else 6)
  if title.startswith(('Head /','Muzzle /','Eye socket /','Iris /','Pupil /','Glint /','Forehead /','Horn /')):return rigid(2)
 else:
  x,y,z=raw_point(species,point)
  if title.startswith(('Body /','Belly /','Belly fold /')):return rigid(1)
  if title.startswith(('Head /','Muzzle /','Eye socket /','Eye white /','Pupil /','Glint /','Mouth /','Nostril /','Horn /')):return rigid(2)
  if title.startswith('Antenna /'):return rigid(3 if title.endswith('-1') else 4)
  if title.startswith(('Wing /','Wing border /','Wing spar /','Wing rib /')):
   side=int(title.split()[-2] if title.startswith('Wing rib /') else title.split()[-1]);return rigid(5 if side<0 else 6)
  if title.startswith('Arm /'):
   side=int(title.split()[-1]);points,radii,distance=forelimb_samples(side);d=ring_distance((x,y,z),points,radii,distance);amount=smoothstep(.19,.33,d);return influences(((7 if side<0 else 8,1.-amount),(9 if side<0 else 10,amount)))
  if title.startswith(('Hand /','Hand claw /')):
   side=int(title.split()[-2] if title.startswith('Hand claw /') else title.split()[-1]);return rigid(9 if side<0 else 10)
  if title.startswith(('Haunch /','Leg /')):
   side=int(title.split()[-1]);amount=smoothstep(.10,.25,y);return influences(((0,1.-amount),(11 if side<0 else 12,amount)))
  if title.startswith(('Foot /','Foot claw /')):return rigid(0)
  if title.startswith(('Tail /','Tail underside /')):
   d=spine_distance(species,title,point);a=smoothstep(.12,.32,d);b=smoothstep(.55,.78,d);return influences(((1,1.-a),(13,a*(1.-b)),(14,a*b)))
 raise ValueError('unowned dragon anatomy: '+title)


def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid dragon clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,-.25),(.34,1.),(.63,.18),(1.,0.)))
  b=envelope(t,((0.,0.),(.26,-.20),(.46,1.),(.76,-.08),(1.,0.)));strength=2.0 if clip=='attack' else -1.7
 a*=strength;b*=strength;out=[]
 for name,parent,pivot in JOINTS[species]:
  side=-1 if name.endswith('left') else 1
  if name=='root':angle=(0.,0.,0.)
  elif name=='body':angle=(0.,.014*a,0.) if species=='dragonite' else (.018*b,.014*a,.035*a)
  elif name=='head':angle=(.012*a,.022*b,0.)
  elif name.startswith('ear_'):angle=(.018*b,side*.025*b,side*.075*a)
  elif name.startswith('antenna_'):angle=(.018*b,side*.035*b,side*.020*a)
  elif name.startswith('wing_'):
   beat=math.sin(math.tau*t*4) if clip=='idle' else a
   angle=(.025*b,side*.018*b,side*(.22*beat if clip=='idle' else .16*a))
  elif name.startswith('arm_'):angle=(.035*a if clip=='idle' else -.18*a,side*.025*b,side*.025*a)
  elif name.startswith('wrist_'):angle=(.050*b if clip=='idle' else -.12*b,side*.035*a,side*.025*b)
  elif name.startswith('leg_'):angle=(0.,side*.022*b,0.)
  elif name=='tail_base':angle=(0.,.035*b,0.)
  else:angle=(0.,-.045*a,0.)
  out.append(angle)
 return out


def pack_parts(model):
 result=[];groups={}
 for order,p in enumerate(model['primitives']):
  title=p['part'];prefix=title.split(' /')[0]
  group=('breast folds' if prefix=='Belly fold' else
         'head ink' if prefix in ('Eye socket','Pupil','Mouth','Nostril') else
         'head ivory' if prefix in ('Eye white','Glint') else
         prefix+' '+title.split()[-2] if prefix in ('Hand claw','Foot claw','Wing rib') else None)
  if group is None:result.append((p,{'source_order':order}));continue
  key=(group,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=prefix+' / complete '+group,positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3
  meta['source_parts'].append(dict(part=title,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
 return result


def export_dragon_family(species, model=None):
    if species not in SPECIES:raise ValueError('unknown dragon family')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed dragon family geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original grounded dragon family standard skin v1'
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
        'geometry_source':'tools/dragon_family_sculpt.py','animation_contract':'rotation-only source cues; grounded coil and stationary soles; shared original spline-ring weights for ventral ribbons; coherent faces and pearls; articulated heads, ears, antennae, wings, arms, wrists, legs and tails'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('dragon family exceeds existing model limits')
    return blob


@lru_cache(maxsize=3)
def canonical_blob(species):
    return export_dragon_family(species,authoring_model(species))


def decode_dragon_family(blob, species):
    if len(blob)>1024*1024 or blob!=export_dragon_family(species):
        raise ValueError('noncanonical dragon family GLB')
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




def read_dragon_family(path,species):
 with Path(path).open('rb') as f:return decode_dragon_family(f.read(1024*1024+1),species)
def export_dratini(model=None):return export_dragon_family('dratini',model)
def export_dragonair(model=None):return export_dragon_family('dragonair',model)
def export_dragonite(model=None):return export_dragon_family('dragonite',model)
def read_dratini(path):return read_dragon_family(path,'dratini')
def read_dragonair(path):return read_dragon_family(path,'dragonair')
def read_dragonite(path):return read_dragon_family(path,'dragonite')
