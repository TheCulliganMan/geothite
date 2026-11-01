#!/usr/bin/env python3
"""Canonical Steelix skin preserving its exact angular sculpture and physical datums."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from steelix_sculpt import sculpture as source_model,anatomy,HEIGHT,CHAIN,HEAD_X,body_reference_span
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep
SPECIES=('steelix',);HEIGHTS={'steelix':HEIGHT}
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=1)
def authoring_model(s):
 if s!='steelix':raise ValueError('unknown steel serpent')
 return source_model()
PARTS={'steelix':tuple(p['part'] for p in authoring_model('steelix')['primitives'])}
REST_DIGESTS={'steelix':'87fe7010288e39368021ba4b84e9477f898cfc9e341d884f334cbaeeca40eceb'}
@lru_cache(maxsize=1)
def authoring_transform():
 ps=anatomy();lo=min(v for p in ps for v in p['positions'][1::3]);hi=max(v for p in ps for v in p['positions'][1::3]);return lo,HEIGHT/(hi-lo)
def normalized(point):
 lo,k=authoring_transform();return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(point))
def reference_points():return [(.69,.043,.22)]+[p[:3] for p in CHAIN]+[(HEAD_X,1.16,.21)]
RAW_JOINTS=(('root',None,(0.,0.,0.)),)+tuple(('segment_'+str(j),j-3,CHAIN[j-1][:3]) for j in range(3,10))+(('head',7,CHAIN[9][:3]),('jaw',8,(HEAD_X,.997,.02)))
JOINTS={'steelix':tuple((n,parent,(0.,0.,0.) if parent is None else normalized(p)) for n,parent,p in RAW_JOINTS)}
CLIPS=(('idle',4.,85),('attack',1.,65),('hit',1.,65))
def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def rock_owner(j):return max(0,j-2)
def skin_weights(s,n,p):
 if n.startswith('Segment /'):return rigid(rock_owner(int(n.split()[-1])))
 if n.startswith('Spike /'):return rigid(rock_owner(int(n.split()[-2])))
 if n.startswith('Tail /'):return rigid(0)
 if n.startswith(('Jaw /','Lower tooth /')):return rigid(9)
 if n.startswith(('Head /','Hinge /','Mouth back /','Upper tooth /','Socket /','Eye /','Iris /','Pupil /')):return rigid(8)
 if n.startswith('Joint /'):
  j=int(n.split()[-1]);owner=rock_owner(j);other=rock_owner(j+1)
  if owner==other:return rigid(owner)
  a=normalized(CHAIN[j][:3]);b=normalized(CHAIN[j+1][:3]);delta=[v-u for u,v in zip(a,b)];length=sum(v*v for v in delta);t=max(0.,min(1.,sum((v-u)*d for v,u,d in zip(p,a,delta))/length));w=smoothstep(.10,.90,t)
  return [owner,other,0,0],[1.-w,w,0.,0.]
 raise ValueError('unowned steel anatomy '+n)
def pose_angles(s,clip,t):
 if s not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid steel-chain pose')
 if t in (0.,1.):return [(0.,0.,0.)]*10
 out=[(0.,0.,0.)];breath=.5-.5*math.cos(math.tau*t)
 early=envelope(t,((0.,0.),(.18,1.),(.40,.70),(.67,.10),(1.,0.)));late=envelope(t,((0.,0.),(.27,1.),(.52,.65),(.78,-.08),(1.,0.)));sign=1. if clip=='attack' else -.65
 for k in range(7):
  phase=k*.13;wave=math.sin(math.tau*t-phase)+math.sin(phase)
  if clip=='idle':angle=(.0015*breath,.0025*wave,.002*wave)
  else:
   gain=.65+.035*k;angle=(sign*.0035*early*gain,sign*.005*late*gain,sign*.004*early*gain)
  out.append(angle)
 if clip=='idle':out.extend(((.004*breath,.011*wave,0.),(.018*breath,0.,0.)))
 else:out.extend(((sign*.017*late,sign*.014*late,sign*.005*late),(.070*early if clip=='attack' else -.012*late,0.,0.)))
 return out
def pack_parts(model):
 result=[];groups={}
 for order,p in enumerate(model['primitives']):
  n=p['part'];prefix=n.split(' /')[0];group='Socket' if prefix in ('Socket','Pupil','Mouth back') else prefix if prefix in ('Upper tooth','Lower tooth','Eye','Iris','Hinge','Spike') else None
  if group is None:result.append((p,{'source_order':order}));continue
  owner=skin_weights('steelix',n,(0.,0.,0.))[0][0];key=(group,owner,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=group+' / coherent steel carrier '+str(owner)+' '+str(len(groups)),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3;meta['source_parts'].append(dict(part=n,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
 return result

def export_steelix_model(species, model=None):
    if species not in SPECIES:raise ValueError('unknown steel chain')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed steel chain geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated steel chain standard skin v1'
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
        'body_centerline_length':body_reference_span(),'body_centerline_points':[normalized(p) for p in reference_points()],
        'geometry_preservation':'exact original source f32 anatomy, normals, colours and triangle order',
        'geometry_source':'tools/steelix_sculpt.py','animation_contract':'rotation-only source cues; fixed low three boulders and tail; rigid segmented armor and spikes; blended buried joints; coherent skull/gaze/upper teeth; independently hinged jaw and lower teeth'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('steel chain exceeds existing model limits')
    return blob


def decode_steelix(blob, species):
    if len(blob)>1024*1024 or blob!=export_steelix_model(species):
        raise ValueError('noncanonical snake GLB')
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
def canonical_blob(s):return export_steelix_model(s,authoring_model(s))
def export_steelix(model=None):return export_steelix_model('steelix',model)
def read_steelix(path):
 with Path(path).open('rb') as f:return decode_steelix(f.read(1024*1024+1),'steelix')
def main():
 import argparse
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'steelix.glb').write_bytes(export_steelix())
if __name__=='__main__':main()
