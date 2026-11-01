#!/usr/bin/env python3
"""Canonical Larvitar and Pupitar with grounded feet/keel and coherent armored faces."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from rock_pupa_sculpt import sculpture,SPECIES,HEIGHTS,arm_path,LARVA_TAIL,LARVA_FAN
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()


@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'larvitar': '427f80e604f64b3939804ea5f70602622dbeb7d3c322beaada67965d9d3e1111', 'pupitar': '1905fcbbb5bdf9066860f8fb4d6b5c2a3646da2fd4098cd3fc75a43c59d1e6b7'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,HEIGHTS[species]/(hi-lo)
def normalized(species,point):
 lo,scale=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*scale for i,v in enumerate(point))
def joints(species):
 if species=='larvitar':
  raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.36,-.015)),('head',1,(0.,.65,.020)),('arm_left',1,arm_path(-1)[0][0]),('arm_right',1,arm_path(1)[0][0]),('wrist_left',3,arm_path(-1)[0][-1]),('wrist_right',4,arm_path(1)[0][-1]),('leg_left',0,(-.15,.125,.005)),('leg_right',0,(.15,.125,.005)),('tail_base',1,LARVA_TAIL[0]),('tail_tip',9,LARVA_FAN)]
 else:raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.42,-.02)),('head',1,(0.,.70,0.)),('tail_fold',0,(0.,.12,-.20))]
 return tuple((n,p,(0.,0.,0.) if p is None else normalized(species,v)) for n,p,v in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def influences(pairs):
 pairs=[(j,w) for j,w in pairs if w>1e-12]
 if len(pairs)==1:return rigid(pairs[0][0])
 return [j for j,w in pairs]+[0]*(4-len(pairs)),[w for j,w in pairs]+[0.]*(4-len(pairs))
def skin_weights(species,title,point):
 if title.startswith(('Head /','Eye socket /','Eye white /','Iris /','Pupil /','Glint /','Lip /','Cheek tear /','Nostril /','Horn /','Crown spike /')):return rigid(2)
 if species=='pupitar':
  if title.startswith(('Shell seam /','Back seam /')):return rigid(1)
  if title.startswith('Side spike /'):return rigid(2 if title.endswith(' 2') else 1)
  lo,scale=authoring_transform(species);y=point[1]/scale+lo
  if title.startswith('Body /'):a=smoothstep(.035,.16,y);return influences(((0,1.-a),(1,a)))
  if title.startswith('Tail keel /'):a=smoothstep(.035,.10,y);return influences(((0,1.-a),(3,a)))
 else:
  if title.startswith(('Body /','Belly /','Body vent /','Back vent /')):return rigid(1)
  if title.startswith(('Foot /','Toe /')):return rigid(0)
  if title.startswith('Arm /'):
   from bovine_sculpt import smooth
   from rhino_glb import path_fraction
   side=-1 if title.endswith('-1') else 1;lo,scale=authoring_transform(species)
   raw=tuple(v/scale+(lo if k==1 else 0.) for k,v in enumerate(point))
   path=smooth(*arm_path(side))[0];t=path_fraction(raw,path);blend=smoothstep(.45,.95,t)
   return influences(((3 if side<0 else 4,1.-blend),(5 if side<0 else 6,blend)))
  if title.startswith(('Hand /','Finger /')):
   side=int(title.split()[-2] if title.startswith('Finger /') else title.split()[-1]);return rigid(5 if side<0 else 6)
  if title.startswith('Thigh /'):
   lo,scale=authoring_transform(species);a=smoothstep(.04,.14,point[1]/scale+lo);return influences(((0,1.-a),(7 if title.endswith('-1') else 8,a)))
  if title.startswith('Tail /'):return rigid(9)
  if title.startswith(('Tail prong /','Tail hub /')):return rigid(10)
 raise ValueError('unowned rock larva/pupa anatomy: '+title)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid rock larva/pupa clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,-.25),(.34,1.),(.63,.18),(1.,0.)));b=envelope(t,((0.,0.),(.26,-.20),(.46,1.),(.76,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.7
 a*=strength;b*=strength;out=[]
 for name,parent,pivot in JOINTS[species]:
  side=-1 if name.endswith('left') else 1
  if name=='root':angle=(0.,0.,0.)
  elif name=='body':angle=(0.,.010*a,0.)
  elif name=='head':angle=(.010*a,.022*b,0.) if species=='larvitar' else (.006*a,.014*b,0.)
  elif name.startswith('arm_'):angle=(-.07*a,side*.025*b,side*.02*a)
  elif name.startswith('wrist_'):angle=(-.05*b,side*.02*a,side*.018*b)
  elif name.startswith('leg_'):angle=(0.,side*.023*b,0.)
  elif name=='tail_base':angle=(0.,.03*b,0.)
  elif name=='tail_tip':angle=(0.,-.05*a,0.)
  elif name=='tail_fold':angle=(0.,.03*b,0.)
  else:raise ValueError(name)
  out.append(angle)
 return out

def pack_parts(model):
 result=[];groups={};species=model['name']
 for order,p in enumerate(model['primitives']):
  title=p['part'];prefix=title.split(' /')[0]
  group='head ink' if prefix in ('Eye socket','Pupil','Lip','Cheek tear','Nostril') else 'head ivory' if prefix in ('Eye white','Glint') else 'head armor' if prefix in ('Crown spike','Side spike') else 'shell seam' if prefix in ('Shell seam','Back seam') else 'hand' if prefix in ('Hand','Finger') else prefix if prefix in ('Iris','Toe','Tail prong','Body vent') else None
  if group is None:result.append((p,{'source_order':order}));continue
  owner=skin_weights(species,title,(0.,0.,0.))[0][0];side=title.split()[-2] if prefix=='Toe' else '';key=(group,owner,side,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=(prefix+' / complete '+group+' '+str(owner)+' '+side).rstrip()+' '+str(len(groups)),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3;meta['source_parts'].append(dict(part=title,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
 return result

def export_rock_pupa(species, model=None):
    if species not in SPECIES:raise ValueError('unknown rock larva/pupa')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed rock larva/pupa geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated rock larva/pupa standard skin v1'
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
        'geometry_source':'tools/rock_pupa_sculpt.py','animation_contract':('rotation-only source cues; fixed soles; coherent deep head and fitted gaze; tapered larva arms with spatial wrist blends; short thick tail with a six-point radial fan and solid hub; front and rear flank markings' if species=='larvitar' else 'rotation-only source cues; fixed soles/keel; fitted rigid frontal gaze; one-horn larva with complete blunt digits and three-pronged tail; sealed pupa with restrained shell motion and coherent front/back seams')}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('rock larva/pupa exceeds existing model limits')
    return blob


def decode_rock_pupa(blob, species):
    if len(blob)>1024*1024 or blob!=export_rock_pupa(species):
        raise ValueError('noncanonical rock larva/pupa GLB')
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
def canonical_blob(species):return export_rock_pupa(species,authoring_model(species))
def read_rock_pupa(path,species):
 with Path(path).open('rb') as f:return decode_rock_pupa(f.read(1024*1024+1),species)
def read_larvitar(path):return read_rock_pupa(path,'larvitar')
def read_pupitar(path):return read_rock_pupa(path,'pupitar')
