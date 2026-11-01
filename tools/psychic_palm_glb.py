#!/usr/bin/env python3
"""Six grounded psychic eggs and a three-faced palm with individually bending fronds."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
import exeggcute_sculpt as eggs
import exeggutor_sculpt as palm
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep
SPECIES=('exeggcute','exeggutor');HEIGHTS={s:1.22 for s in SPECIES}
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=2)
def authoring_bundle(s):
 if s not in SPECIES:raise ValueError('unknown psychic palm species')
 return (eggs if s=='exeggcute' else palm).sculpture(rig_ready=True,with_transform=True)
def authoring_model(s):return authoring_bundle(s)[0]
def authoring_transform(s):return authoring_bundle(s)[1]
def normalized(s,p):
 lo,k=authoring_transform(s);return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(p))
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
REST_DIGESTS={'exeggcute': 'ade3f296f60113b7cc3d155aa94d30b0ba2782b42af8e9f50d5986688d63eb73', 'exeggutor': 'ec29d1bd63c9e9a5f88998f1e0ac8a846f51f2632680fc277bd31aeeb0126364'}
@lru_cache(maxsize=11)
def frond_path(j):
 _,centers=palm.frond(j,palm.ENDS[j]);return tuple((x,y-.36,z-.012) for x,y,z in centers)
def joints(s):
 if s=='exeggcute':raw=[('root',None,(0.,0.,0.))]+[(f'egg_{j}',0 if j<3 else j-2,c[:3]) for j,c in enumerate(eggs.EGGS)]
 else:
  raw=[('root',None,(0.,0.,0.)),('trunk_base',0,(0.,.20,-.01)),('crown_core',1,(0.,.55,-.02))]
  raw.extend((f'head_{j}',2,(cx*.55,cy-.36-.10,cz*.5)) for j,(cx,cy,cz) in enumerate(((-.245,.980,-.055),(-.025,.945,.167),(.205,1.012,.040))))
  raw.extend((('leg_left',0,(-.197,.137,.09)),('leg_right',0,(.197,.137,.09))))
  for j in range(11):
   base=len(raw);path=frond_path(j);raw.extend(((f'frond_base_{j}',2,path[0]),(f'frond_tip_{j}',base,path[12])))
 return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(s,p)) for n,parent,p in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def egg_owner(n):
 if n.startswith('Yolk /'):return 3
 return int(n.split()[-2] if n.startswith(('Eye white /','Pupil /','Lid /')) else n.split()[-1])
def palm_head(n):
 if n.startswith(('Tongue /','Fang /')):return 1
 if n.startswith('Slack tongue /'):return 0
 if n.startswith('Smile lining /'):return 2
 return int(n.split()[-2] if n.startswith(('Eye white /','Pupil /','Lid /')) else n.split()[-1])
def fraction_xy(point,path):
 best=(math.inf,0.);lengths=[math.dist(a[:2],b[:2]) for a,b in zip(path,path[1:])];offset=0.;total=sum(lengths)
 for a,b,length in zip(path,path[1:],lengths):
  d=[b[i]-a[i] for i in range(2)];t=max(0.,min(1.,sum((point[i]-a[i])*d[i] for i in range(2))/(length*length)));distance=sum((point[i]-a[i]-t*d[i])**2 for i in range(2));best=min(best,(distance,(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(s,n,point):
 lo,k=authoring_transform(s);raw=(point[0]/k,point[1]/k+lo,point[2]/k);x,y,z=raw
 if s=='exeggcute':
  j=egg_owner(n)
  if n.startswith('Egg /') and j<3:
   cx,cy,cz,rx,ry,rz=eggs.EGGS[j];a=smoothstep(.03,.26,(y-cy+ry)/(2*ry));return [0,j+1,0,0],[1.-a,a,0.,0.]
  return rigid(j+1)
 if n.startswith(('Foot /','Sole /','Claw /')):return rigid(0)
 if n.startswith(('Trunk /','Trunk crease /')):
  a=smoothstep(.16,.65,y);return [1,2,0,0],[1.-a,a,0.,0.]
 if n.startswith('Leg /'):
  j=6 if n.endswith('-1') else 7;a=smoothstep(.085,.235,y);b=.4*smoothstep(.08,.13,y)*(1.-smoothstep(.18,.24,y));return [0,1,j,0],[(1.-a)*(1.-b),a*(1.-b),b,0.]
 if n.startswith(('Head /','Eye white /','Pupil /','Lid /','Mouth /','Tongue /','Slack tongue /','Fang /','Smile lining /')):return rigid(3+palm_head(n))
 if n.startswith(('Frond /','Midrib /')):
  j=int(n.split()[-1]);t=fraction_xy(raw,frond_path(j));a=smoothstep(.02,.32,t);b=smoothstep(.40,.95,t);return [2,8+2*j,9+2*j,0],[1.-a,a*(1.-b),a*b,0.]
 raise ValueError('unowned coconut palm anatomy '+n)
def pose_angles(s,clip,t):
 if s not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid psychic palm cue/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[s])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);gain=1.
 else:a=envelope(t,((0.,0.),(.18,1.),(.42,.6),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));gain=2. if clip=='attack' else -1.4
 a*=gain;b*=gain
 if s=='exeggcute':return [(0.,0.,0.),(0.,.020*a,0.),(0.,-.017*b,0.),(0.,.022*a,0.),(.006*b,.012*a,0.),(.008*a,-.015*b,.004*a),(-.010*b,.018*a,-.004*b)]
 out=[(0.,0.,0.),(0.,.008*a,0.),(.006*b,.010*a,.004*b),(.010*b,.016*a,.005*b),(.008*a,-.018*b,.004*a),(-.010*b,.017*a,-.005*b),(0.,.012*a,0.),(0.,-.012*a,0.)]
 for j in range(11):
  sign=-1 if palm.ENDS[j][0]<0 else 1;f=.8+.05*j;out.extend(((.010*b*f,.010*a*sign,.012*b*sign),(.014*a*f,.012*b*sign,.015*a*sign)))
 return out
def joint_rotation(s,j,angles):return quaternion_xyz(*angles)
def pack_parts(model):
 result=[];groups={}
 for order,p in enumerate(model['primitives']):
  s=model['name'];n=p['part'];category=None;label=''
  if s=='exeggcute':category='Egg unit';label=str(egg_owner(n))
  elif n.startswith(('Head /','Eye white /','Pupil /','Lid /','Mouth /','Tongue /','Slack tongue /','Fang /','Smile lining /')):category='Head';label=str(palm_head(n))
  elif n.startswith(('Foot /','Sole /','Claw /')):category='Foot';label=n.split()[-2] if n.startswith('Claw /') else n.split()[-1]
  elif n.startswith('Frond /'):category='Frond bank'
  elif n.startswith('Midrib /'):category='Midrib bank'
  elif n.startswith('Trunk crease /'):category='Trunk creases'
  if category is None:result.append((p,{'source_order':order}));continue
  key=(category,label,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=category+' / coherent psychic anatomy '+label+' material '+str(len(groups)),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3;meta['source_parts'].append(dict(part=n,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices']+=[i+start for i in p['indices']]
 return result
def export_psychic_palm(species, model=None):
    if species not in SPECIES:raise ValueError('unknown psychic_palm')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed psychic_palm geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated psychic_palm standard skin v1'
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
            rotations = [v for i in range(count) for v in joint_rotation(species,joint,pose_angles(species,clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':species+'.'+clip,'channels':channels,'samplers':samplers,
                                   'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle'}})
    g.doc['extras'] = {'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
        'source_part_ranges':[meta for p,meta in parts],'packed_sha256':geometry_digest({'primitives':[p for p,meta in parts]}),
        'reference_span':HEIGHTS[species],'body_reference_height':HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
        'geometry_source':'tools/exeggcute_sculpt.py + tools/exeggutor_sculpt.py:sculpture(rig_ready=True)','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
        'animation_contract':'rotation-only source cues; grounded six-egg pile with coherent expressions and intact hollow shell/yolk; independent coconut faces; two-stage woody crown and eleven flexing fronds with matching midribs; planted complete palm feet, pads and claws'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('psychic_palm exceeds existing model limits')
    return blob


def decode_psychic_palm(blob, species):
    if len(blob)>1024*1024 or blob!=export_psychic_palm(species):
        raise ValueError('noncanonical psychic_palm GLB')
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
def canonical_blob(s):return export_psychic_palm(s,authoring_model(s))
def read_psychic_palm(path,s):
 with Path(path).open('rb') as f:return decode_psychic_palm(f.read(1024*1024+1),s)
def read_exeggcute(path):return read_psychic_palm(path,'exeggcute')
def read_exeggutor(path):return read_psychic_palm(path,'exeggutor')
if __name__=='__main__':
 import argparse
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_psychic_palm(s))
