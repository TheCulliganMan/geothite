#!/usr/bin/env python3
"""Canonical Pikachu standard glTF skin and source-cue body response."""
import argparse
import hashlib
import math
from pathlib import Path
from pikachu_sculpt import sculpture,HEIGHT
from sculpt_geometry import sub
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
PARTS=tuple(p['part'] for p in sculpture()['primitives'])
MAX_BYTES=1024000
JOINTS=(('root',None,(0.,0.,0.)),('torso',0,(0.,.20,-.01)),
        ('head',1,(0.,.49,.02)),('arm_left',1,(-.14,.42,.04)),
        ('arm_right',1,(.14,.42,.04)),('ear_left',2,(-.11,.70,-.035)),
        ('ear_left_tip',5,(-.17,.87,-.038)),('ear_right',2,(.12,.70,-.04)),
        ('ear_right_tip',7,(.27,.82,-.065)),('tail_base',1,(0.,.20,-.14)),
        ('tail_tip',9,(.20,.45,-.21)))
CLIPS=(('idle',3.6,77),('attack',1.,65),('hit',1.,65))


def torso_weights(position):
    x,y,z=position;flex=smoothstep(.045,.18,y)
    return [0,1,0,0],[1.-flex,flex,0.,0.]


def skin_weights(part,position):
    x,y,z=position
    if part.startswith(('Foot /','Toe crease /')):return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Head /','Eye /','Eye glint /','Cheek /','Mouth /','Nose /')):return [2,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Ear /','Ear tip /')):
        left=part.endswith('-1');joint=5 if left else 7
        base=smoothstep(.68,.75,y);tip=smoothstep(.83,.92,y) if left else smoothstep(.78,.85,y)
        return [2,joint,joint+1,0],[1.-base,base*(1.-tip),base*tip,0.]
    if part.startswith('Arm /'):
        joint=3 if x<0 else 4;arm=smoothstep(.14,.195,abs(x))
        return [1,joint,0,0],[1.-arm,arm,0.,0.]
    if part.startswith(('Hand /','Finger /','Thumb /')):return [3 if x<0 else 4,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Tail /','Tail base /')):
        tip=smoothstep(.13,.29,x);return [9,10,0,0],[1.-tip,tip,0.,0.]
    return torso_weights(position)


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Pikachu clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(p):return math.sin(math.tau*t-p)+math.sin(p)
    if clip=='idle':
        return [(0.,0.,0.),(.007*breath,.015*wave,.009*wave),
                (-.009*breath,.036*lag(.3),-.009*wave),
                (.026*lag(.3),0.,-.020*breath),(.024*lag(.6),0.,.020*breath),
                (.025*lag(.3),0.,.022*wave),(.036*lag(.7),0.,-.025*lag(.6)),
                (.022*lag(.5),0.,-.021*wave),(.032*lag(.9),0.,.025*lag(.8)),
                (.012*breath,.044*wave,0.),(-.018*breath,.065*lag(.7),0.)]
    if clip=='attack':
        brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
        trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
        return [(0.,0.,0.),(.045*brace,0.,0.),(.036*brace,.018*brace,0.),
                (.110*brace,0.,-.055*brace),(.100*brace,0.,.055*brace),
                (-.042*trail,0.,-.038*trail),(-.065*trail,0.,.046*trail),
                (-.039*trail,0.,.034*trail),(-.062*trail,0.,-.042*trail),
                (.025*brace,.072*brace,0.),(-.038*trail,.098*trail,0.)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.50,.23),(.78,-.12),(1.,0.)))
    return [(0.,0.,0.),(-.032*recoil,.012*recoil,0.),(-.045*recoil,0.,.010*recoil),
            (-.068*recoil,.012*recoil,.045*recoil),(-.063*recoil,-.012*recoil,-.045*recoil),
            (.049*trail,0.,.027*trail),(.065*trail,0.,-.033*trail),
            (.045*trail,0.,-.027*trail),(.061*trail,0.,.033*trail),
            (-.016*recoil,-.060*recoil,0.),(.033*trail,-.075*trail,0.)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_pikachu(model=None):
    canonical=sculpture()
    if model is None:model=canonical
    if (model.get('name')!='pikachu' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Pikachu anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Pikachu mouse body and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'pikachu/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'pikachu','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'pikachu','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'pikachu','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'pikachu/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'pikachu','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'pikachu.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'mouse survey; early torso and paw brace; delayed tipped ears and bolt tail settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'pikachu',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/pikachu_sculpt.py',
        'animation_contract':'identity floor root; planted mouse pads; fitted rigid face; articulated torso, head, paws, tipped ears and bolt tail; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_pikachu(blob):
    if len(blob)>MAX_BYTES or blob!=export_pikachu():raise ValueError('noncanonical Pikachu GLB')
    return sculpture()


def read_pikachu(path):
    with Path(path).open('rb') as source:return decode_pikachu(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/pikachu.glb'))
    args=parser.parse_args();blob=export_pikachu();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    model=sculpture()
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; '+
          f'{sum(len(p["positions"])//3 for p in model["primitives"])} vertices; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__':main()
