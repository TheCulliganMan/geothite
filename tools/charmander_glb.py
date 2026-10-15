#!/usr/bin/env python3
"""Canonical Charmander standard glTF skin and source-cue body response."""
import argparse
import hashlib
import math
from pathlib import Path
from charmander_sculpt import sculpture,HEIGHT
from sculpt_geometry import sub
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
PARTS=tuple(p['part'] for p in sculpture()['primitives'])
MAX_BYTES=1024000
JOINTS=(('root',None,(0.,0.,0.)),('torso',0,(0.,.20,0.)),
        ('head',1,(0.,.61,.08)),('arm_left',1,(-.17,.55,.05)),
        ('arm_right',1,(.17,.55,.05)),('leg_left',1,(-.15,.20,.01)),
        ('leg_right',1,(.15,.20,.01)),('tail_base',1,(0.,.23,-.14)),
        ('tail_mid',7,(.17,.21,-.37)),('tail_tip',8,(.33,.37,-.55)),
        ('flame',9,(.37,.46,-.59)))
CLIPS=(('idle',3.8,81),('attack',1.,65),('hit',1.,65))


def torso_weights(position):
    x,y,z=position;flex=smoothstep(.07,.22,y)
    return [0,1,0,0],[1.-flex,flex,0.,0.]


def skin_weights(part,position):
    x,y,z=position
    if part.startswith(('Foot /','Foot claw /')):return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Head /','Eye /','Eye glint /','Iris /','Pupil /','Mouth /','Tongue /','Nostril /','Fang /')):
        return [2,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Arm /'):
        joint=3 if x<0 else 4;arm=smoothstep(.15,.23,abs(x))
        return [1,joint,0,0],[1.-arm,arm,0.,0.]
    if part.startswith(('Hand /','Finger /')):return [3 if x<0 else 4,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Haunch /'):
        joint=5 if x<0 else 6;lift=smoothstep(.04,.16,y);shoulder=smoothstep(.23,.32,y)
        return [0,joint,1,0],[1.-lift,lift*(1.-shoulder),lift*shoulder,0.]
    if part.startswith('Tail /'):
        middle=smoothstep(.20,.38,-z);tip=smoothstep(.43,.57,-z)
        return [7,8,9,0],[1.-middle,middle*(1.-tip),middle*tip,0.]
    if part.startswith('Flame /'):
        flex=smoothstep(.43,.56,y);return [9,10,0,0],[1.-flex,flex,0.,0.]
    return torso_weights(position)


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Charmander clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(p):return math.sin(math.tau*t-p)+math.sin(p)
    if clip=='idle':
        return [(0.,0.,0.),(.008*breath,.012*wave,.008*wave),
                (-.010*breath,.031*wave,-.008*wave),
                (.024*lag(.3),0.,-.016*breath),(.023*lag(.6),0.,.016*breath),
                (-.009*breath,0.,.007*wave),(-.009*breath,0.,-.007*wave),
                (.010*breath,.020*wave,0.),(-.013*breath,.034*lag(.4),0.),
                (.017*breath,.048*lag(.8),0.),(.035*lag(.7),.023*wave,.055*lag(.9))]
    if clip=='attack':
        brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
        trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
        return [(0.,0.,0.),(.043*brace,0.,0.),(.044*brace,.016*brace,0.),
                (.092*brace,0.,-.045*brace),(.088*brace,0.,.045*brace),
                (-.024*brace,0.,-.010*brace),(-.024*brace,0.,.010*brace),
                (.025*brace,.032*brace,0.),(-.034*trail,.052*trail,0.),
                (.045*trail,.068*trail,0.),(-.050*trail,0.,.070*trail)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.50,.23),(.78,-.12),(1.,0.)))
    return [(0.,0.,0.),(-.028*recoil,.010*recoil,0.),(-.038*recoil,0.,.009*recoil),
            (-.059*recoil,.014*recoil,.035*recoil),(-.055*recoil,-.014*recoil,-.035*recoil),
            (.018*recoil,0.,.009*recoil),(.018*recoil,0.,-.009*recoil),
            (-.017*recoil,-.025*recoil,0.),(.025*trail,-.042*trail,0.),
            (-.033*trail,-.055*trail,0.),(.041*trail,0.,-.062*trail)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_charmander(model=None):
    canonical=sculpture()
    if model is None:model=canonical
    if (model.get('name')!='charmander' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Charmander anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Charmander flame lizard body and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'charmander/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'charmander','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'charmander','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'charmander','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'charmander/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'charmander','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'charmander.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'lizard survey; early torso and arm brace; delayed curved tail and folded fire settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'charmander',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/charmander_sculpt.py',
        'animation_contract':'identity floor root; planted pads and claws; fitted rigid face; articulated torso, head, arms, haunches, curved tail and folded fire; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_charmander(blob):
    if len(blob)>MAX_BYTES or blob!=export_charmander():raise ValueError('noncanonical Charmander GLB')
    return sculpture()


def read_charmander(path):
    with Path(path).open('rb') as source:return decode_charmander(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/charmander.glb'))
    args=parser.parse_args();blob=export_charmander();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    model=sculpture()
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; '+
          f'{sum(len(p["positions"])//3 for p in model["primitives"])} vertices; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__':main()
