#!/usr/bin/env python3
"""Canonical Tyranitar with paired armor, compact jaws and a low ring-bound tail."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from tyranitar_sculpt import sculpture,HEIGHT,TAIL,TAIL_RADII,arm_path,wrist,tail_spines
from bovine_sculpt import smooth
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep
SPECIES=('tyranitar',);HEIGHTS={'tyranitar':HEIGHT}
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()


@lru_cache(maxsize=1)
def authoring_model(species):
 if species!='tyranitar':raise ValueError('unknown armored dinosaur')
 return sculpture()
REST_DIGESTS={'tyranitar': '143d10ba16eaa62f9cab0c83a4b73bd310cee49c2932f042a43a86adb076101e'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
@lru_cache(maxsize=1)
def authoring_transform(species):
 raw=sculpture(False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,HEIGHT/(hi-lo)
def normalized(species,point):
 lo,scale=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*scale for i,v in enumerate(point))
def joints(species):
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.55,-.02)),('head',1,(0.,1.17,.015)),('jaw',2,(0.,1.10,-.015)),
      ('arm_left',1,arm_path(-1)[0][0]),('arm_right',1,arm_path(1)[0][0]),('forearm_left',4,arm_path(-1)[0][1]),('forearm_right',5,arm_path(1)[0][1]),('wrist_left',6,wrist(-1)),('wrist_right',7,wrist(1)),
      ('leg_left',0,(-.275,.32,-.005)),('leg_right',0,(.275,.32,-.005)),('tail_base',1,TAIL[0]),('tail_tip',12,TAIL[3])]
 return tuple((n,p,(0.,0.,0.) if p is None else normalized(species,v)) for n,p,v in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
@lru_cache(maxsize=1)
def tail_samples(species):
 points,radii=smooth(TAIL,TAIL_RADII,4);distance=[0.]
 for a,b in zip(points,points[1:]):distance.append(distance[-1]+math.dist(a,b))
 return points,radii,distance
def tail_distance(species,point):
 lo,scale=authoring_transform(species);point=tuple(v/scale+(lo if i==1 else 0.) for i,v in enumerate(point));points,radii,distance=tail_samples(species);scores=[]
 for i,center in enumerate(points):
  tangent=tuple(b-a for a,b in zip(points[max(0,i-1)],points[min(len(points)-1,i+1)]));length=math.sqrt(sum(v*v for v in tangent));tangent=tuple(v/length for v in tangent);delta=tuple(a-b for a,b in zip(point,center));axial=sum(a*b for a,b in zip(delta,tangent));radial=math.sqrt(max(0.,sum(v*v for v in delta)-axial*axial));scores.append((axial*axial+(radial-radii[i])**2,distance[i]))
 return min(scores)[1]

def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def influences(pairs):
 pairs=[(j,w) for j,w in pairs if w>1e-12]
 if len(pairs)==1:return rigid(pairs[0][0])
 return [j for j,w in pairs]+[0]*(4-len(pairs)),[w for j,w in pairs]+[0.]*(4-len(pairs))
def tail_influences(species,point):
 d=tail_distance(species,point);points,radii,distance=tail_samples(species);a=smoothstep(0.,.18,d);b=smoothstep(distance[8],distance[12],d);return influences(((1,1.-a),(12,a*(1.-b)),(13,a*b)))
def skin_weights(species,title,point):
 if title.startswith(('Foot /','Foot claw /')):return rigid(0)
 if title.startswith(('Body /','Belly /','Belly armor /','Mantle /','Torso vent /','Back armor /','Back vent /','Back spike /')):return rigid(1)
 if title.startswith(('Head /','Brow keel /','Mouth back /','Eye socket /','Eye white /','Iris /','Pupil /','Glint /','Nostril /','Crown spike /','Cheek plate /','Fang /')):return rigid(2)
 if title.startswith(('Jaw /','Tongue /','Lower tooth /')):return rigid(3)
 if title.startswith('Arm /'):return rigid(4 if title.endswith('-1') else 5)
 if title.startswith('Forearm /'):return rigid(6 if title.endswith('-1') else 7)
 if title.startswith(('Hand /','Hand claw /')):
  side=int(title.split()[-2] if title.startswith('Hand claw /') else title.split()[-1]);return rigid(8 if side<0 else 9)
 if title.startswith(('Thigh /','Shin /','Thigh vent /')):
  side=int(title.split()[-2] if title.startswith('Thigh vent /') else title.split()[-1])
  lo,scale=authoring_transform(species);a=smoothstep(.065,.20,point[1]/scale+lo);return influences(((0,1.-a),(10 if side<0 else 11,a)))
 if title.startswith('Tail /'):return tail_influences(species,point)
 if title.startswith('Tail hub /'):return rigid(13)
 if title.startswith('Tail spike /'):
  j=int(title.split()[-1]);root=tail_spines()[j][1];return tail_influences(species,normalized(species,root))
 raise ValueError('unowned Tyranitar anatomy: '+title)
def pose_angles(species,clip,t):
 if species!='tyranitar' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Tyranitar clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*14
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,-.25),(.34,1.),(.63,.18),(1.,0.)));b=envelope(t,((0.,0.),(.26,-.20),(.46,1.),(.76,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.7
 a*=strength;b*=strength;out=[]
 for name,parent,pivot in JOINTS[species]:
  side=-1 if name.endswith('left') else 1
  if name=='root':angle=(0.,0.,0.)
  elif name=='body':angle=(0.,.010*a,0.)
  elif name=='head':angle=(.016*a,.025*b,0.)
  elif name=='jaw':angle=(.018*abs(b) if clip=='idle' else .06*abs(b),0.,0.)
  elif name.startswith('arm_'):angle=(-.07*a,side*.02*b,side*.018*a)
  elif name.startswith('forearm_'):angle=(.06*b,side*.025*a,side*.018*b)
  elif name.startswith('wrist_'):angle=(-.08*b,side*.02*a,side*.018*b)
  elif name.startswith('leg_'):angle=(0.,side*.022*b,0.)
  elif name=='tail_base':angle=(0.,.04*b,0.)
  elif name=='tail_tip':angle=(0.,-.06*a,0.)
  else:raise ValueError(name)
  out.append(angle)
 return out

def pack_parts(model):
 result=[];groups={};species=model['name']
 for order,p in enumerate(model['primitives']):
  title=p['part'];prefix=title.split(' /')[0]
  group='head ink' if prefix in ('Eye socket','Pupil','Nostril','Mouth back') else 'head ivory' if prefix in ('Eye white','Glint','Fang') else 'head armor' if prefix in ('Crown spike','Cheek plate') else prefix if prefix in ('Iris','Belly armor','Torso vent','Back vent','Back spike','Tail spike','Hand claw','Foot claw','Lower tooth') else None
  if group is None:result.append((p,{'source_order':order}));continue
  owner=skin_weights(species,title,(0.,0.,0.))[0][0];side=title.split()[-2] if prefix=='Foot claw' else '';key=(group,owner,side,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=(prefix+' / complete '+group+' '+str(owner)+' '+side).rstrip()+' '+str(len(groups)),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3
  meta['source_parts'].append(dict(part=title,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
 return result

def export_tyranitar(species='tyranitar', model=None):
    if species not in SPECIES:raise ValueError('unknown Tyranitar')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed Tyranitar geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated Tyranitar standard skin v1'
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
        'geometry_source':'tools/tyranitar_sculpt.py','animation_contract':'rotation-only source cues; fixed soles; coherent fitted armor and frontal gaze; open toothed mandible; complete claws; sampled-ring heavy tail and rooted spines'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('Tyranitar exceeds existing model limits')
    return blob


def decode_tyranitar(blob, species):
    if len(blob)>1024*1024 or blob!=export_tyranitar(species):
        raise ValueError('noncanonical Tyranitar GLB')
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
def canonical_blob(species):return export_tyranitar(species,authoring_model(species))
def read_tyranitar(path):
 with Path(path).open('rb') as f:return decode_tyranitar(f.read(1024*1024+1),'tyranitar')
