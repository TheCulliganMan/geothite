#!/usr/bin/env python3
"""Coherent Pineco whorls and grounded Forretress shell/port standard skins."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from armored_bug_sculpt import sculpture,SPECIES,HEIGHT
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep
HEIGHTS={s:HEIGHT for s in SPECIES}
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def influences(pairs):
 pairs=[(j,w) for j,w in pairs if w>1e-12]
 if len(pairs)==1:return rigid(pairs[0][0])
 return [j for j,w in pairs]+[0]*(4-len(pairs)),[w for j,w in pairs]+[0.]*(4-len(pairs))
@lru_cache(maxsize=2)
def authoring_model(s):return sculpture(s)
REST_DIGESTS={'pineco': 'fb4c0dd8521ce539b3b024789b11b05641d5b9c88a12bff7fcc91a159b666d52', 'forretress': 'a55e58ca3cb62964d17ca334965d012468af3c82b80da9b18f4fdb1054229427'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
@lru_cache(maxsize=2)
def authoring_transform(s):
 raw=sculpture(s,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,HEIGHT/(hi-lo)
def normalized(s,p):
 lo,k=authoring_transform(s);return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(p))
def joints(s):
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.37 if s=='pineco' else .44,0.))]
 if s=='pineco':
  raw += [('whorl_low',1,(0.,.095,0.)),('whorl_middle',1,(0.,.23,0.)),('whorl_high',1,(0.,.415,0.)),('crown',1,(0.,.64,0.)),('stem',5,(0.,.68,-.015))]
 else:
  raw += [('upper_shell',0,(0.,.59,-.25))]
  for side in (-1,1):
   for j in range(2):raw.append((f'port_{side}_{j}',1,(side*.24,.455 if j==0 else .333,.12 if j==0 else -.13)))
 return tuple((n,p,(0.,0.,0.) if p is None else normalized(s,v)) for n,p,v in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.4,85),('attack',1.,65),('hit',1.,65))
def skin_weights(s,n,point):
 if s=='pineco':
  if n.startswith('Body /'):
   lo,k=authoring_transform(s);a=smoothstep(.04,.15,point[1]/k+lo);return influences(((0,1.-a),(1,a)))
  if n.startswith('Plate /'):return rigid(2+int(n.split()[-2]))
  if n.startswith('Crown /'):return rigid(5)
  if n.startswith('Stem /'):return rigid(6)
  if n.startswith('Eye /'):return rigid(1)
 else:
  if n.startswith(('Core /','Belt /','Eye /')):return rigid(1)
  if n.startswith('Shell /'):return rigid(2 if 'upper' in n else 0)
  if n.startswith('Chip /'):return rigid(2 if n.split()[-3]=='True' else 0)
  if n.startswith(('Port /','Port recess /')):
   side,j=map(int,n.split()[-2:]);return rigid(3+(0 if side<0 else 2)+j)
 raise ValueError('unowned armored bug part '+n)
def pose_angles(s,clip,t):
 if s not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid armored bug pose')
 if t in (0.,1.):return [(0.,0.,0.)]*7
 if clip=='idle':a=math.sin(math.tau*t);b=.5-.5*math.cos(math.tau*t);gain=1.
 else:a=envelope(t,((0.,0.),(.18,1.),(.40,.65),(.72,.1),(1.,0.)));b=envelope(t,((0.,0.),(.28,1.),(.52,.62),(.81,-.05),(1.,0.)));gain=2. if clip=='attack' else -1.4
 out=[]
 for n,parent,p in JOINTS[s]:
  if n=='root':v=(0.,0.,0.)
  elif n=='body':v=(0.,.010*a*gain,0.)
  elif n.startswith('whorl_'):v=(0.,(.014*a if n=='whorl_low' else -.018*b if n=='whorl_middle' else .020*a)*gain,0.)
  elif n=='crown':v=(0.,-.025*b*gain,0.)
  elif n=='stem':v=(.020*b*gain,0.,.030*a*gain)
  elif n=='upper_shell':v=(-.020*b if clip=='idle' else (-.055*a if clip=='attack' else .020*a),0.,0.)
  elif n.startswith('port_'):
   side,j=map(int,n.split('_')[-2:]);v=(.018*b*gain,side*.025*a*gain,side*(1 if j==0 else -1)*.020*b*gain)
  else:raise ValueError(n)
  out.append(v)
 return out

def pack_parts(model):
 result=[];groups={};s=model['name']
 for order,p in enumerate(model['primitives']):
  n=p['part'];prefix=n.split(' /')[0]
  # Whorl interfaces and front/back chip layers keep their exact source ranges.
  group=prefix if prefix in ('Plate','Crown','Chip','Eye') else None
  if group is None:result.append((p,{'source_order':order}));continue
  owner=skin_weights(s,n,(0.,0.,0.))[0][0];key=(group,owner,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=prefix+' / coherent armor carrier '+str(owner)+' '+str(len(groups)),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3;meta['source_parts'].append(dict(part=n,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
 return result

def export_armored_bug(species, model=None):
    if species not in SPECIES:raise ValueError('unknown armored bug')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed armored bug geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated armored bug standard skin v1'
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
        'geometry_source':'tools/armored_bug_sculpt.py','animation_contract':'rotation-only source cues; fixed pine core tip and lower rocky shell; coherent layered armor; independent plate whorls, crown and stem; hinged upper shell with supported chips; articulated hollow ports and deep bores'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('armored bug exceeds existing model limits')
    return blob


def decode_armored_bug(blob, species):
    if len(blob)>1024*1024 or blob!=export_armored_bug(species):
        raise ValueError('noncanonical armored bug GLB')
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
def canonical_blob(species):return export_armored_bug(species,authoring_model(species))
def read_armored_bug(path,species):
 with Path(path).open('rb') as f:return decode_armored_bug(f.read(1024*1024+1),species)
def read_pineco(path):return read_armored_bug(path,'pineco')
def read_forretress(path):return read_armored_bug(path,'forretress')
