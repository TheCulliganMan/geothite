#!/usr/bin/env python3
"""Canonical Wooper skin, preserving the existing axolotl sculpture.

Rotation-only body clips follow production battle cues. Grounded soles never
move; the head surveys while the two external gills and paddle settle behind it.
"""
import argparse
import hashlib
import math
from pathlib import Path
from axolotl_sculpt import sculpture
from sculpt_geometry import sub
from skin_glb import COORDINATES, Glb, envelope, packed, quantized_weights, quaternion_xyz, smoothstep

HEIGHT=.70
PARTS=tuple(p['part'] for p in sculpture('wooper')['primitives'])
MAX_BYTES=500000
JOINTS=(('root',None,(0.,0.,0.)),
        ('torso',0,(0.,.23,0.)),('head',1,(0.,.46,.02)),
        ('leg_left',0,(-.12,.10,.02)),('leg_right',0,(.12,.10,.02)),
        ('gill_left',2,(-.23,.55,.02)),('gill_right',2,(.23,.55,.02)),
        ('tail_base',1,(0.,.15,-.12)),('tail_tip',7,(0.,.22,-.39)))
CLIPS=(('idle',3.6,73),('attack',1.,65),('hit',1.,65))


def body_weights(position):
    x,y,z=position
    active=smoothstep(.075,.205,y)
    head=smoothstep(.36,.48,y)
    return [0,1,2,0],[1.-active,active*(1.-head),active*head,0.]


def skin_weights(part,position):
    x,y,z=position
    if part.startswith('Foot /'):return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Leg /'):
        flex=smoothstep(.075,.15,y)
        return [0,3 if part.endswith('-1') else 4,0,0],[1.-flex,flex,0.,0.]
    if part.startswith(('Eye /','Mouth /')):return [2,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Gill /','Gill branch /')):
        # Both branches and stalk share the same rigid hinge: no separation.
        side=part.split(' / ')[1].split()[0]
        return [5 if side=='-1' else 6,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Tail /'):
        tip=smoothstep(-.26,-.50,z)
        return [7,8,0,0],[1.-tip,tip,0.,0.]
    return body_weights(position)


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Wooper clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(phase):return math.sin(math.tau*t-phase)+math.sin(phase)
    if clip=='idle':
        return [(0.,0.,0.),(.009*breath,.014*wave,0.),
                (-.014*breath,.035*wave,.012*wave),
                (.007*breath,0.,0.),(.007*breath,0.,0.),
                (.020*lag(.4),.028*lag(.5),.025*lag(.4)),
                (.020*lag(.6),-.028*lag(.7),-.025*lag(.6)),
                (.015*wave,.028*wave,0.),(.026*lag(.7),.048*lag(.9),0.)]
    if clip=='attack':
        brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
        trail=envelope(t,((0.,0.),(.28,1.),(.50,.64),(.78,-.14),(1.,0.)))
        return [(0.,0.,0.),(.045*brace,0.,0.),(.082*brace,.018*brace,0.),
                (-.016*brace,0.,0.),(-.016*brace,0.,0.),
                (-.048*trail,0.,-.038*trail),(-.043*trail,0.,.035*trail),
                (-.085*brace,-.023*brace,0.),(-.110*trail,.042*trail,0.)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.50,.23),(.78,-.12),(1.,0.)))
    return [(0.,0.,0.),(-.035*recoil,.006*recoil,0.),
            (-.060*recoil,-.020*recoil,.012*recoil),
            (.014*recoil,0.,0.),(.014*recoil,0.,0.),
            (.054*trail,0.,.045*trail),(.050*trail,0.,-.040*trail),
            (.038*recoil,.023*recoil,0.),(.060*trail,-.038*trail,0.)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_wooper(model=None):
    canonical=sculpture('wooper')
    if model is None:model=canonical
    if (model.get('name') not in ('wooper','wooper') or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Wooper anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Wooper axolotl sculpture and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'wooper/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'wooper','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'wooper','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'wooper','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'wooper/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'wooper','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'wooper.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'planted curious survey; early head brace; delayed gill and paddle settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'wooper',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/axolotl_sculpt.py',
        'animation_contract':'identity floor root; fixed soles; fitted rigid face; independent gills and paddle chain; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_wooper(blob):
    if len(blob)>MAX_BYTES or blob!=export_wooper():raise ValueError('noncanonical Wooper GLB')
    return sculpture('wooper')


def read_wooper(path):
    with Path(path).open('rb') as source:return decode_wooper(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/wooper.glb'))
    args=parser.parse_args();blob=export_wooper();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; neutral {geometry_digest(sculpture("wooper"))}')


if __name__=='__main__':main()
