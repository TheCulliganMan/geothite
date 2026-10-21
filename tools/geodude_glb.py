#!/usr/bin/env python3
"""Canonical Geodude standard glTF skin and source-cue body response."""
import argparse
import hashlib
import math
from pathlib import Path
from geodude_sculpt import sculpture,HEIGHT
from sculpt_geometry import sub
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
PARTS=tuple(p['part'] for p in sculpture()['primitives'])
MAX_BYTES=1024000
_raw=sculpture(False)
_floor=min(v for p in _raw['primitives'] for v in p['positions'][1::3])
_scale=HEIGHT/(max(v for p in _raw['primitives'] for v in p['positions'][1::3])-_floor)
def normalized(point):return tuple((v-(_floor if i==1 else 0.))*_scale for i,v in enumerate(point))
JOINTS=tuple((name,parent,normalized(point)) for name,parent,point in
        (('root',None,(0.,_floor,0.)),('boulder',0,(0.,.28,-.015)),
         ('arm_left',1,(-.258,.270,.005)),('forearm_left',2,(-.510,.166,.032)),
         ('hand_left',3,(-.691,.222,.090)),('arm_right',1,(.258,.270,.005)),
         ('forearm_right',5,(.510,.166,.032)),('hand_right',6,(.691,.222,.090))))
CLIPS=(('idle',4.2,85),('attack',1.,65),('hit',1.,65))


def boulder_weights(position):
    x,y,z=position;flex=smoothstep(.035,.14,y)
    return [0,1,0,0],[1.-flex,flex,0.,0.]


def skin_weights(part,position):
    x,y,z=position;left=x<0;raw_x=abs(x)/_scale
    if part.startswith(('Brow /','Eye /','Eye glint /','Pupil /','Mouth /','Lip /')):
        return [1,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Upper arm /'):
        joint=2 if left else 5;arm=smoothstep(.28,.38,raw_x)
        return [1,joint,0,0],[1.-arm,arm,0.,0.]
    if part.startswith('Forearm /'):
        shoulder=2 if left else 5;forearm=shoulder+1
        bend=smoothstep(.50,.64,raw_x)
        return [shoulder,forearm,0,0],[1.-bend,bend,0.,0.]
    if part.startswith(('Hand /','Finger /','Thumb /')):return [4 if left else 7,0,0,0],[1.,0.,0.,0.]
    return boulder_weights(position)


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Geodude clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(p):return math.sin(math.tau*t-p)+math.sin(p)
    if clip=='idle':
        return [(0.,0.,0.),(.007*breath,.018*wave,.005*wave),
                (.018*lag(.3),.009*wave,-.012*breath),(-.020*lag(.5),0.,.016*wave),(.026*lag(.8),.013*wave,0.),
                (.017*lag(.6),-.009*wave,.012*breath),(-.018*lag(.8),0.,-.016*wave),(.025*lag(1.),-.013*wave,0.)]
    if clip=='attack':
        brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
        trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
        return [(0.,0.,0.),(.032*brace,0.,0.),
                (.095*brace,.018*brace,-.042*brace),(.073*brace,0.,-.032*brace),(.068*trail,.017*trail,.016*trail),
                (.090*brace,-.018*brace,.042*brace),(.068*brace,0.,.032*brace),(.061*trail,-.017*trail,-.016*trail)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.50,.23),(.78,-.12),(1.,0.)))
    return [(0.,0.,0.),(-.022*recoil,.010*recoil,0.),
            (-.050*recoil,0.,.032*recoil),(-.045*trail,.014*trail,.020*trail),(-.042*trail,0.,-.015*trail),
            (-.047*recoil,0.,-.032*recoil),(-.041*trail,-.014*trail,-.020*trail),(-.038*trail,0.,.015*trail)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_geodude(model=None):
    canonical=sculpture()
    if model is None:model=canonical
    if (model.get('name')!='geodude' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Geodude anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Geodude stone brawler body and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'geodude/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'geodude','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'geodude','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'geodude','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'geodude/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'geodude','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'geodude.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'rock survey; early boulder and arm brace; delayed open hands settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'geodude',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/geodude_sculpt.py',
        'animation_contract':'identity floor root; fixed lowest boulder contact; fitted recessed rigid rock face; articulated boulder, shoulders, forearms and open hands; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_geodude(blob):
    if len(blob)>MAX_BYTES or blob!=export_geodude():raise ValueError('noncanonical Geodude GLB')
    return sculpture()


def read_geodude(path):
    with Path(path).open('rb') as source:return decode_geodude(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/geodude.glb'))
    args=parser.parse_args();blob=export_geodude();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    model=sculpture()
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; '+
          f'{sum(len(p["positions"])//3 for p in model["primitives"])} vertices; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__':main()
