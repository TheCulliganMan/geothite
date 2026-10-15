#!/usr/bin/env python3
"""Canonical Snorlax standard glTF skin and source-cue body response."""
import argparse
import hashlib
import math
from pathlib import Path
from snorlax_sculpt import sculpture,HEIGHT
from sculpt_geometry import sub
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
PARTS=tuple(p['part'] for p in sculpture()['primitives'])
MAX_BYTES=1024000
JOINTS=(('root',None,(0.,0.,0.)),('torso',0,(0.,.27,-.015)),
        ('head',1,(0.,.93,.05)),('arm_left',1,(-.35,.85,0.)),
        ('hand_left',3,(-.75,.62,.13)),('arm_right',1,(.35,.85,0.)),
        ('hand_right',5,(.75,.62,.13)),('ear_left',2,(-.26,1.19,0.)),
        ('ear_right',2,(.26,1.19,0.)))
CLIPS=(('idle',4.4,89),('attack',1.,65),('hit',1.,65))


def torso_weights(position):
    x,y,z=position;flex=smoothstep(.08,.36,y)
    return [0,1,0,0],[1.-flex,flex,0.,0.]


def skin_weights(part,position):
    x,y,z=position
    if part.startswith(('Foot /','Foot pad /','Foot claw /')):return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Head /','Face /','Eye /','Mouth /','Fang /')):return [2,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Ear /'):
        joint=7 if x<0 else 8;flex=smoothstep(1.19,1.30,y)
        return [2,joint,0,0],[1.-flex,flex,0.,0.]
    if part.startswith('Arm /'):
        joint=3 if x<0 else 5;arm=smoothstep(.34,.51,abs(x));wrist=smoothstep(.65,.75,abs(x))
        return [1,joint,joint+1,0],[1.-arm,arm*(1.-wrist),arm*wrist,0.]
    if part.startswith(('Hand /','Hand claw /')):return [4 if x<0 else 6,0,0,0],[1.,0.,0.,0.]
    return torso_weights(position)


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Snorlax clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(p):return math.sin(math.tau*t-p)+math.sin(p)
    if clip=='idle':
        return [(0.,0.,0.),(-.008*breath,.008*wave,.006*wave),
                (.012*breath,.012*lag(.4),-.006*wave),
                (.013*lag(.3),0.,-.015*breath),(.018*lag(.7),0.,.010*breath),
                (.013*lag(.5),0.,.015*breath),(.018*lag(.9),0.,-.010*breath),
                (0.,0.,.014*lag(.5)),(0.,0.,-.014*lag(.7))]
    if clip=='attack':
        brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
        trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
        return [(0.,0.,0.),(.034*brace,0.,0.),(.026*brace,.015*brace,0.),
                (.090*brace,0.,-.050*brace),(.060*trail,.012*trail,.025*trail),
                (.082*brace,0.,.050*brace),(.055*trail,-.012*trail,-.025*trail),
                (0.,0.,-.023*trail),(0.,0.,.023*trail)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.50,.23),(.78,-.12),(1.,0.)))
    return [(0.,0.,0.),(-.025*recoil,.010*recoil,0.),(-.031*recoil,0.,.008*recoil),
            (-.045*recoil,.012*recoil,.028*recoil),(-.035*trail,0.,-.021*trail),
            (-.042*recoil,-.012*recoil,-.028*recoil),(-.031*trail,0.,.021*trail),
            (0.,0.,.021*trail),(0.,0.,-.021*trail)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_snorlax(model=None):
    canonical=sculpture()
    if model is None:model=canonical
    if (model.get('name')!='snorlax' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Snorlax anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Snorlax sleeping bear body and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'snorlax/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'snorlax','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'snorlax','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'snorlax','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'snorlax/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'snorlax','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'snorlax.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'sleepy breathing survey; early torso and arm brace; delayed paws and ears settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'snorlax',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/snorlax_sculpt.py',
        'animation_contract':'identity floor root; planted soles; fitted rigid sleeping face; articulated torso, head, arms, paws and ears; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_snorlax(blob):
    if len(blob)>MAX_BYTES or blob!=export_snorlax():raise ValueError('noncanonical Snorlax GLB')
    return sculpture()


def read_snorlax(path):
    with Path(path).open('rb') as source:return decode_snorlax(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/snorlax.glb'))
    args=parser.parse_args();blob=export_snorlax();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    model=sculpture()
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; '+
          f'{sum(len(p["positions"])//3 for p in model["primitives"])} vertices; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__':main()
