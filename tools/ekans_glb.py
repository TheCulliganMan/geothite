#!/usr/bin/env python3
"""Canonical Ekans skin with grounded coil, continuous collar and raised tail."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from ekans_sculpt import sculpture,HEIGHT,PATH,RADII,STEPS,spine_samples,authoring_transform
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep
SPECIES=('ekans',);HEIGHTS={'ekans':HEIGHT}
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=1)
def authoring_model(s):return sculpture(s)
PARTS={'ekans':tuple(p['part'] for p in sculpture()['primitives'])}
REST_DIGESTS={'ekans':'eaf2ebad196f288b8c3827336f2ca7c90dd56070ced67699b45e13427dfbb6e4'}
def normalized(p):
 lo,k=authoring_transform();return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(p))
RAW_JOINTS=(('root',None,(0.,0.,0.)),('body',0,PATH[2]),('head',1,PATH[0]),('tail_base',0,PATH[9]),('tail_tip',3,PATH[10]))
JOINTS={'ekans':tuple((n,parent,(0.,0.,0.) if parent is None else normalized(p)) for n,parent,p in RAW_JOINTS)}
CLIPS=(('idle',3.4,85),('attack',1.,65),('hit',1.,65))
def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def influences(pairs):
 pairs=[(j,w) for j,w in pairs if w>1e-12]
 if len(pairs)==1:return rigid(pairs[0][0])
 return [j for j,w in pairs]+[0]*(4-len(pairs)),[w for j,w in pairs]+[0.]*(4-len(pairs))
@lru_cache(maxsize=1)
def samples():
 points,radii=spine_samples();dist=[0.]
 for a,b in zip(points,points[1:]):dist.append(dist[-1]+math.dist(a,b))
 return points,radii,dist
@lru_cache(maxsize=1)
def ring_frames():
 ps,rs,ds=samples();out=[]
 for i,(p,r,d) in enumerate(zip(ps,rs,ds)):
  a=ps[max(0,i-1)];b=ps[min(len(ps)-1,i+1)];v=tuple(y-x for x,y in zip(a,b));length=math.sqrt(sum(x*x for x in v));out.append((p,tuple(x/length for x in v),r,d))
 return out
@lru_cache(maxsize=24000)
def spine_distance(point):
 lo,k=authoring_transform();p=tuple(v/k+(lo if i==1 else 0.) for i,v in enumerate(point));best=(math.inf,0.)
 for center,tangent,radius,d in ring_frames():
  delta=tuple(v-c for v,c in zip(p,center));axial=sum(v*t for v,t in zip(delta,tangent));radial=math.sqrt(max(0.,sum(v*v for v in delta)-axial*axial));best=min(best,(axial*axial+(radial-radius)**2,d))
 return best[1]
def skin_weights(s,n,p):
 if n.startswith(('Head /','Eye /','Pupil /','Glint /','Nostril /','Mouth /')):return rigid(2)
 if n.startswith('Rattle /'):return rigid(4)
 if n.startswith(('Body /','Belly /','Collar /')):
  d=spine_distance(tuple(p));ps,rs,ds=samples()
  if d<ds[3*STEPS]:
   a=smoothstep(.08,.21,d);b=smoothstep(.39,.62,d)
   return influences(((2,1.-a),(1,a*(1.-b)),(0,a*b)))
  a=smoothstep(ds[9*STEPS]-.12,ds[9*STEPS]+.06,d)
  b=smoothstep(ds[10*STEPS]-.10,ds[10*STEPS]+.035,d)
  return influences(((0,1.-a),(3,a*(1.-b)),(4,a*b)))
 raise ValueError('unowned snake anatomy '+n)
def pose_angles(s,clip,t):
 if s not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid snake pose')
 if t in (0.,1.):return [(0.,0.,0.)]*5
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.35)+math.sin(.35)
 else:
  a=envelope(t,((0.,0.),(.16,-.25),(.34,1.),(.63,.15),(1.,0.)))
  b=envelope(t,((0.,0.),(.25,-.2),(.46,1.),(.78,-.08),(1.,0.)))
  a*=2. if clip=='attack' else -1.6;b*=2. if clip=='attack' else -1.6
 return [(0.,0.,0.),(.012*b,.014*a,.026*a),(.022*a,.038*b,-.018*a),(0.,.05*b,0.),(0.,-.065*a,0.)]
def pack_parts(model):return [(p,{'source_order':i}) for i,p in enumerate(model['primitives'])]

def export_ekans_model(species, model=None):
    if species not in SPECIES:raise ValueError('unknown coiled snake')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed coiled snake geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated coiled snake standard skin v1'
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
        'geometry_source':'tools/ekans_sculpt.py','animation_contract':'rotation-only source cues; fixed grounded coil; coherent fitted face; continuous neck collar and belly weights; independent raised tail base and tip'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('coiled snake exceeds existing model limits')
    return blob


def decode_ekans(blob, species):
    if len(blob)>1024*1024 or blob!=export_ekans_model(species):
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
def canonical_blob(s):return export_ekans_model(s,authoring_model(s))
def export_ekans(model=None):return export_ekans_model('ekans',model)
def read_ekans(path):
 with Path(path).open('rb') as f:return decode_ekans(f.read(1024*1024+1),'ekans')
def main():
 import argparse
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'ekans.glb').write_bytes(export_ekans())
if __name__=='__main__':main()
