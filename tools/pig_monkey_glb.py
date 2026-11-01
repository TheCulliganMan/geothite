#!/usr/bin/env python3
"""Canonical angry monkey skins with planted paws, coherent fur and articulated fists."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from pig_monkey_sculpt import sculpture,SPECIES,HEIGHTS,normalized,authoring_transform,arm_path,leg_path,TAIL
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep
from bovine_sculpt import smooth

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=2)
def authoring_model(s):return sculpture(s)
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
REST_DIGESTS={'mankey': '15aa326ed611cbb9226add1d0fdda0e0cfe567d565e6bdc09998ed410856e4ce', 'primeape': '3bc3bdcfdadf6699cca07fa2a056a240dfad8af39cc863bc99a91efb2040970e'}
def joints(s):
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.38,0.)),('arm_left',1,arm_path(s,-1)[0]),('hand_left',2,arm_path(s,-1)[-1]),('arm_right',1,arm_path(s,1)[0]),('hand_right',4,arm_path(s,1)[-1]),('ear_left',1,(-.185,body_y(s)+.19,-.015)),('ear_right',1,(.185,body_y(s)+.19,-.015)),('leg_left',0,leg_path(s,-1)[1]),('leg_right',0,leg_path(s,1)[1])]
 if s=='mankey':raw.extend((('tail_base',1,TAIL[0]),('tail_tip',10,TAIL[3])))
 return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(s,p)) for n,parent,p in raw)
def body_y(s):return .55 if s=='mankey' else .57
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,85),('attack',1.,65),('hit',1.,65))
def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def fraction(point,path):
 best=(math.inf,0.);lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];offset=0.;total=sum(lengths)
 for a,b,length in zip(path,path[1:],lengths):
  delta=[v-u for u,v in zip(a,b)];t=max(0.,min(1.,sum((v-u)*d for v,u,d in zip(point,a,delta))/(length*length)));near=[v+t*d for v,d in zip(a,delta)];best=min(best,(math.dist(point,near),(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(s,n,point):
 lo,k=authoring_transform(s);raw=tuple(v/k+(lo if i==1 else 0.) for i,v in enumerate(point));x,y,z=raw
 if n.startswith(('Foot /','Toe /')):return rigid(0)
 if n.startswith(('Leg /','Ankle cuff /')):
  side=int(n.split()[-1]);a=smoothstep(.075,.31,y);bend=.42*smoothstep(.08,.15,y)*(1.-smoothstep(.23,.32,y));return [0,1,8 if side==-1 else 9,0],[(1.-a)*(1.-bend),a*(1.-bend),bend,0.]
 if n.startswith(('Arm /','Hand /','Knuckle /','Thumb /','Wrist cuff /')):
  side=int(n.split()[-2] if n.startswith('Knuckle /') else n.split()[-1]);owner=2 if side==-1 else 4
  if not n.startswith('Arm /'):return rigid(owner+1)
  pp,rr=smooth(arm_path(s,side),[.057,.053,.046],4);t=fraction(raw,pp);a=smoothstep(.02,.30,t);b=smoothstep(.35,.93,t);return [1,owner,owner+1,0],[1.-a,a*(1.-b),a*b,0.]
 if n.startswith(('Ear /','Ear inset /')):
  side=int(n.split()[-1]);w=smoothstep(body_y(s)+.20,body_y(s)+.30,y);return [1,6 if side==-1 else 7,0,0],[1.-w,w,0.,0.]
 if n.startswith('Tail /'):
  pp,rr=smooth(TAIL,[.031,.029,.026,.023,.017,.004],4);t=fraction(raw,pp);a=smoothstep(.02,.23,t);b=smoothstep(.44,.78,t);return [1,10,11,0],[1.-a,a*(1.-b),a*b,0.]
 if n.startswith(('Body /','Fur /','Belly /','Eye socket /','Eye /','Pupil /','Snout /','Nostril /')):return rigid(1)
 raise ValueError('unowned pig-monkey anatomy '+n)
def pose_angles(s,clip,t):
 if s not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid pig-monkey cue/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[s])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.40)+math.sin(.40);gain=1.
 else:a=envelope(t,((0.,0.),(.18,1.),(.41,.65),(.69,.08),(1.,0.)));b=envelope(t,((0.,0.),(.30,1.),(.53,.55),(.80,-.06),(1.,0.)));gain=2. if clip=='attack' else -1.4
 a*=gain;b*=gain
 out=[(0.,0.,0.),(.006*a,.014*b,0.),(.025*a,.015*b,-.024*a),(.025*b,.020*a,-.012*b),(.025*a,-.015*b,.024*a),(.025*b,-.020*a,.012*b),(.012*b,0.,-.014*a),(.012*b,0.,.014*a),(0.,.015*a,0.),(0.,-.015*a,0.)]
 if s=='mankey':out.extend(((.018*b,.035*a,0.),(.025*a,-.045*b,0.)))
 return out

def pack_parts(model):
 result=[];groups={}
 for order,p in enumerate(model['primitives']):
  n=p['part'];category='Hand' if n.startswith(('Hand /','Knuckle /','Thumb /')) else 'Foot' if n.startswith(('Foot /','Toe /')) else None
  if category is None:result.append((p,{'source_order':order}));continue
  side=int(n.split()[-2] if n.startswith(('Knuckle /','Toe /')) else n.split()[-1]);key=(category,side,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=category+' / coherent monkey digits '+str(side),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3;meta['source_parts'].append(dict(part=n,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices']+=[i+start for i in p['indices']]
 return result

def export_pig_monkey(species, model=None):
    if species not in SPECIES:raise ValueError('unknown pig monkey')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed pig monkey geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated pig monkey standard skin v1'
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
        'geometry_source':'tools/pig_monkey_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
        'animation_contract':'rotation-only source cues; fixed paws and toes; blended supporting shins; coherent pelt, angry face and nostrils; articulated forelimbs, clenched fists, ears and long Mankey tail'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('pig monkey exceeds existing model limits')
    return blob


def decode_pig_monkey(blob, species):
    if len(blob)>1024*1024 or blob!=export_pig_monkey(species):
        raise ValueError('noncanonical pig monkey GLB')
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
def canonical_blob(s):return export_pig_monkey(s,authoring_model(s))
def read_pig_monkey(path,s):
 with Path(path).open('rb') as f:return decode_pig_monkey(f.read(1024*1024+1),s)
def read_mankey(path):return read_pig_monkey(path,'mankey')
def read_primeape(path):return read_pig_monkey(path,'primeape')
if __name__=='__main__':
 import argparse
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_pig_monkey(s))
