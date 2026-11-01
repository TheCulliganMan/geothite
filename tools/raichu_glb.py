#!/usr/bin/env python3
"""Canonical Raichu skin: coherent curled ears, grounded paws and a flexible thin tail."""
import hashlib,json,math,struct
from functools import lru_cache
from pathlib import Path
from raichu_sculpt import sculpture,HEIGHT,TAIL,authoring_transform as source_transform
from bovine_sculpt import smooth
from skin_glb import Glb,COORDINATES,packed,quantized_weights,quaternion_xyz,envelope,smoothstep
SPECIES=('raichu',);HEIGHTS={'raichu':HEIGHT}
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=1)
def authoring_model(s):
 if s!='raichu':raise ValueError('unknown electric mouse')
 return sculpture()
def normalized(p):
 lo,k=source_transform();return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(p))
def raw_point(p):
 lo,k=source_transform();return tuple(v/k+(lo if i==1 else 0.) for i,v in enumerate(p))
PARTS={'raichu':tuple(p['part'] for p in sculpture()['primitives'])}
REST_DIGESTS={'raichu': '648cdc04220d10c5d9f0ffcb22a3f8b6fe77054184ab9034f225b344e6927528'}
RAW_JOINTS=(('root',None,(0.,0.,0.)),('body',0,(0.,.20,-.01)),('head',1,(0.,.50,.02)),('arm_left',1,(-.16,.43,.035)),('arm_right',1,(.16,.43,.035)),('ear_left',2,(-.135,.745,-.04)),('ear_right',2,(.135,.745,-.04)),('ear_left_tip',5,(-.27,.94,-.055)),('ear_right_tip',6,(.27,.94,-.055)),('tail_base',1,TAIL[0]),('tail_tip',9,TAIL[3]),('leg_left',0,(-.125,.17,.015)),('leg_right',0,(.125,.17,.015)))
JOINTS={'raichu':tuple((n,parent,(0.,0.,0.) if parent is None else normalized(p)) for n,parent,p in RAW_JOINTS)}
CLIPS=(('idle',3.4,85),('attack',1.,65),('hit',1.,65))
def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
def influences(pairs):
 pairs=[(j,w) for j,w in pairs if w>1e-12]
 if len(pairs)==1:return rigid(pairs[0][0])
 return [j for j,w in pairs]+[0]*(4-len(pairs)),[w for j,w in pairs]+[0.]*(4-len(pairs))
@lru_cache(maxsize=1)
def tail_samples():return smooth(TAIL,[1.]*len(TAIL))[0]
def tail_fraction(point):
 path=tail_samples();lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.;best=(math.inf,0.)
 for a,b,length in zip(path,path[1:],lengths):
  d=[y-x for x,y in zip(a,b)];t=max(0.,min(1.,sum((v-x)*u for v,x,u in zip(point,a,d))/(length*length)));distance=math.dist(point,[x+t*u for x,u in zip(a,d)]);best=min(best,(distance,(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(s,n,p):
 x,y,z=raw_point(p)
 if n.startswith('Foot /'):return rigid(0)
 if n.startswith('Body /'):
  w=smoothstep(.07,.13,y);return influences(((0,1.-w),(1,w)))
 if n.startswith(('Belly /','Back stripe /')):return rigid(1)
 if n.startswith(('Head /','Eye /','Eye glint /','Cheek /','Nose /','Mouth /')):return rigid(2)
 if n.startswith(('Ear /','Ear inset /','Ear curl /','Ear curl inset /')):
  base=5 if n.endswith('-1') else 6;tip=base+2;a=smoothstep(.74,.82,y);b=smoothstep(.93,1.025,y)
  return influences(((2,1.-a),(base,a*(1.-b)),(tip,a*b)))
 if n.startswith('Arm /'):
  w=smoothstep(.17,.25,abs(x));return influences(((1,1.-w),(3 if n.endswith('-1') else 4,w)))
 if n.startswith(('Hand /','Thumb /')):return rigid(3 if n.endswith('-1') else 4)
 if n.startswith('Finger /'):return rigid(3 if n.split()[-2]=='-1' else 4)
 if n.startswith('Leg /'):
  w=smoothstep(.04,.115,y);return influences(((0,1.-w),(11 if n.endswith('-1') else 12,w)))
 if n.startswith('Tail cord /'):
  w=smoothstep(.40,.76,tail_fraction((x,y,z)));return influences(((9,1.-w),(10,w)))
 if n.startswith('Tail bolt /'):return rigid(10)
 raise ValueError('unowned Raichu anatomy '+n)
def pose_angles(s,clip,t):
 if s!='raichu' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Raichu pose')
 if t in (0.,1.):return [(0.,0.,0.)]*13
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);gain=1.
 else:a=envelope(t,((0.,0.),(.18,-.25),(.34,1.),(.63,.18),(1.,0.)));b=envelope(t,((0.,0.),(.26,-.20),(.46,1.),(.76,-.08),(1.,0.)));gain=2. if clip=='attack' else -1.5
 a*=gain;b*=gain
 return [(0.,0.,0.),(0.,.014*a,0.),(.018*a,.025*b,.008*a),(-.06*a,0.,-.025*b),(-.06*a,0.,.025*b),(.020*b,0.,-.020*a),(.020*b,0.,.020*a),(.030*a,0.,-.025*b),(.030*a,0.,.025*b),(.010*a,.035*a,0.),(.015*b,.055*b,0.),(0.,-.020*b,0.),(0.,.020*b,0.)]
def pack_parts(model):
 result=[];groups={};s=model['name']
 for order,p in enumerate(model['primitives']):
  n=p['part'];prefix=n.split(' /')[0];group='Face' if prefix in ('Eye','Eye glint','Cheek','Nose','Mouth') else 'Hand' if prefix in ('Hand','Finger','Thumb') else 'Back stripe' if prefix=='Back stripe' else None
  if group is None:result.append((p,{'source_order':order}));continue
  owner=skin_weights(s,n,(0.,0.,0.))[0][0];key=(group,owner,tuple(p['base_color']))
  if key not in groups:
   q=dict(part=group+' / coherent Raichu carrier '+str(owner)+' '+str(len(groups)),positions=[],normals=[],indices=[],base_color=p['base_color']);meta={'source_parts':[]};groups[key]=(q,meta);result.append((q,meta))
  q,meta=groups[key];start=len(q['positions'])//3;meta['source_parts'].append(dict(part=n,order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
 return result

def export_mouse(species, model=None):
    if species not in SPECIES:raise ValueError('unknown mouse')
    if model is None:return canonical_blob(species)
    if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
        or geometry_digest(model)!=REST_DIGESTS[species]
        or [p['part'] for p in model['primitives']]!=list(PARTS[species])):
        raise ValueError('unreviewed mouse geometry')
    g = Glb();g.doc['asset']['generator'] = 'Geothite original articulated mouse standard skin v1'
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
        'geometry_source':'tools/raichu_sculpt.py','animation_contract':'rotation-only source cues; planted mouse paws; coherent fitted face and belly; curled pinnae and their linings share longitudinal weights; independent ear tips and flexible thin cord with rigid bolt'}
    blob = g.bytes()
    if (len(g.doc['accessors'])>256 or len(parts)>64 or len(js)>64 or len(blob)>1024*1024
        or sum(len(p['positions'])//3 for p,meta in parts)>20000 or sum(len(p['indices'])//3 for p,meta in parts)>40000):
        raise ValueError('mouse exceeds existing model limits')
    return blob


def decode_mouse(blob, species):
    if len(blob)>1024*1024 or blob!=export_mouse(species):
        raise ValueError('noncanonical mouse GLB')
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
def canonical_blob(species):return export_mouse(species,authoring_model(species))
def read_mouse(path,species):
 with Path(path).open('rb') as f:return decode_mouse(f.read(1024*1024+1),species)
def export_raichu(model=None):return export_mouse('raichu',model)
def read_raichu(path):return read_mouse(path,'raichu')
def main():
 import argparse
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'raichu.glb').write_bytes(export_raichu())
if __name__=='__main__':main()
