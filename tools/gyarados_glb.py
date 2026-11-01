#!/usr/bin/env python3
"""Canonical Gyarados skin with grounded coil, neck, jaw, barbels and tail."""
import argparse
import hashlib
import math
from pathlib import Path
from gyarados_sculpt import sculpture,HEIGHT
from sculpt_geometry import sub
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

PARTS=tuple(p['part'] for p in sculpture()['primitives'])
MAX_BYTES=800000
JOINTS=(('root',None,(0.,0.,0.)),
        ('neck_base',0,(0.,.44,.20)),('neck_upper',1,(0.,.72,.20)),
        ('head',2,(0.,1.03,.27)),('lower_jaw',3,(0.,.73,.33)),
        ('barbel_left',3,(-.14,.86,.42)),('barbel_right',3,(.14,.86,.42)),
        ('tail_base',0,(.25,.18,-.50)),('tail_tip',7,(.50,.27,-.55)))
CLIPS=(('idle',4.0,81),('attack',1.,65),('hit',1.,65))


def body_weights(position):
    x,y,z=position
    active=smoothstep(.11,.34,y)
    head=smoothstep(.83,.98,y)
    upper=(1.-head)*smoothstep(.58,.76,y)
    neck=1.-head-upper
    tail=smoothstep(.09,.40,x)*(1.-smoothstep(-.20,.01,z))
    tip=smoothstep(.38,.64,x)
    if tail>0.:
        # The lower coil remains floor rooted; tail sweeps around vertical
        # axes so the rear tip can move without dipping below its contact plane.
        return [0,7,8,0],[1.-tail,tail*(1.-tip),tail*tip,0.]
    return [0,1,2,3],[1.-active,active*neck,active*upper,active*head]


def skin_weights(part,position):
    x,y,z=position
    if part.startswith(('Head /','Eye /','Crest /','Cheek fin /','Mouth /')):
        return [3,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Jaw /','Tongue /','Fang / lower')):
        return [4,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Fang / upper') or part=='Lip / cream upper muzzle':
        return [3,0,0,0],[1.,0.,0.,0.]
    if part=='Lip / continuous cream open-mouth rim':
        jaw=1.-smoothstep(.76,.96,y)
        return [3,4,0,0],[1.-jaw,jaw,0.,0.]
    if part.startswith('Barbel /'):
        return [5 if part.endswith('-1') else 6,0,0,0],[1.,0.,0.,0.]
    return body_weights(position)


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Gyarados clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(phase):return math.sin(math.tau*t-phase)+math.sin(phase)
    if clip=='idle':
        return [(0.,0.,0.),(.008*breath,.012*wave,0.),
                (-.012*breath,.018*wave,.006*wave),(.012*breath,.025*wave,-.006*wave),
                (.012*breath,0.,0.),
                (.020*lag(.4),0.,.028*lag(.5)),(.018*lag(.6),0.,-.025*lag(.7)),
                (0.,.032*wave,0.),(0.,.047*lag(.7),0.)]
    if clip=='attack':
        brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
        jaw=envelope(t,((0.,0.),(.12,1.),(.32,.75),(.60,.20),(1.,0.)))
        trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.14),(1.,0.)))
        return [(0.,0.,0.),(.028*brace,0.,0.),(.038*brace,0.,0.),
                (.045*brace,.015*brace,0.),(.12*jaw,0.,0.),
                (-.055*trail,.014*trail,-.025*trail),(-.050*trail,-.014*trail,.025*trail),
                (0.,-.045*brace,0.),(0.,.065*trail,0.)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.50,.23),(.78,-.12),(1.,0.)))
    return [(0.,0.,0.),(-.022*recoil,.006*recoil,0.),
            (-.035*recoil,-.010*recoil,.008*recoil),(-.047*recoil,-.020*recoil,-.008*recoil),
            (.08*max(0.,recoil),0.,0.),
            (.060*trail,0.,.030*trail),(.055*trail,0.,-.030*trail),
            (0.,.040*recoil,0.),(0.,-.060*trail,0.)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_gyarados(model=None):
    canonical=sculpture()
    if model is None:model=canonical
    if (model.get('name')!='gyarados' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Gyarados anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Gyarados coiled sea serpent and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'gyarados/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'gyarados','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'gyarados','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'gyarados','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'gyarados/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'gyarados','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'gyarados.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'grounded coil survey; early neck and jaw brace; delayed barbel and tail settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'gyarados',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/gyarados_sculpt.py',
        'animation_contract':'identity floor root; grounded coil; rigid eyes and crest; fitted jaw hinge; independent barbels and tail chain; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_gyarados(blob):
    if len(blob)>MAX_BYTES or blob!=export_gyarados():raise ValueError('noncanonical Gyarados GLB')
    return sculpture()


def read_gyarados(path):
    with Path(path).open('rb') as source:return decode_gyarados(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/gyarados.glb'))
    args=parser.parse_args();blob=export_gyarados();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; neutral {geometry_digest(sculpture())}')


if __name__=='__main__':main()
