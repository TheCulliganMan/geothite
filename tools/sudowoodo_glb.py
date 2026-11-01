#!/usr/bin/env python3
"""Canonical Sudowoodo standard glTF skin and source-cue body response."""
import argparse
import hashlib
import math
from pathlib import Path
from sudowoodo_sculpt import sculpture,HEIGHT,CROWN
from sculpt_geometry import sub
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
PARTS=tuple(p['part'] for p in sculpture()['primitives'])
MAX_BYTES=650000
JOINTS=(('root',None,(0.,0.,0.)),('torso',0,(0.,.32,.0)),('head',1,(0.,.63,.04)),
        ('arm_left',1,(-.09,.55,.032)),('hand_left',3,(-.39,.73,.036)),
        ('arm_right',1,(.09,.55,.032)),('hand_right',5,(.39,.73,.036)),
        ('crown_base',2,(0.,.835,.032)),('crown_right_fork',7,(-.008,.90,.03)))
CLIPS=(('idle',3.8,77),('attack',1.,65),('hit',1.,65))


def body_weights(position):
    x,y,z=position;active=smoothstep(.075,.25,y);head=smoothstep(.56,.66,y)
    return [0,1,2,0],[1.-active,active*(1.-head),active*head,0.]


def skin_weights(part,position):
    x,y,z=position
    if part.startswith('Foot /'):return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Eye /','Smile /')):return [2,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Arm /'):
        arm=3 if part.endswith('-1') else 5
        shoulder=smoothstep(.10,.24,abs(x));wrist=smoothstep(.29,.37,abs(x))
        return [1,arm,arm+1,0],[1.-shoulder,shoulder*(1.-wrist),shoulder*wrist,0.]
    if part.startswith(('Finger stalk /','Finger orb /')):
        sign=part.split(' / ')[1].split()[0]
        return [4 if sign=='-1' else 6,0,0,0],[1.,0.,0.,0.]
    if part==CROWN:
        flex=smoothstep(.84,.92,y);return [2,7,0,0],[1.-flex,flex,0.,0.]
    if part=='Crown / upward right fork':
        flex=smoothstep(.91,.985,y);return [7,8,0,0],[1.-flex,flex,0.,0.]
    if part.startswith('Crown / pale'):
        return [7 if part.endswith('-1') else 8,0,0,0],[1.,0.,0.,0.]
    return body_weights(position)


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Sudowoodo clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(p):return math.sin(math.tau*t-p)+math.sin(p)
    if clip=='idle':
        return [(0.,0.,0.),(.006*breath,.009*wave,.012*wave),
                (-.009*breath,.027*wave,-.008*wave),
                (.015*lag(.3),0.,.025*wave),(.020*lag(.7),.015*wave,-.025*lag(.6)),
                (.014*lag(.5),0.,.025*wave),(.018*lag(.9),-.015*wave,-.025*lag(.8)),
                (.014*lag(.4),0.,.024*lag(.6)),(.021*lag(.8),0.,-.027*lag(.9))]
    if clip=='attack':
        brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
        trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.14),(1.,0.)))
        return [(0.,0.,0.),(.044*brace,0.,0.),(.051*brace,.015*brace,0.),
                (.055*brace,-.030*brace,-.13*brace),(.082*trail,.022*trail,.08*trail),
                (.048*brace,.030*brace,.12*brace),(.075*trail,-.022*trail,-.073*trail),
                (-.040*trail,0.,-.023*trail),(-.060*trail,0.,.040*trail)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.50,.23),(.78,-.12),(1.,0.)))
    return [(0.,0.,0.),(-.035*recoil,.005*recoil,.008*recoil),
            (-.054*recoil,-.014*recoil,-.010*recoil),
            (-.040*recoil,.021*recoil,.09*recoil),(-.065*trail,0.,-.06*trail),
            (-.037*recoil,-.021*recoil,-.083*recoil),(-.060*trail,0.,.055*trail),
            (.054*trail,0.,.030*trail),(.071*trail,0.,-.041*trail)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_sudowoodo(model=None):
    canonical=sculpture()
    if model is None:model=canonical
    if (model.get('name')!='sudowoodo' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Sudowoodo anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Sudowoodo stone trunk and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'sudowoodo/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'sudowoodo','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'sudowoodo','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'sudowoodo','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'sudowoodo/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'sudowoodo','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'sudowoodo.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'planted imitation-tree survey; early branch-arm brace; delayed fingers and crown settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'sudowoodo',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/sudowoodo_sculpt.py',
        'animation_contract':'identity floor root; fixed root pads; fitted rigid face; articulated branch arms, fingers and forked crown; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_sudowoodo(blob):
    if len(blob)>MAX_BYTES or blob!=export_sudowoodo():raise ValueError('noncanonical Sudowoodo GLB')
    return sculpture()


def read_sudowoodo(path):
    with Path(path).open('rb') as source:return decode_sudowoodo(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/sudowoodo.glb'))
    args=parser.parse_args();blob=export_sudowoodo();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    model=sculpture()
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; '+
          f'{sum(len(p["positions"])//3 for p in model["primitives"])} vertices; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__':main()
