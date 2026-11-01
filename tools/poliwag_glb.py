#!/usr/bin/env python3
"""Canonical Poliwag standard glTF skin and source-cue body response."""
import argparse
import hashlib
import math
from pathlib import Path
from poliwag_sculpt import sculpture,HEIGHT,authoring_transform
from sculpt_geometry import sub
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
PARTS=tuple(p['part'] for p in sculpture()['primitives'])
MAX_BYTES=1024000
_floor,_scale=authoring_transform()
def normalized(point):return tuple((v-(_floor if i==1 else 0.))*_scale for i,v in enumerate(point))
JOINTS=(('root',None,(0.,0.,0.)),('body',0,normalized((0.,.306,0.))),
        ('tail_base',1,normalized((0.,.176,-.156))),('tail_middle',2,normalized((0.,.190,-.472))),
        ('tail_tip',3,normalized((0.,.219,-.667))))
CLIPS=(('idle',3.4,85),('attack',1.,65),('hit',1.,65))


def skin_weights(part,position):
    x,y,z=position
    if part.startswith('Foot /'):return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Belly /','Spiral /','Eye /','Pupil /','Eye glint /','Eye reflection /','Mouth /','Mouth opening /')):return [1,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Tail stem /','Tail fin /','Tail vein /')):
        rear=-z/_scale
        if rear<.28:
            blend=smoothstep(.16,.28,rear);return [1,2,0,0],[1.-blend,blend,0.,0.]
        if rear<.57:
            blend=smoothstep(.35,.57,rear);return [2,3,0,0],[1.-blend,blend,0.,0.]
        blend=smoothstep(.60,.77,rear);return [3,4,0,0],[1.-blend,blend,0.,0.]
    return [1,0,0,0],[1.,0.,0.,0.]


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Poliwag clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(p):return math.sin(math.tau*t-p)+math.sin(p)
    if clip=='idle':return [(0.,0.,0.),(.010*breath,.012*wave,.006*wave),
        (.007*wave,.030*wave,0.),(.010*lag(.4),.043*lag(.4),0.),(.012*lag(.8),.052*lag(.8),0.)]
    brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
    late=envelope(t,((0.,0.),(.35,1.),(.59,.60),(.82,-.12),(1.,0.)))
    if clip=='attack':return [(0.,0.,0.),(.034*brace,0.,0.),(.016*brace,.045*brace,0.),(.022*trail,.065*trail,0.),(.027*late,.083*late,0.)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    return [(0.,0.,0.),(-.026*recoil,.008*recoil,0.),(-.014*recoil,-.035*recoil,0.),(-.021*trail,-.059*trail,0.),(-.026*late,-.074*late,0.)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_poliwag(model=None):
    canonical=sculpture()
    if model is None:model=canonical
    if (model.get('name')!='poliwag' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Poliwag anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Poliwag round tadpole and tail skin and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'poliwag/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'poliwag','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'poliwag','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'poliwag','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'poliwag/paper_'+str(colors[color]),
                'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
        ids,weights=[],[]
        for i in range(0,len(part['positions']),3):
            js,ws=skin_weights(part['part'],part['positions'][i:i+3]);qs=quantized_weights(ws)
            ids.extend(j if w else 0 for j,w in zip(js,qs));weights.extend(qs)
        primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
            'NORMAL':g.accessor(part['normals'],'VEC3',target=34962),
            'JOINTS_0':g.accessor(ids,'VEC4',5121,34962),
            'WEIGHTS_0':g.accessor(weights,'VEC4',5121,34962,normalized=True)},
            'indices':g.accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],
            'mode':4,'extras':{'part':part['part']}})
    g.doc['meshes']=[{'name':'poliwag','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'poliwag.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'gentle tadpole sway; early body brace; delayed tail-chain settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'poliwag',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/poliwag_sculpt.py',
        'animation_contract':'identity floor root; planted paddle feet; rigid fitted face and belly spiral; three-joint tadpole tail; rotation-only source-cue clips; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_poliwag(blob):
    if len(blob)>MAX_BYTES or blob!=export_poliwag():raise ValueError('noncanonical Poliwag GLB')
    return sculpture()


def read_poliwag(path):
    with Path(path).open('rb') as source:return decode_poliwag(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/poliwag.glb'))
    args=parser.parse_args();blob=export_poliwag();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    model=sculpture()
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; '+
          f'{sum(len(p["positions"])//3 for p in model["primitives"])} vertices; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__':main()
