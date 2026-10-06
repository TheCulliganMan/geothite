#!/usr/bin/env python3
"""Deterministic Chikorita sculpture, quadruped skin, and cue-relative clips.

The source recipe and standard-library glTF writer are sufficient to rebuild.
Blender is only a preview tool. The identity floor root never moves; individual
leg fields compress above fixed soles, the head carries its fitted face, and a
three-joint leaf chain flexes progressively behind head and chest motion.
"""
import argparse
import hashlib
import math
from pathlib import Path

from chikorita_sculpt import (BODY, BODY_HEIGHT, FULL_HEIGHT, LEAF,
                             STEM, TAIL, sculpt, sub)
from skin_glb import (COORDINATES, Glb, envelope, packed, quantized_weights,
                      quaternion_xyz, smoothstep)

FILE='chikorita.glb'
PARTS=(BODY,
       'Eye / left fitted tall ink','Eye / left paper-white crescent','Eye / left catchlight',
       'Eye / right fitted tall ink','Eye / right paper-white crescent','Eye / right catchlight',
       'Mouth / fitted subtle smiling seam')+tuple('Collar / seated bud '+str(i) for i in range(1,9))+(TAIL,STEM,LEAF)
MAX_BYTES=512000
JOINTS=(('root',None,(0.,0.,0.)),('torso',0,(0.,.170,-.020)),
        ('head',1,(0.,.405,.130)),
        ('foreleg_left',1,(-.174,.195,.174)),('foreleg_right',1,(.174,.195,.174)),
        ('hindleg_left',1,(-.174,.185,-.224)),('hindleg_right',1,(.174,.185,-.224)),
        ('tail_nub',1,(0.,.230,-.285)),
        ('leaf_petiole',2,(0.,.665,.160)),
        ('leaf_mid_fold',8,(.030,.895,.025)),
        ('leaf_tip_fold',9,(.119,.988,-.221)))
CLIPS=(('idle',3.4,73),('attack',1.,65),('hit',1.,65))


def body_weights(position):
    x,y,z=position
    # The entire facial surface and its fitted artwork are rigidly owned by
    # the head. A soft throat transition begins well below the mouth seam.
    upper=smoothstep(.038,.190,y)
    head=smoothstep(.330,.428,y)*smoothstep(-.090,.070,z)
    leg=(1.-smoothstep(.130,.270,y))*smoothstep(.080,.165,abs(x))
    legjoint=(3 if x<0 else 4) if z>-.015 else (5 if x<0 else 6)
    active=[(j,w) for j,w in ((0,1.-upper),(1,upper*(1.-head)*(1.-leg)),
                             (2,upper*head),(legjoint,upper*(1.-head)*leg)) if w>0.]
    return [j for j,_ in active]+[0]*(4-len(active)),[w for _,w in active]+[0.]*(4-len(active))


def skin_weights(part,position):
    if part==TAIL:return [7,0,0,0],[1.,0.,0.,0.]
    if part==STEM:return [8,0,0,0],[1.,0.,0.,0.]
    if part==LEAF:
        # Z is monotone along the authored curved blade, so its two blend
        # bands follow the complete width, thickness, and central crease.
        z=position[2];middle=smoothstep(.065,-.065,z);tip=smoothstep(-.155,-.285,z)
        active=[(j,w) for j,w in ((8,1.-middle),(9,middle*(1.-tip)),(10,middle*tip)) if w>0.]
        return [j for j,_ in active]+[0]*(4-len(active)),[w for _,w in active]+[0.]*(4-len(active))
    return body_weights(position)


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Chikorita clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    if clip=='idle':
        def lag(phase):return math.sin(math.tau*t-phase)+math.sin(phase)
        return [(0.,0.,0.),(.012*wave,0.,.006*math.sin(2*math.tau*t)),
                (-.042*breath,.065*wave,.014*wave),
                (-.018*breath,0.,.015*wave),(-.018*breath,0.,-.015*wave),
                (.012*breath,0.,.010*wave),(.012*breath,0.,-.010*wave),
                (.024*wave,.035*wave,0.),
                (.027*lag(.2),.012*wave,.022*wave),
                (.042*lag(.65),0.,-.020*lag(.45)),
                (.065*lag(1.1),.009*wave,.017*lag(.85))]
    if clip=='attack':
        brace=envelope(t,((0.,0.),(.12,-.22),(.34,1.),(.53,.66),(.77,.14),(1.,0.)))
        flex=envelope(t,((0.,0.),(.15,-.20),(.41,1.),(.66,-.16),(.87,.04),(1.,0.)))
        trail=envelope(t,((0.,0.),(.22,-.12),(.50,1.),(.73,-.23),(.91,.035),(1.,0.)))
        tip=envelope(t,((0.,0.),(.28,-.07),(.57,1.),(.80,-.18),(1.,0.)))
        return [(0.,0.,0.),(.115*brace,0.,0.),(.130*brace,.018*brace,0.),
                (-.090*brace,0.,-.020*brace),(-.090*brace,0.,.020*brace),
                (.060*brace,0.,-.015*brace),(.060*brace,0.,.015*brace),
                (-.065*flex,0.,0.),(-.085*flex,0.,-.022*flex),
                (-.105*trail,0.,.022*trail),(-.145*tip,0.,.030*tip)]
    recoil=envelope(t,((0.,0.),(.16,1.),(.38,.35),(.65,-.13),(.84,.025),(1.,0.)))
    lag=envelope(t,((0.,0.),(.24,1.),(.49,.23),(.73,-.12),(1.,0.)))
    tip=envelope(t,((0.,0.),(.31,1.),(.57,.10),(.80,-.08),(1.,0.)))
    return [(0.,0.,0.),(-.095*recoil,.012*recoil,.015*recoil),
            (-.135*recoil,-.028*recoil,-.019*recoil),
            (.105*recoil,0.,.025*recoil),(.095*recoil,0.,-.025*recoil),
            (-.080*recoil,0.,.022*recoil),(-.074*recoil,0.,-.022*recoil),
            (.085*lag,0.,.028*lag),(.115*lag,0.,.031*lag),
            (.130*tip,0.,-.023*tip),(.150*tip,0.,-.025*tip)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_chikorita(model=None):
    canonical=sculpt()
    if model is None:model=canonical
    if (model.get('name') not in ('battle_chikorita','chikorita') or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Chikorita anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite continuous paper Chikorita sculpture and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'chikorita/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'chikorita','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'chikorita','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'chikorita','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'chikorita/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'chikorita','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'chikorita.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'planted breathing and head survey; chest-driven brace; delayed leaf recoil; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'battle_chikorita',
        'source_sha256':geometry_digest(model),'reference_span':FULL_HEIGHT,
        'body_reference_height':BODY_HEIGHT,'full_silhouette_height':FULL_HEIGHT,
        'geometry_source':'tools/chikorita_sculpt.py',
        'animation_contract':'identity floor root; fixed soles; continuous quadruped body; fitted rigid face; three-joint leaf; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_chikorita(blob):
    if len(blob)>MAX_BYTES or blob!=export_chikorita():raise ValueError('noncanonical Chikorita GLB')
    return sculpt()


def read_chikorita(path):
    with Path(path).open('rb') as source:return decode_chikorita(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input',type=Path)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/actor_props/battle_chikorita.glb'))
    args=parser.parse_args();model=read_chikorita(args.input) if args.input else sculpt();blob=export_chikorita(model)
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(blob)
    print(f'{args.output}: {len(blob)} bytes; {len(model["primitives"])} parts; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; sha256 {hashlib.sha256(blob).hexdigest()}')


if __name__=='__main__':main()
