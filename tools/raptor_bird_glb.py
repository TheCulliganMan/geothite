#!/usr/bin/env python3
"""Articulated crested Pidgeot and long-necked Fearow; canonical recipes, no sidecars."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from raptor_bird_sculpt import sculpture,SPECIES,HEIGHTS
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=2)
def authoring_bundle(s):return sculpture(s,rig_ready=True,with_transform=True)
def authoring_model(s):return authoring_bundle(s)[0]
def authoring_transform(s):return authoring_bundle(s)[1]
def normalized(s,p):
 lo,k=authoring_transform(s);return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(p))
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
REST_DIGESTS={'pidgeot': 'f9c6d9be6d7789060f1079349ed13a4485e0533fb665ff92b363e4453dbab74a', 'fearow': '6a1f4b7a3c30abd876b8955986224d09f1c3929828520ccacd6003c3dd679dc3'}

def joints(s):
 fear=s=='fearow'
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.40,-.02)),('neck_base',1,(0.,.59,.04 if fear else .02)),('neck_tip',2,(0.,.78,.21 if fear else .02)),('head',3,(0.,.94,.358) if fear else (0.,.82,.02)),('lower_bill',4,(0.,.92,.468) if fear else (0.,.81,.16)),('wing_left',1,(-.105,.59,-.02)),('wrist_left',6,(-.50,.90,-.09)),('wing_right',1,(.105,.59,-.02)),('wrist_right',8,(.50,.90,-.09))]
 if not fear:
  from raptor_bird_sculpt import folded_wing
  for i in (6,7,8,9):
   n,parent,p=raw[i];side=-1 if p[0]<0 else 1
   q=folded_wing(dict(positions=list(p),normals=[]),side);raw[i]=(n,parent,tuple(q['positions']))
  raw[6]=('wing_left',1,(-.180,.550,-.020));raw[8]=('wing_right',1,(.180,.550,-.020))
  raw[2]=('neck_base',1,(0.,.58,.04));raw[3]=('neck_tip',2,(0.,.67,.06));raw[4]=('head',3,(0.,.705,.07));raw[5]=('lower_bill',4,(0.,.685,.205))
 else:
  from fearow_sculpt import wing_frame
  for i in (6,7,8,9):
   n,parent,p=raw[i];side=-1 if p[0]<0 else 1
   q=wing_frame(dict(positions=list(p),normals=[]),side);raw[i]=(n,parent,tuple(q['positions']))
  raw[1]=('body',0,(0.,.59,-.13));raw[2]=('neck_base',1,(0.,.73,.07));raw[3]=('neck_tip',2,(0.,.815,.285));raw[4]=('head',3,(0.,.740,.455));raw[5]=('lower_bill',4,(0.,.700,.538))
 raw.extend((f'tail_{j}',1,((j-2)*.012,.575 if fear else .30,-.325 if fear else -.145)) for j in range(5))
 raw.extend((('leg_left',1 if fear else 0,(-.105,.49,-.10) if fear else (-.108,.11,.02)),('leg_right',1 if fear else 0,(.105,.49,-.10) if fear else (.108,.11,.02))))
 if fear:raw.extend((('crown',4,(0.,.783,.490)),('mane',2,(0.,.75,.075))))
 else:raw.extend((f'crest_{j}',4,(j*.065,.805,.19)) for j in (-1,0,1))
 return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(s,p)) for n,parent,p in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',1.6,81),('attack',1.,65),('hit',1.,65))
def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def neck_weights(s,y):
 if s=='pidgeot':
  a=smoothstep(.555,.620,y);b=smoothstep(.640,.705,y);c=smoothstep(.700,.750,y)
  return [1,2,3,4],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
 a=smoothstep(.58,.69,y);b=smoothstep(.70,.82 if s=='pidgeot' else .86,y);c=smoothstep(.78 if s=='pidgeot' else .86,.85 if s=='pidgeot' else .94,y)
 return [1,2,3,4],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
@lru_cache(maxsize=2)
def neck_weight_surface(s):
 # Fitted throat vertices inherit the quantized weights of the actual neck
 # triangle beneath them. Evaluating the analytic neck field independently at
 # a denser marking vertex can pull that marking beneath a coarser neck face.
 parent=next(p for p in authoring_model(s)['primitives'] if p['part'].startswith('Neck /'))
 ps=struct.unpack('<'+'f'*len(parent['positions']),packed(parent['positions']));points=[ps[i:i+3] for i in range(0,len(ps),3)];lo,k=authoring_transform(s);weights=[quantized_weights(neck_weights(s,p[1]/k+lo)[1]) for p in points];triangles=[]
 for i in range(0,len(parent['indices']),3):
  ids=parent['indices'][i:i+3];a,b,c=[points[j] for j in ids];den=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
  if den>1e-12:triangles.append((ids,a,b,c,den))
 def at(point):
  x,y,z=point;best=None
  for ids,a,b,c,den in triangles:
   u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/den;v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/den;ws=(u,v,1-u-v)
   if min(ws)<-1e-6:continue
   height=sum(w*p[2] for w,p in zip(ws,(a,b,c)))
   if best is None or height>best[0]:best=(height,ids,ws)
  if best is None:raise ValueError('fitted throat lies outside stored neck')
  _,ids,ws=best;ws=[max(0.,w) for w in ws];total=sum(ws)
  return [1,2,3,4],[sum(ws[i]*weights[ids[i]][j]/255. for i in range(3))/total for j in range(4)]
 return at
def skin_weights(s,n,point):
 lo,k=authoring_transform(s);x,y,z=(point[0]/k,point[1]/k+lo,point[2]/k)
 if s=='fearow':
  if n.startswith(('Foot /','Toe /','Talon /','Leg /')):
   side=int(n.split()[-2] if n.startswith(('Toe / three','Talon / forward')) else n.split()[-1]);return rigid(15 if side<0 else 16)
  if n.startswith('Neck /'):
   a=smoothstep(.075,.175,z);b=smoothstep(.19,.34,z);c=smoothstep(.35,.445,z)
   return [1,2,3,4],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
  if n.startswith('Crown /'):return rigid(17)
  if n.startswith('Mane /'):return rigid(18)
  if n.startswith('Tail /'):
   a=smoothstep(.02,.15,-z-.325);return [1,10+int(n.split()[-1]),0,0],[1.-a,a,0.,0.]
 if n.startswith(('Foot /','Toe /','Talon /')):return rigid(0)
 if n.startswith('Leg /'):
  j=15 if n.endswith('-1') else 16;a=smoothstep(.045,.26,y);b=.4*smoothstep(.05,.13,y)*(1.-smoothstep(.20,.275,y));return [0,1,j,0],[(1.-a)*(1.-b),a*(1.-b),b,0.]
 if n.startswith(('Body /','Breast /')):return rigid(1)
 if n.startswith('Throat /'):return neck_weight_surface(s)(point)
 if n.startswith('Neck /'):return neck_weights(s,y)
 if n.startswith(('Head /','Cheek /','Mask /','Eye /','Eye socket /','Pupil /','Glint /')):return rigid(4)
 if n.startswith('Bill /'):return rigid(5 if 'lower' in n or 'mandible' in n or 'seam' in n else 4)
 if n.startswith('Crest /'):
  return rigid(18+max(-1,min(1,int(n.split()[-1]))))
 if n.startswith('Crown /'):
  a=smoothstep(1.01,1.16,y);return [4,17,0,0],[1.-a,a,0.,0.]
 if n.startswith('Mane /'):
  a=.6*smoothstep(.69,.79,y)*(1.-smoothstep(.87,.97,y));return [2,18,0,0],[1.-a,a,0.,0.]
 if n.startswith('Tail /'):
  j=10+int(n.split()[-1]);a=smoothstep(.0,.12,.355-y) if s=='fearow' else smoothstep(.025,.23,-z-.145);return [1,j,0,0],[1.-a,a,0.,0.]
 if n.startswith(('Wing /','Wing field /','Primary /','Rear primary /','Covert /','Rear covert /','Primary tip /','Rear primary tip /')):
  j=6 if x<0 else 8
  if s=='pidgeot':
   from raptor_bird_sculpt import folded_wing
   x=folded_wing(dict(positions=[x,y,z],normals=[]),-1 if x<0 else 1,True)['positions'][0]
  else:
   from fearow_sculpt import wing_frame
   x=wing_frame(dict(positions=[x,y,z],normals=[]),-1 if x<0 else 1,True)['positions'][0]
  b=smoothstep(.43,.84,abs(x));return [j,j+1,0,0],[1.-b,b,0.,0.]
 raise ValueError('unowned raptor anatomy '+n)
def pose_angles(s,clip,t):
 if s not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid raptor cue/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[s])
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);gain=1.
  a=math.sin(math.tau*t)*(1.-math.cos(math.tau*t))*.769800358919501;b=a
 else:a=envelope(t,((0.,0.),(.18,1.),(.42,.6),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));gain=1.5 if clip=='attack' else -1.1
 a*=gain;b*=gain
 out=[(0.,0.,0.),(0.,.008*a,0.),(.008*b,.010*a,0.),(.010*a,-.012*b,0.),(.012*b,.018*a,.005*b),(.022*max(a,0.),0.,0.),(.01*b,.01*a,-.20*a),(.008*a,0.,-.035*b),(.01*b,-.01*a,.20*a),(.008*a,0.,.035*b)]
 if s=='pidgeot' and clip=='idle':out[6]=(.01*b,.01*a,-.025*a);out[8]=(.01*b,-.01*a,.025*a)
 out.extend((.010*b,(j-2)*.010*a,.008*(j-2)*b) for j in range(5));out.extend(((0.,.012*a,0.),(0.,-.012*a,0.)))
 if s=='pidgeot':out.extend((.010*b,.012*a*j,.012*b*j) for j in (-1,0,1))
 else:out.extend(((.014*b,.012*a,0.),(.008*a,.010*b,0.)))
 return out

def joint_rotation(s,j,angles):return quaternion_xyz(*angles)
def pack_parts(model):
 result=[];groups={}
 for order,p in enumerate(model['primitives']):
  n=p['part'];category=None;label=''
  if n.startswith(('Wing /','Wing field /','Primary /','Rear primary /','Covert /','Rear covert /','Primary tip /','Rear primary tip /')):category='Wing bank';label='left' if sum(p['positions'][::3])<0 else 'right'
  elif n.startswith(('Foot /','Toe /','Talon /')):category='Airborne digits' if model['name']=='fearow' else 'Grounded digits'
  elif n.startswith(('Head /','Cheek /','Mask /','Eye /','Eye socket /','Pupil /','Glint /')):category='Coherent head'
  elif n.startswith('Tail /'):category='Tail fan'
  if category is None:result.append((p,{'source_order':order}));continue
  key=(category,label,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=category+' / '+label+' material '+str(len(groups)),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3;meta['source_parts'].append(dict(part=n,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices']+=[i+start for i in p['indices']]
 return result
def export_raptor_bird(species, model=None):
    if species not in SPECIES:raise ValueError('unknown raptor_bird')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed raptor_bird geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated raptor_bird standard skin v1'
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
        'geometry_source':'tools/raptor_bird_sculpt.py:sculpture(rig_ready=True)','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
        'animation_contract':('rotation-only source cues; airborne tucked clawed feet; rooted wing and wrist banks; coherent fitted head and bill; forward arched neck; five tail feathers; sagittal crown and back ruff' if species=='fearow' else 'rotation-only source cues; planted complete clawed feet; rooted wing and wrist banks; coherent fitted head and bill; two-stage neck; five tail feathers; swept crest or crown and mane')}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('raptor_bird exceeds existing model limits')
    return blob


def decode_raptor_bird(blob, species):
    if len(blob)>1024*1024 or blob!=export_raptor_bird(species):
        raise ValueError('noncanonical raptor_bird GLB')
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
def canonical_blob(s):return export_raptor_bird(s,authoring_model(s))
def read_raptor_bird(path,s):
 with Path(path).open('rb') as f:return decode_raptor_bird(f.read(1024*1024+1),s)
def read_pidgeot(path):return read_raptor_bird(path,'pidgeot')
def read_fearow(path):return read_raptor_bird(path,'fearow')
if __name__=='__main__':
 import argparse
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_raptor_bird(s))
