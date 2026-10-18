#!/usr/bin/env python3
"""Canonical Gloom and Vileplume botanical skins with independently hinged crowns."""
import hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from rafflesia_sculpt import sculpture,SPECIES,HEIGHTS,flower_point
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=2)
def authoring_model(s):return sculpture(s)
@lru_cache(maxsize=2)
def authoring_transform(s):
 p=sculpture(s,False)['primitives'];lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);return lo,HEIGHTS[s]/(hi-lo)
def normalized(s,p):
 lo,k=authoring_transform(s);return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(p))
def raw_point(s,p):
 lo,k=authoring_transform(s);return tuple(v/k+(lo if i==1 else 0.) for i,v in enumerate(p))
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
REST_DIGESTS={'gloom': 'ac149d788c449f8fda299e25d45cb7fd522d22d12b9c704bc37a32ff9b465025', 'vileplume': 'ef7abbebcd46edf5ed368ab5843546107efe9b41c070f2e3b6bcbac68462e16a'}
def joints(s):
 points=[('root',None,(0.,0.,0.)),('body',0,(0.,.24,0.)),('crown',1,(0.,.51,0.) if s=='gloom' else flower_point((0.,.59,0.))),('arm_left',1,(-.17,.32,0.)),('arm_right',1,(.17,.32,0.)),('leg_left',0,(-.10,.18,0.)),('leg_right',0,(.10,.18,0.))]
 for j in range(4 if s=='gloom' else 5):
  a=j*math.tau/(4 if s=='gloom' else 5)+(math.pi/4 if s=='gloom' else 0.);p=(.06*math.sin(a),.555,.06*math.cos(a)) if s=='gloom' else flower_point((.105*math.sin(a),.685,.105*math.cos(a)))
  points.append(('sepal_'+str(j) if s=='gloom' else 'petal_'+str(j),2,p))
 return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(s,p)) for n,parent,p in points)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.4,85),('attack',1.,65),('hit',1.,65))
def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def influences(pairs):
 pairs=[(j,w) for j,w in pairs if w>1e-12]
 if len(pairs)==1:return rigid(pairs[0][0])
 return [j for j,w in pairs]+[0]*(4-len(pairs)),[w for j,w in pairs]+[0.]*(4-len(pairs))
def skin_weights(s,n,p):
 x,y,z=raw_point(s,p)
 if n.startswith('Foot /'):return rigid(0)
 if n.startswith('Body /'):
  w=smoothstep(.10,.19,y);return influences(((0,1.-w),(1,w)))
 if n.startswith('Leg /'):
  w=smoothstep(.055,.14,y);return influences(((0,1.-w),(5 if n.endswith('-1') else 6,w)))
 if n.startswith('Arm /'):
  w=smoothstep(.18,.27,abs(x));return influences(((1,1.-w),(3 if n.endswith('-1') else 4,w)))
 if n.startswith(('Eye /','Lip /','Mouth /','Drool /')):return rigid(1)
 if n.startswith(('Bud /','Bud spot /','Crown /','Stem /','Rim /','Throat /')):return rigid(2)
 if n.startswith(('Sepal /','Petal /')):return rigid(7+int(n.split()[-1]))
 if n.startswith('Petal spot /'):return rigid(7+int(n.split()[-2]))
 raise ValueError('unowned rafflesia anatomy '+n)
def pose_angles(s,clip,t):
 if s not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid rafflesia pose')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[s])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);gain=1.
 else:a=envelope(t,((0.,0.),(.18,-.25),(.34,1.),(.63,.18),(1.,0.)));b=envelope(t,((0.,0.),(.26,-.20),(.46,1.),(.76,-.08),(1.,0.)));gain=2. if clip=='attack' else -1.5
 a*=gain;b*=gain;out=[(0.,0.,0.),(0.,.010*a,0.),(.025*b,0.,.015*a),(0.,-.015*b,.035*a),(0.,.015*b,-.035*a),(0.,-.020*b,0.),(0.,.020*b,0.)]
 for j in range(4 if s=='gloom' else 5):
  angle=j*math.tau/(4 if s=='gloom' else 5)+(math.pi/4 if s=='gloom' else 0.)
  out.append((math.cos(angle)*.035*b,0.,-math.sin(angle)*.035*b))
 return out

def pack_parts(model):
 result=[];groups={};s=model['name']
 for order,p in enumerate(model['primitives']):
  n=p['part'];prefix=n.split(' /')[0];group=prefix if prefix in ('Eye','Bud spot','Petal spot') else None
  if group is None:result.append((p,{'source_order':order}));continue
  owner=skin_weights(s,n,(0.,0.,0.))[0][0];key=(group,owner,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=prefix+' / coherent botanical carrier '+str(owner)+' '+str(len(groups)),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3;meta['source_parts'].append(dict(part=n,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
 return result

def export_rafflesia(species, model=None):
    if species not in SPECIES:raise ValueError('unknown rafflesia')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed rafflesia geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated rafflesia standard skin v1'
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
        'geometry_source':'tools/rafflesia_sculpt.py','animation_contract':'rotation-only source cues; planted root feet; coherent fitted face and drool; hinged four-lobed bud or hollow five-petal crown; fitted mottling shares each carrier'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('rafflesia exceeds existing model limits')
    return blob


def decode_rafflesia(blob, species):
    if len(blob)>1024*1024 or blob!=export_rafflesia(species):
        raise ValueError('noncanonical rafflesia GLB')
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
def canonical_blob(species):return export_rafflesia(species,authoring_model(species))
def read_rafflesia(path,species):
 with Path(path).open('rb') as f:return decode_rafflesia(f.read(1024*1024+1),species)
def read_gloom(path):return read_rafflesia(path,'gloom')
def read_vileplume(path):return read_rafflesia(path,'vileplume')
def main():
 import argparse
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_rafflesia(s))
if __name__=='__main__':main()
