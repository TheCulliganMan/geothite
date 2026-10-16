#!/usr/bin/env python3
"""Standard water-rabbit skins with welded bellies, coherent ears and attached tail balls."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from aqua_rabbit_sculpt import sculpture,HEIGHT
SPECIES=('marill','azumarill');HEIGHTS={s:HEIGHT for s in SPECIES}
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'marill': '39c4a5f6e20db9f7c4aeddef3353131a3dda11ddd1befd021dfcb5aa1e88fbe7', 'azumarill': '19aba9a5fabbfd5f187d24b28f9c10fc291c0312046b6142dfa73d3e11219333'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

TAILS={'marill':((-.218,.262,-.127),(-.318,.226,-.211),(-.376,.361,-.234),(-.447,.301,-.244),(-.489,.461,-.252),(-.559,.499,-.256)),
 'azumarill':((.219,.217,-.151),(.350,.190,-.229),(.329,.292,-.248),(.473,.266,-.277),(.453,.374,-.300),(.586,.361,-.307))}
def joints(species):
 adult=species=='azumarill';tail=TAILS[species]
 raw=[('root',None,(0.,0.,0.)),('torso',0,(0.,.4 if adult else .3,0.)),('head',1,(0.,.62 if adult else .43,0.))]
 for side in (-1,1):
  base=len(raw);raw.extend((('ear_'+('left' if side<0 else 'right'),2,(side*.124,.838,.008) if adult else (side*.233,.60,.003)),
   ('ear_'+('left' if side<0 else 'right')+'_tip',base,(-.218,1.149,.026) if adult and side<0 else (.258,1.180,.012) if adult else (side*.233,.72,.003))))
 raw.extend((('arm_left',1,(-.263,.447,.008) if adult else (-.251,.341,.023)),('arm_right',1,(.263,.447,.008) if adult else (.251,.341,.023)),
  ('tail_base',1,tail[0]),('tail_bend',9,tail[3]),('tail_ball',10,tail[-1])))
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={species:joints(species) for species in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def path_fraction(point,path):
 best=(math.inf,0.);lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.
 for a,b,length in zip(path,path[1:],lengths):
  delta=[y-x for x,y in zip(a,b)];t=max(0.,min(1.,sum((p-x)*d for p,x,d in zip(point,a,delta))/(length*length)))
  best=min(best,(math.dist(point,[x+t*d for x,d in zip(a,delta)]),(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(species,part,position):
 adult=species=='azumarill';low,span=authoring_transform(species)
 raw=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position));x,y,z=raw
 if part.startswith('Foot /'):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Body /','Belly /')):
  f=smoothstep(.46 if adult else .35,.575 if adult else .39,y)
  return [1,2,0,0],[1.-f,f,0.,0.]
 if part.startswith('Ear /'):
  left=part.endswith('-1');base=3 if left else 5
  a=smoothstep(.838 if adult else .56,.97 if adult else .66,y)
  b=smoothstep((1.05 if left else 1.08) if adult else .68,(1.19 if left else 1.26) if adult else .75,y)
  return [2,base,base+1,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith('Arm /'):
  owner=7 if part.endswith('-1') else 8
  f=smoothstep(.263 if adult else .251,.37 if adult else .34,abs(x))
  return [1,owner,0,0],[1.-f,f,0.,0.]
 if part.startswith('Tail / blue'):return [11,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail /'):
  t=path_fraction(raw,TAILS[species]);a=smoothstep(0.,.3,t);b=smoothstep(.25,.70,t);c=smoothstep(.65,.90,t)
  return [1,9,10,11],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
 return [2,0,0,0],[1.,0.,0.,0.]
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid water-rabbit clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 return [(0.,0.,0.),(0.,.014*a,0.),(0.,.020*a,0.),
  (.010*a,0.,-.020*a),(.015*b,0.,-.025*b),(.010*a,0.,.020*a),(.015*b,0.,.025*b),
  (.015*a,.015*b,-.025*a),(.015*a,-.015*b,.025*a),(0.,.020*a,0.),(.018*b,.025*b,0.),(.020*b,.030*b,0.)]
def export_water_rabbit(species,model=None):
 if species not in SPECIES:raise ValueError('unknown water_rabbit species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed water_rabbit neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite water rabbit welded belly and articulated buoy skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS[species]
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':species+'/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':species,'mesh':0,'skin':0})
 g.doc['scenes']=[{'name':species,'nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':species,'skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  # Preserve exact f32 bytes, including signed zero in authored normals.
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':species+'/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(species,part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':species,'primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(species,clip,i/(count-1))[joint])]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':species+'.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted paddles; welded belly seam and fitted spots; coherent gaze; articulated ears, arms and tail buoy; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/aqua_rabbit_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; planted paddles; welded two-color body and fitted spot field; coherent gaze; separate ear bases/tips with shared linings, arm paddles and three-stage tail; attached buoy; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'water_rabbit exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_water_rabbit(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_water_rabbit(species):raise ValueError('noncanonical water_rabbit GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_water_rabbit(path,species):
 with Path(path).open('rb') as f:return decode_water_rabbit(f.read(MAX_BYTES+1),species)
def export_marill(model=None):return export_water_rabbit('marill',model)
def export_azumarill(model=None):return export_water_rabbit('azumarill',model)
def read_marill(path):return read_water_rabbit(path,'marill')
def read_azumarill(path):return read_water_rabbit(path,'azumarill')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_water_rabbit(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
