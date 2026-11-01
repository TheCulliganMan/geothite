#!/usr/bin/env python3
"""Canonical Bellsprout, Weepinbell and Victreebel articulated botanical skins."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from pitcher_sculpt import sculpture,SPECIES,HEIGHT,STEM_PATH,HOOK_PATH,VINE_PATH,leaf_axis,root_path
from bovine_sculpt import smooth
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
@lru_cache(maxsize=3)
def authoring_model(s):return sculpture(s)
REST_DIGESTS={'bellsprout': 'dcacef4f2684505c937f76c4f5e7b66fc268da1b528d16a3723855efd618bf67', 'weepinbell': '2e9cb117e0c0ed3b15493d7dce8756bad76511f9f87af773cab74bd9a605a6f3', 'victreebel': '42140432e5eabe120833d48cb23b83490161ecb5044ffa90cd67e4a7171314c3'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
@lru_cache(maxsize=3)
def authoring_transform(s):
 raw=sculpture(s,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,HEIGHT/(hi-lo)
def normalized(s,p):
 lo,k=authoring_transform(s);return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(p))
def raw_point(s,p):
 lo,k=authoring_transform(s);return tuple(v/k+(lo if i==1 else 0.) for i,v in enumerate(p))
def midpoint(a,b,t=.55):return tuple(x+(y-x)*t for x,y in zip(a,b))
def joints(s):
 if s=='bellsprout':
  raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.12,-.004)),('upper_stem',1,STEM_PATH[3]),('head',2,STEM_PATH[-1]),('leaf_left',2,leaf_axis(s,-1)[0]),('leaf_right',2,leaf_axis(s,1)[0]),('leaf_tip_left',4,midpoint(*leaf_axis(s,-1))),('leaf_tip_right',5,midpoint(*leaf_axis(s,1))),('leg_left',0,root_path(-1)[0]),('leg_right',0,root_path(1)[0])]
 else:
  raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.40,-.01)),('mouth',1,(0.,.755,-.02) if s=='victreebel' else (0.,.545,.144)),('leaf_left',1,leaf_axis(s,-1)[0]),('leaf_right',1,leaf_axis(s,1)[0]),('leaf_tip_left',3,midpoint(*leaf_axis(s,-1))),('leaf_tip_right',4,midpoint(*leaf_axis(s,1)))]
  raw += [('hook',1,HOOK_PATH[0]),('hook_tip',7,HOOK_PATH[2])] if s=='weepinbell' else [('hood',2,(0.,.90,-.205)),('vine',1,VINE_PATH[0]),('vine_tip',8,VINE_PATH[2])]
 return tuple((n,p,(0.,0.,0.) if p is None else normalized(s,v)) for n,p,v in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.4,85),('attack',1.,65),('hit',1.,65))
@lru_cache(maxsize=3)
def tendril_samples(s):
 points=HOOK_PATH if s=='weepinbell' else VINE_PATH
 return smooth(points,[1.]*len(points))[0]
def skin_weights(s,n,point):
 p=raw_point(s,point);y=p[1]
 if n.startswith('Leaf /'):
  side=int(n.split(' / ')[1].split()[-1]);a,b=leaf_axis(s,side);dx=b[0]-a[0];dy=b[1]-a[1];t=((p[0]-a[0])*dx+(p[1]-a[1])*dy)/(dx*dx+dy*dy);w=smoothstep(.32,.82,t)
  base=(4 if side<0 else 5) if s=='bellsprout' else (3 if side<0 else 4);tip=base+2
  if t<.15:
   branch=smoothstep(.015,.15,t);return influences(((2 if s=='bellsprout' else 1,1.-branch),(base,branch)))
  return influences(((base,1.-w),(tip,w)))
 if s=='bellsprout':
  if n.startswith('Root /'):
   w=smoothstep(.032,.065,y);return influences(((0,1.-w),(8 if n.endswith('-1') else 9,w)))
  if n.startswith('Stem /'):
   if y<.18:w=smoothstep(.055,.18,y);return influences(((0,1.-w),(1,w)))
   w=smoothstep(.38,.60,y);return influences(((1,1.-w),(2,w)))
  return rigid(3)
 if n.startswith('Body /'):
  w=smoothstep(.04,.14,y);return influences(((0,1.-w),(1,w)))
 if n.startswith(('Eye /','Pigment /')):return rigid(1)
 if n.startswith(('Mouth /','Fang /')):return rigid(2)
 if n.startswith('Hood /'):return rigid(7)
 if n.startswith(('Hook /','Vine /')):
  samples=tendril_samples(s);i=min(range(len(samples)),key=lambda i:sum((samples[i][j]-p[j])**2 for j in range(3)));w=smoothstep(.38,.80,i/(len(samples)-1));j=7 if s=='weepinbell' else 8;return influences(((j,1.-w),(j+1,w)))
 raise ValueError('unowned pitcher anatomy '+n)
def pose_angles(s,clip,t):
 if s not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid pitcher pose')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[s])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);gain=1.
 else:a=envelope(t,((0.,0.),(.18,-.25),(.34,1.),(.63,.18),(1.,0.)));b=envelope(t,((0.,0.),(.26,-.20),(.46,1.),(.76,-.08),(1.,0.)));gain=2. if clip=='attack' else -1.5
 a*=gain;b*=gain;out=[]
 for n,parent,p in JOINTS[s]:
  side=-1 if n.endswith('left') else 1
  if n=='root':v=(0.,0.,0.)
  elif n=='body':v=(.018*a,.010*b,.014*b) if s=='bellsprout' else (0.,.012*a,0.)
  elif n=='upper_stem':v=(.025*b,-.015*a,.020*a)
  elif n=='head':v=(-.035*a,.035*b,.015*a)
  elif n=='mouth':v=(-.012*a,.012*b,0.) if s=='victreebel' else (-.025*a,0.,.015*b)
  elif n.startswith('leaf_tip_'):v=(side*.020*b,0.,side*.08*a)
  elif n.startswith('leaf_'):v=(side*.020*a,0.,side*.06*b)
  elif n.startswith('leg_'):v=(0.,side*.025*b,0.)
  elif n=='hood':v=(-.045*b,.025*a,0.)
  elif n in ('hook','vine'):v=(0.,.020*b,.025*a)
  elif n in ('hook_tip','vine_tip'):v=(.020*a,.025*a,-.060*b)
  else:raise ValueError(n)
  out.append(v)
 return out

def pack_parts(model):
 result=[];groups={};s=model['name']
 for order,p in enumerate(model['primitives']):
  n=p['part'];prefix=n.split(' /')[0];group=prefix if prefix in ('Eye','Pigment','Fang') else None
  if group is None:result.append((p,{'source_order':order}));continue
  owner=skin_weights(s,n,(0.,0.,0.))[0][0];key=(group,owner,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=prefix+' / coherent botanical carrier '+str(owner)+' '+str(len(groups)),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3;meta['source_parts'].append(dict(part=n,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
 return result

def export_pitcher(species, model=None):
    if species not in SPECIES:raise ValueError('unknown pitcher')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed pitcher geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated pitcher standard skin v1'
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
        'geometry_source':'tools/pitcher_sculpt.py','animation_contract':'rotation-only source cues; grounded roots/pitcher tips; flexible stalk and coherent hollow mouths; supported gaze and pigment; cupped leaves with bound veins; independent hood and curved tendrils'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('pitcher exceeds existing model limits')
    return blob


def decode_pitcher(blob, species):
    if len(blob)>1024*1024 or blob!=export_pitcher(species):
        raise ValueError('noncanonical pitcher GLB')
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



@lru_cache(maxsize=3)
def canonical_blob(species):return export_pitcher(species,authoring_model(species))
def read_pitcher(path,species):
 with Path(path).open('rb') as f:return decode_pitcher(f.read(1024*1024+1),species)
def read_bellsprout(path):return read_pitcher(path,'bellsprout')
def read_weepinbell(path):return read_pitcher(path,'weepinbell')
def read_victreebel(path):return read_pitcher(path,'victreebel')
