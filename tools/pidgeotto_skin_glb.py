#!/usr/bin/env python3
"""Canonical original Pidgeotto sculpture and full-body standard skin."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep
SPECIES=('pidgeotto',);HEIGHTS={'pidgeotto':1.0}
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=1)
def authoring_bundle():
 from pidgeotto_sculpt import sculpture
 return sculpture(with_transform=True)
def authoring_model(s):
 if s!='pidgeotto':raise ValueError('unknown Pidgeotto skin')
 return authoring_bundle()[0]

PARTS={'pidgeotto':('Bird body', 'Light breast', 'Bird head', 'Eye sclera.004', 'Eye iris.004', 'Eye catchlight.004', 'Eye sclera.005', 'Eye iris.005', 'Eye catchlight.005', 'Beak', 'Bird shin', 'Toe', 'Toe.001', 'Toe.002', 'Layered wing', 'Flight feather', 'Flight feather.001', 'Flight feather.002', 'Flight feather.003', 'Bird shin.001', 'Toe.003', 'Toe.004', 'Toe.005', 'Layered wing.001', 'Flight feather.004', 'Flight feather.005', 'Flight feather.006', 'Flight feather.007', 'Tail feather', 'Tail feather.001', 'Tail feather.002', 'Head crest', 'Head crest.001', 'Head crest.002')}
REST_DIGESTS={'pidgeotto':'49c12b14c5250c0b106ea983d6d9239166200e27a9f444d3bf7e43cebe2abcf1'}

def prepare_model(model):
 if geometry_digest(model)!=REST_DIGESTS['pidgeotto']:raise ValueError('unreviewed Pidgeotto anatomy')
 return model

def normalized(p):
 lo,k=authoring_bundle()[1]
 return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(p))
PIVOTS=(normalized((-.168,.491,-.038)),normalized((.168,.491,-.038)))
JOINTS={'pidgeotto':tuple((n,parent,(0.,0.,0.) if parent is None else normalized(p)) for n,parent,p in (
 ('root',None,(0.,0.,0.)),('body',0,(0.,.35,-.04)),('head',1,(0.,.63,.076)),
 ('wing_left',1,(-.168,.491,-.038)),('wrist_left',3,(-.244,.429,-.13)),
 ('wing_right',1,(.168,.491,-.038)),('wrist_right',5,(.244,.429,-.13)),
 ('leg_left',0,(-.112,.116,-.004)),('leg_right',0,(.112,.116,-.004)),
 ('tail_base',1,(0.,.262,-.225)),('tail_outer',9,(0.,.352,-.410)),
 ('tail_middle',9,(0.,.352,-.410)),('tail_center',9,(0.,.352,-.410)),
 ('crest',2,(0.,.772,.148))))}
CLIPS=(('idle',.8,65),('attack',1.,65),('hit',1.,65))
def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def face_mask_points(model):
 from pidgeotto_contours import components
 masks=set()
 for p in model['primitives']:
  if p['part']!='Light breast':continue
  for c in components(p):
   if min(c['positions'][1::3])>normalized((0.,.55,0.))[1]:
    masks.update(struct.unpack('<3f',packed(c['positions'][i:i+3])) for i in range(0,len(c['positions']),3))
 return masks
def shoulder_points(model):
 from pidgeotto_contours import components
 return {i:{struct.unpack('<3f',packed(p['positions'][j:j+3])) for p in components(model['primitives'][i])[:1] for j in range(0,len(p['positions']),3)} for i in (14,23)}
def skin_weights(s,n,p,mask_points,shoulders):
 lo,k=authoring_bundle()[1];x,y,z=p[0]/k,p[1]/k+lo,p[2]/k
 if n.startswith('Toe'):return rigid(0)
 if n=='Bird body':return rigid(1)
 if n=='Light breast':return rigid(2 if tuple(p) in mask_points else 1)
 if n=='Bird head' or n.startswith('Eye ') or n=='Beak':return rigid(2)
 if n.startswith(('Layered wing','Flight feather')):
  base=3 if x<0 else 5
  if n.startswith('Layered wing'):
   return rigid(1 if tuple(p) in shoulders[14 if x<0 else 23] else base)
  w=smoothstep(.12,.37,-z);return [base,base+1,0,0],[1.-w,w,0.,0.]
 if n.startswith('Bird shin'):
  # Authored digits stay fixed even though they share the pink shank material.
  a=smoothstep(.055,.205,y);bend=.32*smoothstep(.06,.11,y)*(1.-smoothstep(.14,.22,y));return [0,1,7 if x<0 else 8,0],[(1.-a)*(1.-bend),a*(1.-bend),bend,0.]
 if n.startswith('Tail feather'):
  tip=10 if n=='Tail feather' else 11 if n.endswith('.001') else 12;a=1.-smoothstep(-.32,-.19,z);b=1.-smoothstep(-.58,-.37,z);return [1,9,tip,0],[1.-a,a*(1.-b),a*b,0.]
 if n.startswith('Head crest'):
  w=1.-smoothstep(-.12,.14,z);return [2,13,0,0],[1.-w,w,0.,0.]
 raise ValueError('unowned Pidgeotto anatomy '+n)
def pose_angles(s,clip,t):
 if s!='pidgeotto' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Pidgeotto cue/time')
 if t in (0.,1.):return [(0.,0.,0.)]*14
 a=math.sin(math.tau*t)*(1.-math.cos(math.tau*t))*.769800358919501
 if clip=='idle':e=0.;flap=.025*a;head=.008*a;tail=.012*a;wrist=0.
 else:
  e=envelope(t,((0.,0.),(.18,1.),(.41,.65),(.69,.08),(1.,0.)))
  flap=.045*a+.10*e if clip=='attack' else -.15*e
  head=-.024*e if clip=='attack' else .028*e;tail=.030*e if clip=='attack' else -.020*e;wrist=.040*e if clip=='attack' else -.020*e
 return [(0.,0.,0.),(.009*e,0.,0.),(head,.010*a,0.),
         (0.,0.,flap),(.020*e,0.,wrist),(0.,0.,-flap),(.020*e,0.,-wrist),
         (0.,-.024*e,0.),(0.,.024*e,0.),(.010*a,tail,0.),
         (.012*a,tail,0.),(-.010*a,-tail,0.),(.014*a,tail*.5,0.),(.014*a+.010*e,0.,0.)]
def pack_parts(model):return [(p,{'source_order':i}) for i,p in enumerate(model['primitives'])]
def export_pidgeotto_skin(species, model=None):
    if species not in SPECIES:raise ValueError('unknown pidgeotto_skin')
    if model is None:return canonical_blob(species)
    model=prepare_model(model)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed pidgeotto_skin geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated pidgeotto_skin standard skin v1'
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
    mask_points=face_mask_points(model);shoulders=shoulder_points(model)
    for p,meta in parts:
        color = tuple(p['base_color'])
        if color not in colors:
            colors[color] = len(colors)
            g.doc['materials'].append({'name':species+'/paper_'+str(colors[color]),'alphaMode':'OPAQUE',
                'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.}})
        ids = [];weights = []
        for source in meta.get('source_parts',[dict(part=p['part'],vertex_start=0,vertex_count=len(p['positions'])//3)]):
            for i in range(source['vertex_start']*3,(source['vertex_start']+source['vertex_count'])*3,3):
                point = struct.unpack('<3f',packed(p['positions'][i:i+3]));jj,ww = skin_weights(species,source['part'],point,mask_points,shoulders);ww = quantized_weights(ww)
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
        'geometry_source':'tools/pidgeotto_sculpt.py:sculpture','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
        'animation_contract':'rotation-only idle/attack/hit source cues; anatomical folded-wing pivots and flat-ended 0.8-second standing idle; coherent head and cheek masks; articulated wrists, swept crest and three tail layers; fixed claws and blended supports'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('pidgeotto_skin exceeds existing model limits')
    return blob


def _decode_pidgeotto_skin(blob, species):
    if not 28<=len(blob)<=1024*1024 or blob[:4]!=b'glTF' or struct.unpack_from('<I',blob,8)[0]!=len(blob):
        raise ValueError('noncanonical pidgeotto_skin GLB')
    length=struct.unpack_from('<I',blob,12)[0]
    if struct.unpack_from('<I',blob,4)[0]!=2 or blob[16:20]!=b'JSON' or length%4 or length+28>len(blob):raise ValueError('invalid Pidgeotto GLB chunks')
    if blob[24+length:28+length]!=b'BIN\0' or struct.unpack_from('<I',blob,20+length)[0]!=len(blob)-28-length:raise ValueError('invalid Pidgeotto binary chunk')
    doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
    if len(doc['accessors'])>256 or len(doc['bufferViews'])>256 or len(doc['meshes'])!=1 or len(doc['meshes'][0]['primitives'])!=34:raise ValueError('Pidgeotto schema exceeds existing limits')
    def values(index,kind,component):
        a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
        if not isinstance(a,dict) or not isinstance(view,dict):raise ValueError('invalid Pidgeotto accessor')
        width=3 if kind=='VEC3' else 1;size=4 if component==5126 else 2;count=a['count'];offset=view.get('byteOffset',0)
        if type(count) is not int or not 1<=count<=(20000 if width==3 else 120000) or a['type']!=kind or a['componentType']!=component or a.get('byteOffset',0)!=0 or a.get('normalized',False) or 'sparse' in a:raise ValueError('unsupported Pidgeotto accessor')
        if type(offset) is not int or offset<0 or view['byteLength']!=count*width*size or offset+count*width*size>len(binary) or view.get('buffer')!=0 or view.get('byteStride') is not None:raise ValueError('out-of-bounds Pidgeotto accessor')
        return list(struct.unpack_from('<'+('f' if component==5126 else 'H')*(count*width),binary,offset))
    parts = []
    for p,meta in zip(doc['meshes'][0]['primitives'],doc['extras']['source_part_ranges']):
        positions = values(p['attributes']['POSITION'],'VEC3',5126);normals = values(p['attributes']['NORMAL'],'VEC3',5126)
        indices = values(p['indices'],'SCALAR',5123);color = doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
        sources = meta.get('source_parts',[dict(part=p['extras']['part'],order=meta.get('source_order'),
            vertex_start=0,vertex_count=len(positions)//3,index_start=0,index_count=len(indices))])
        for source in sources:
            a = source['vertex_start'];b = a+source['vertex_count']
            i = source['index_start'];j = i+source['index_count']
            parts.append((source['order'],dict(part=source['part'],positions=positions[3*a:3*b],
                normals=normals[3*a:3*b],indices=[k-a for k in indices[i:j]],base_color=color)))
    model=dict(name=species,version=1,coordinate_system=COORDINATES,
                primitives=[p for order,p in sorted(parts,key=lambda item:item[0])])
    if blob!=export_pidgeotto_skin(species,model):raise ValueError('noncanonical Pidgeotto skin GLB')
    return model

def decode_pidgeotto_skin(blob,species='pidgeotto'):
 try:return _decode_pidgeotto_skin(blob,species)
 except (KeyError,IndexError,TypeError,AttributeError,OverflowError,struct.error,RecursionError) as e:raise ValueError('malformed Pidgeotto standard skin') from e

@lru_cache(maxsize=2)
def canonical_blob(s):return export_pidgeotto_skin(s,authoring_model(s))
def read_pidgeotto_skin(path,s):
 with Path(path).open('rb') as f:return decode_pidgeotto_skin(f.read(1024*1024+1),s)
def read_pidgeotto(path):return read_pidgeotto_skin(path,'pidgeotto')
if __name__=='__main__':
 import argparse
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_pidgeotto_skin(s))
