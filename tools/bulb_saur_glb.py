#!/usr/bin/env python3
"""Original evolved seed saurs: grounded quadrupeds and independently rooted foliage."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
import bulb_saur_sculpt as author
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep
SPECIES=author.SPECIES;HEIGHTS=author.HEIGHTS
REST_DIGESTS={'ivysaur':'4195e2b91009093f48f1ca35847d42e9058b0147cc4b13209b9da41fca21bd7f','venusaur':'ba3e09a51c8df2b1d3e4a30ea99c955679b8b1d7fdb630f349bd7645b471230b'}
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=2)
def authoring_bundle(s):return author.sculpture(s,with_transform=True)
def authoring_model(s):return authoring_bundle(s)[0]
def authoring_transform(s):return authoring_bundle(s)[1]
def normalized(s,p):
 lo,k=authoring_transform(s);return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(p))
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
def blade_frame(s,j,t,*,petal=False):
 angle=math.tau*j/(5 if petal else 6)+(.16 if petal else .22)
 length=.61 if petal or s=='venusaur' else .43;base=.99 if petal else .59 if s=='venusaur' else .50
 along=.035+length*t;y=base+(.090 if petal else .108)*math.sin(math.pi*t)-(.045 if petal else .10)*t*t
 return (math.sin(angle)*along,y,-.16+math.cos(angle)*along)
def joints(s):
 mature=s=='venusaur';front=.335*author.MATURE_BODY_WIDTH if mature else .235;hind=.32*author.MATURE_BODY_WIDTH if mature else .225
 raw=[('root',None,(0.,0.,0.)),('torso',0,(0.,.265,-.19 if mature else -.095)),('head',1,(0.,.39,.24)),
      ('foreleg_left',1,(-front,.29,.205)),('foreleg_right',1,(front,.29,.205)),
      ('hindleg_left',1,(-hind,.28,-.445 if mature else -.355)),('hindleg_right',1,(hind,.28,-.445 if mature else -.355)),
      ('plant_base',1,(0.,.48 if mature else .47,-.16)),('flower_crown' if mature else 'bud_tip',7,(0.,.94 if mature else .83,-.16 if mature else -.19))]
 for j in range(6):
  base=len(raw);raw.extend(((f'leaf_base_{j}',7,blade_frame(s,j,.10)),(f'leaf_tip_{j}',base,blade_frame(s,j,.58))))
 if mature:
  for j in range(5):
   base=len(raw);raw.extend(((f'petal_base_{j}',8,blade_frame(s,j,.10,petal=True)),(f'petal_tip_{j}',base,blade_frame(s,j,.58,petal=True))))
  raw.append(('flower_heart',8,(0.,1.057,-.16)))
 return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(s,p)) for n,parent,p in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',4.,81),('attack',1.,65),('hit',1.,65))
def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def raw_point(s,p):
 lo,k=authoring_transform(s);return p[0]/k,p[1]/k+lo,p[2]/k
def blade_fraction(s,j,p,*,petal=False):
 angle=math.tau*j/(5 if petal else 6)+(.16 if petal else .22);length=.61 if petal or s=='venusaur' else .43
 return max(0.,min(1.,(math.sin(angle)*p[0]+math.cos(angle)*(p[2]+.16)-.035)/length))
def blade_weights(s,j,p,*,petal=False):
 t=blade_fraction(s,j,p,petal=petal);a=smoothstep(.035,.28,t);b=smoothstep(.42,.92,t)
 base=(21 if petal else 9)+2*j
 return [8 if petal else 7,base,base+1,0],[1.-a,a*(1.-b),a*b,0.]
def skin_weights(s,n,point):
 p=raw_point(s,point);x,y,z=p
 if n.startswith(('Forefoot /','Hindfoot /','Foreclaw /','Hindclaw /')):return rigid(0)
 if n.startswith(('Head /','Head wart /','Ear /','Ear inset /','Eye /','Eye glint /','Iris /','Pupil /','Brow /','Head spot /','Mouth /','Tongue /','Nostril /','Fang /')):return rigid(2)
 if n.startswith(('Foreleg /','Hindleg /','Foreleg spot /','Hindleg spot /')):
  joint=(3 if x<0 else 4) if n.startswith('Fore') else (5 if x<0 else 6)
  lift=smoothstep(.05,.15,y);shoulder=smoothstep(.25,.39,y)
  return [0,joint,1,0],[1.-lift,lift*(1.-shoulder),lift*shoulder,0.]
 if n.startswith(('Body /','Body wart /')):
  a=smoothstep(.065,.23,y);return [0,1,0,0],[1.-a,a,0.,0.]
 if n.startswith(('Bud /','Trunk /')):
  a=smoothstep(.64 if s=='venusaur' else .66,.94 if s=='venusaur' else .98,y);return [7,8,0,0],[1.-a,a,0.,0.]
 if n.startswith(('Leaf /','Leaf midrib /')):return blade_weights(s,int(n.split()[-1]),p)
 if n.startswith(('Petal /','Petal spot /')):
  j=int(n.split()[-2] if n.startswith('Petal spot /') else n.split()[-1]);return blade_weights(s,j,p,petal=True)
 if n.startswith(('Flower heart /','Stamen /')):return rigid(31)
 raise ValueError('unowned evolved seed saur anatomy '+n)
def pose_angles(s,clip,t):
 if s not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid seed saur cue/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[s])
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
 if clip=='idle':a=wave;b=breath;gain=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.62),(.72,.08),(1.,0.)))
  b=envelope(t,((0.,0.),(.30,1.),(.54,.56),(.80,-.10),(1.,0.)));gain=1.8 if clip=='attack' else -1.4
 a*=gain;b*=gain
 out=[(0.,0.,0.),(.008*a,.008*a,.004*a),(.012*a,.018*a,-.004*a),
      (-.010*a,0.,.006*a),(-.010*a,0.,-.006*a),(.008*a,0.,.005*a),(.008*a,0.,-.005*a),
      (.009*b,.008*a,.004*b),(.015*b,-.010*a,.006*a)]
 for j in range(6):
  phase=.20+.13*j;lag=(math.sin(math.tau*t-phase)+math.sin(phase)) if clip=='idle' else b
  sign=-1 if math.sin(math.tau*j/6+.22)<0 else 1
  out.extend(((.014*b,.013*a*sign,.012*lag*sign),(.026*lag,.017*a*sign,.019*b*sign)))
 if s=='venusaur':
  for j in range(5):
   sign=-1 if math.sin(math.tau*j/5+.16)<0 else 1;phase=.35+.14*j;lag=(math.sin(math.tau*t-phase)+math.sin(phase)) if clip=='idle' else b
   out.extend(((.010*b,.010*a*sign,.011*lag*sign),(.020*lag,.013*a*sign,.016*b*sign)))
  out.append((.008*b,.012*a,.006*b))
 return out
def joint_rotation(s,j,angles):return quaternion_xyz(*angles)
def pack_parts(model):
 result=[];groups={}
 for order,p in enumerate(model['primitives']):
  n=p['part'];category=None;label=''
  if n.startswith(('Head /','Head wart /','Ear /','Ear inset /','Eye /','Eye glint /','Iris /','Pupil /','Brow /','Head spot /','Mouth /','Tongue /','Nostril /','Fang /')):category='Head'
  elif model['name']=='venusaur' and n.startswith(('Body /','Body wart /')):category='Body'
  elif n.startswith(('Forefoot /','Foreclaw /','Hindfoot /','Hindclaw /')):
   category='Forefoot' if n.startswith('Fore') else 'Hindfoot';label=n.split()[-2] if 'claw /' in n else n.split()[-1]
  elif n.startswith('Leaf /'):category='Leaf bank'
  elif n.startswith('Leaf midrib /'):category='Leaf midribs'
  elif n.startswith('Petal /'):category='Petal bank'
  elif n.startswith('Petal spot /'):category='Petal markings'
  elif n.startswith(('Flower heart /','Stamen /')):category='Flower heart'
  if category is None:result.append((p,{'source_order':order}));continue
  key=(category,label,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=category+' / coherent seed saur anatomy '+label+' material '+str(len(groups)),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3;meta['source_parts'].append(dict(part=n,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices']+=[i+start for i in p['indices']]
 return result

def export_bulb_saur(species, model=None):
    if species not in SPECIES:raise ValueError('unknown bulb_saur')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed bulb_saur geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated bulb_saur standard skin v1'
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
        'geometry_source':'tools/bulb_saur_sculpt.py:sculpture','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
        'animation_contract':'rotation-only source cues; coherent saur faces, four planted complete feet and twelve claws; independent rooted leaves and matching midribs; folded bud response or woody flower stalk, five spotted flexing petals and a rooted golden heart'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('bulb_saur exceeds existing model limits')
    return blob


def decode_bulb_saur(blob, species):
    if len(blob)>1024*1024 or blob!=export_bulb_saur(species):
        raise ValueError('noncanonical bulb_saur GLB')
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
def canonical_blob(s):return export_bulb_saur(s,authoring_model(s))
def read_bulb_saur(path,s):
 with Path(path).open('rb') as f:return decode_bulb_saur(f.read(1024*1024+1),s)
def read_ivysaur(path):return read_bulb_saur(path,'ivysaur')
def read_venusaur(path):return read_bulb_saur(path,'venusaur')
if __name__=='__main__':
 import argparse
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_bulb_saur(s))
