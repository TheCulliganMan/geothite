#!/usr/bin/env python3
"""Canonical Squirtle standard glTF skin and source-cue body response."""
import argparse
import hashlib
import math
from pathlib import Path
from squirtle_sculpt import sculpture,HEIGHT
from sculpt_geometry import sub
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
PARTS=tuple(p['part'] for p in sculpture()['primitives'])
MAX_BYTES=1024000
JOINTS=(('root',None,(0.,0.,0.)),('torso',0,(0.,.22,.015)),
        ('head',1,(0.,.62,.075)),('arm_left',1,(-.19,.53,.04)),
        ('arm_right',1,(.19,.53,.04)),('leg_left',1,(-.15,.19,.02)),
        ('leg_right',1,(.15,.19,.02)),('tail_base',1,(0.,.22,-.16)),
        ('tail_curl',7,(.25,.27,-.42)))
CLIPS=(('idle',4.,81),('attack',1.,65),('hit',1.,65))


def torso_weights(position):
    x,y,z=position;flex=smoothstep(.06,.20,y)
    return [0,1,0,0],[1.-flex,flex,0.,0.]


def skin_weights(part,position):
    x,y,z=position
    if part.startswith(('Foot /','Toe /')):return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Head /','Eye /','Eye glint /','Iris /','Pupil /','Mouth /','Tongue /','Nostril /')):
        return [2,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Shell /','Shell rim /','Shell seam /','Belly /','Belly seam /')):
        return [1,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Arm /'):
        joint=3 if x<0 else 4;arm=smoothstep(.18,.27,abs(x))
        return [1,joint,0,0],[1.-arm,arm,0.,0.]
    if part.startswith(('Hand /','Finger /')):return [3 if x<0 else 4,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Haunch /'):
        joint=5 if x<0 else 6;lift=smoothstep(.04,.16,y);shoulder=smoothstep(.23,.32,y)
        return [0,joint,1,0],[1.-lift,lift*(1.-shoulder),lift*shoulder,0.]
    if part.startswith(('Tail /','Tail core /')):
        curl=smoothstep(.13,.28,x);return [7,8,0,0],[1.-curl,curl,0.,0.]
    return torso_weights(position)


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Squirtle clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(p):return math.sin(math.tau*t-p)+math.sin(p)
    if clip=='idle':
        return [(0.,0.,0.),(.007*breath,.012*wave,.007*wave),
                (-.010*breath,.030*wave,-.007*wave),
                (.022*lag(.3),0.,-.015*breath),(.021*lag(.6),0.,.015*breath),
                (-.008*breath,0.,.006*wave),(-.008*breath,0.,-.006*wave),
                (.009*breath,.027*wave,0.),(-.012*breath,.042*lag(.6),.014*lag(.8))]
    if clip=='attack':
        brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
        trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
        return [(0.,0.,0.),(.036*brace,0.,0.),(.042*brace,.014*brace,0.),
                (.080*brace,0.,-.043*brace),(.077*brace,0.,.043*brace),
                (-.021*brace,0.,-.010*brace),(-.021*brace,0.,.010*brace),
                (.017*brace,.040*brace,0.),(-.024*trail,.062*trail,.020*trail)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.50,.23),(.78,-.12),(1.,0.)))
    return [(0.,0.,0.),(-.025*recoil,.010*recoil,0.),(-.035*recoil,0.,.008*recoil),
            (-.050*recoil,.012*recoil,.033*recoil),(-.047*recoil,-.012*recoil,-.033*recoil),
            (.016*recoil,0.,.008*recoil),(.016*recoil,0.,-.008*recoil),
            (-.014*recoil,-.032*recoil,0.),(.020*trail,-.048*trail,-.015*trail)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_squirtle(model=None):
    canonical=sculpture()
    if model is None:model=canonical
    if (model.get('name')!='squirtle' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Squirtle anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Squirtle plated turtle body and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'squirtle/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'squirtle','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'squirtle','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'squirtle','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'squirtle/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'squirtle','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'squirtle.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'turtle survey; early torso and arm brace; delayed solid spiral tail settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'squirtle',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/squirtle_sculpt.py',
        'animation_contract':'identity floor root; planted turtle pads and toes; fitted rigid face; coherent rigid shell and plates; articulated head, arms, haunches and solid spiral tail; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_squirtle(blob):
    if len(blob)>MAX_BYTES or blob!=export_squirtle():raise ValueError('noncanonical Squirtle GLB')
    return sculpture()


def read_squirtle(path):
    with Path(path).open('rb') as source:return decode_squirtle(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/squirtle.glb'))
    args=parser.parse_args();blob=export_squirtle();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    model=sculpture()
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; '+
          f'{sum(len(p["positions"])//3 for p in model["primitives"])} vertices; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__':main()
