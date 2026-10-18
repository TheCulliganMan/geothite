#!/usr/bin/env python3
"""Canonical Jynx skin: grounded folded gown, coherent gaze, hair and open hands."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from ice_dancers_sculpt import sculpture as source_model,HEIGHTS,jynx_arm_path,jynx_digit_paths
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep
from bovine_sculpt import smooth
SPECIES=('jynx',);HEIGHT=HEIGHTS['jynx']
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=1)
def authoring_model(s='jynx'):
 if s!='jynx':raise ValueError('unknown gown dancer')
 return source_model(s)
PARTS=tuple(p['part'] for p in authoring_model()['primitives'])
REST_DIGEST='d1a7fc03473ac5f3eb42d0f738ec5926ab96bcdeddbc5d0d0017e5a9a294ecf0'
@lru_cache(maxsize=1)
def authoring_transform():
 raw=source_model('jynx',normalize=False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,HEIGHT/(hi-lo)
def normalized(point):
 lo,k=authoring_transform();return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(point))
RAW_JOINTS=[('root',None,(0.,0.,0.)),('torso',0,(0.,.68,-.015)),('neck',1,(0.,.90,.018)),('head',2,(0.,1.02,.044)),('hair_left',3,(-.132,1.346,.023)),('hair_right',3,(.132,1.346,.023))]
for side,label in ((-1,'left'),(1,'right')):
 path=jynx_arm_path(side);start=len(RAW_JOINTS);RAW_JOINTS.extend(((f'arm_{label}',1,path[0]),(f'elbow_{label}',start,path[1]),(f'hand_{label}',start+1,path[2])))
for side,label,wrist in ((-1,'left',8),(1,'right',11)):
 for j,(path,radii) in enumerate(jynx_digit_paths(side)):RAW_JOINTS.append((f'finger_{label}_{j}',wrist,path[0]))
JOINTS=tuple((name,parent,(0.,0.,0.) if parent is None else normalized(p)) for name,parent,p in RAW_JOINTS)
CLIPS=(('idle',3.6,85),('attack',1.,65),('hit',1.,65))
def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def fraction(point,path):
 best=(math.inf,0.);lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];offset=0.;total=sum(lengths)
 for a,b,length in zip(path,path[1:],lengths):
  delta=[v-u for u,v in zip(a,b)];t=max(0.,min(1.,sum((v-u)*d for v,u,d in zip(point,a,delta))/(length*length)));near=[v+t*d for v,d in zip(a,delta)];best=min(best,(math.dist(point,near),(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(name,point):
 lo,k=authoring_transform();raw=tuple(v/k+(lo if i==1 else 0.) for i,v in enumerate(point));x,y,z=raw
 if name.startswith('Gown /'):
  w=smoothstep(.20,.68,y);return [0,1,0,0],[1.-w,w,0.,0.]
 if name.startswith('Neck /'):
  a=smoothstep(.88,.96,y);b=smoothstep(.98,1.08,y);return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
 if name.startswith(('Chest trim /','Chest bridge /','Bodice /')):return rigid(1)
 if name.startswith('Sleeve /'):
  side=int(name.split()[-1]);start=6 if side==-1 else 9;pp,rr=smooth(jynx_arm_path(side),[.074,.078,.056],4);t=fraction(raw,pp);a=smoothstep(.02,.24,t);b=smoothstep(.32,.68,t);c=smoothstep(.72,.97,t)
  return [1,start,start+1,start+2],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
 if name.startswith('Palm /'):return rigid(8 if name.endswith('-1') else 11)
 if name.startswith('Finger /'):
  side,j=map(int,name.split()[-2:]);wrist=8 if side==-1 else 11;owner=12+j if side==-1 else 17+j;path,radii=jynx_digit_paths(side)[j];pp,rr=smooth(path,radii,4);w=smoothstep(.03,.40,fraction(raw,pp));return [wrist,owner,0,0],[1.-w,w,0.,0.]
 if name.startswith(('Hair curtain /','Hair strand /','Hair tip /')):
  side=int(name.split()[-2] if name.startswith('Hair tip /') else name.split()[-1]);w=1.-smoothstep(1.18,1.34,y);return [3,4 if side==-1 else 5,0,0],[1.-w,w,0.,0.]
 if name.startswith(('Head /','Eye socket /','Eye /','Pupil /','Lips /','Mouth /','Hair','Back hair strand /')):return rigid(3)
 raise ValueError('unowned adult dancer anatomy '+name)
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid dancer cue/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);gain=1.
 else:a=envelope(t,((0.,0.),(.18,1.),(.41,.65),(.69,.08),(1.,0.)));b=envelope(t,((0.,0.),(.30,1.),(.53,.55),(.80,-.06),(1.,0.)));gain=1.7 if clip=='attack' else -1.2
 a*=gain;b*=gain
 out=[(0.,0.,0.),(0.,.009*a,.003*b),(0.,.009*a,0.),(.007*a,.018*b,0.),(.015*b,.014*b,-.012*b),(.015*b,-.014*b,.012*b)]
 for side in (-1,1):out.extend(((.012*a,side*.016*b,side*.030*a),(.020*b,side*.015*a,side*.022*b),(.018*a,side*.020*b,side*.012*a)))
 for side in (-1,1):
  for j in range(5):out.append((.018*a*(.7+.07*j),side*.008*b,side*.009*a))
 return out
def pack_parts(model):
 result=[];groups={}
 for order,p in enumerate(model['primitives']):
  name=p['part']
  if not name.startswith(('Palm /','Finger /')):result.append((p,{'source_order':order}));continue
  side=int(name.split()[-2] if name.startswith('Finger /') else name.split()[-1]);key=(side,tuple(p['base_color']))
  if key not in groups:
   q=dict(part='Hand / coherent five-digit dancer palm '+str(side),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3;meta['source_parts'].append(dict(part=name,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices']+=[i+start for i in p['indices']]
 return result

def export_jynx_model(species, model=None):
    if species not in SPECIES:raise ValueError('unknown dancer')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGEST
        or [p['part'] for p in model['primitives']]!=list(PARTS)):
        raise ValueError('unreviewed dancer geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated dancer standard skin v1'
    nodes = g.doc['nodes'];inverse = [];js = JOINTS
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
                point = struct.unpack('<3f',packed(p['positions'][i:i+3]));jj,ww = skin_weights(source['part'],point);ww = quantized_weights(ww)
                ids += [j if w else 0 for j,w in zip(jj,ww)];weights += ww
        primitives.append({'attributes':{'POSITION':g.accessor(p['positions'],'VEC3',target=34962,bounds=True),
            'NORMAL':accessor(p['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
            'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},'indices':accessor(p['indices'],'SCALAR',5123,34963),
            'material':colors[color],'mode':4,'extras':{'part':p['part']}})
    g.doc['meshes'] = [{'name':species,'primitives':primitives}]
    for clip,duration,count in CLIPS:
        times = g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels = [];samplers = []
        for joint in range(1,len(js)):
            rotations = [v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':species+'.'+clip,'channels':channels,'samplers':samplers,
                                   'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle'}})
    g.doc['extras'] = {'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGEST,
        'source_part_ranges':[meta for p,meta in parts],'packed_sha256':geometry_digest({'primitives':[p for p,meta in parts]}),
        'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
        'geometry_source':'tools/ice_dancers_sculpt.py','animation_contract':'rotation-only source cues; fixed gown hem; blended waist and throat; coherent fitted gaze and hollow kiss rim; independent hair curtains; three-joint arms and individually rooted fingers'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('dancer exceeds existing model limits')
    return blob


def decode_jynx(blob, species):
    if len(blob)>1024*1024 or blob!=export_jynx_model(species):
        raise ValueError('noncanonical dancer GLB')
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
def canonical_blob(s='jynx'):return export_jynx_model(s,authoring_model(s))
def export_jynx(model=None):return export_jynx_model('jynx',model)
def read_jynx(path):
 with Path(path).open('rb') as f:return decode_jynx(f.read(1024*1024+1),'jynx')
if __name__=='__main__':
 import argparse
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_jynx())
