#!/usr/bin/env python3
"""Voltorb and Electrode coherent shell skins above grounded support bands."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from orb_sculpt import sculpture as orb_sculpture,HEIGHT as ORB_HEIGHT
HEIGHT=ORB_HEIGHT['voltorb']
def sculpt():
 model=orb_sculpture('voltorb');model['name']='voltorb';return model
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

SPECIES=('voltorb','electrode')
REST_DIGESTS={'voltorb':'560b40b398d50fcbeb2944890bd8e60733c1182c83c20b824e84b8a358f4e605','electrode':'cc8828f83e653417b3a084b8e0ca908deb1f3a5fe4acfea67e831e87050e7571'}
PARTS={s:tuple(p['part'] for p in orb_sculpture(s)['primitives']) for s in SPECIES};MAX_BYTES=1024*1024

def sculpture(species):
 if species not in SPECIES:raise ValueError('unsupported orb')
 model=orb_sculpture(species);model['name']=species;return model
JOINTS={s:(('root',None,(0.,0.,0.)),('body',0,(0.,ORB_HEIGHT[s]*.5,0.))) for s in SPECIES}

CLIPS=(('idle',3.4,85),('attack',1.,65),('hit',1.,65))


def skin_weights(species,part,position):
 if part.startswith('Lower shell /'):
  height=ORB_HEIGHT[species];body=smoothstep(height*.015,height*(.20 if species=='voltorb' else .15),position[1])
  return [1,0,0,0],[body,1.-body,0.,0.]
 return [1,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid orb clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*2
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
 if clip=='idle':return [(0.,0.,0.),(.014*wave,.018*wave,.010*breath)]
 early=envelope(t,((0.,0.),(.18,1.),(.38,.67),(.67,.12),(1.,0.)))
 trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.79,-.08),(1.,0.)))
 if clip=='attack':return [(0.,0.,0.),(.075*early,.020*trail,.016*trail)]
 return [(0.,0.,0.),(-.060*early,-.018*trail,-.012*trail)]


def export_orb(species,model=None):
 if species not in SPECIES:raise ValueError('unknown orb species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed orb neutral geometry')
 g=Glb();g.doc['asset']['generator']=('Geothite coherent Voltorb orb skin v1' if species=='voltorb' else 'Geothite coherent Electrode orb skin v1')
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
  key=(tuple(values),kind,component,target,normalized)
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
    'NORMAL':g.accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':species,'primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(species,clip,i/(count-1))[joint])]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':species+'.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'early coherent shell rock; face and equator stay together; pinned support band; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':ORB_HEIGHT[species],'body_reference_height':ORB_HEIGHT[species],'full_silhouette_height':ORB_HEIGHT[species],
    'geometry_source':'tools/orb_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; pinned lower shell support; coherent shell, equator and fitted face; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('orb exceeds existing accessor limit')
 return g.bytes()


def decode_orb(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_orb(species):raise ValueError('noncanonical orb GLB')
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


def read_orb(path,species):
 with Path(path).open('rb') as f:return decode_orb(f.read(MAX_BYTES+1),species)
def export_electrode(model=None):return export_orb('electrode',model)
def read_electrode(path):return read_orb(path,'electrode')
def export_voltorb(model=None):return export_orb('voltorb',model)
def read_voltorb(path):return read_orb(path,'voltorb')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,default='voltorb');p.add_argument('--output',type=Path);a=p.parse_args();a.output=a.output or Path('crates/crystal-voxel-view/models/battle_species')/(a.species+'.glb');a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_orb(a.species);a.output.write_bytes(blob);print(len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
