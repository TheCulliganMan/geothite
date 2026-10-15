#!/usr/bin/env python3
"""Canonical Pidgey standard glTF skin and source-cue body response."""
import argparse
import hashlib
import math
from pathlib import Path
from pidgey_sculpt import sculpture,HEIGHT
from sculpt_geometry import sub
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
PARTS=tuple(p['part'] for p in sculpture()['primitives'])
MAX_BYTES=1024000
JOINTS=(('root',None,(0.,0.,0.)),('torso',0,(0.,.26,-.04)),('head',1,(0.,.56,.07)),
        ('lower_beak',2,(0.,.59,.23)),('wing_left',1,(-.18,.43,-.02)),
        ('flight_left',4,(-.25,.25,-.14)),('wing_right',1,(.18,.43,-.02)),
        ('flight_right',6,(.25,.25,-.14)),('tail_fan',1,(0.,.26,-.18)),('crown',2,(0.,.74,.03)))
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))


def skin_weights(part,position):
    x,y,z=position;left=x<0
    if part.startswith(('Foot /','Toe /','Rear toe /','Claw /')):return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Leg /'):
        flex=smoothstep(.05,.18,y);return [0,1,0,0],[1.-flex,flex,0.,0.]
    if part.startswith(('Head /','Face /','Cheek /','Eye /','Pupil /','Brow /','Upper beak /')):return [2,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Lower beak /'):return [3,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Crown /'):return [9,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Wing /'):return [4 if left else 6,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Flight feather /'):return [5 if left else 7,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Tail /'):return [8,0,0,0],[1.,0.,0.,0.]
    return [1,0,0,0],[1.,0.,0.,0.]


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Pidgey clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    if clip=='idle':return [(0.,0.,0.),(.010*breath,0.,.006*wave),(-.018*breath,.047*wave,.009*wave),
        (.012*breath,0.,0.),(.018*wave,0.,-.010*breath),(.012*wave,0.,-.007*breath),
        (.018*wave,0.,.010*breath),(.012*wave,0.,.007*breath),(.025*wave,.010*wave,0.),(.020*wave,0.,0.)]
    brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
    if clip=='attack':return [(0.,0.,0.),(.033*brace,0.,0.),(.047*brace,.012*brace,0.),(.095*brace,0.,0.),
        (.025*brace,.016*brace,-.095*brace),(.021*trail,0.,-.054*trail),
        (.025*brace,-.016*brace,.095*brace),(.021*trail,0.,.054*trail),(.058*trail,0.,0.),(.047*trail,0.,0.)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    return [(0.,0.,0.),(-.024*recoil,0.,0.),(-.048*recoil,0.,.015*recoil),(.055*recoil,0.,0.),
        (-.020*recoil,0.,-.050*recoil),(-.015*trail,0.,-.028*trail),
        (-.020*recoil,0.,.050*recoil),(-.015*trail,0.,.028*trail),(-.043*trail,0.,0.),(-.031*trail,0.,0.)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_pidgey(model=None):
    canonical=sculpture()
    if model is None:model=canonical
    if (model.get('name')!='pidgey' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Pidgey anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Pidgey tiny bird body and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'pidgey/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'pidgey','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'pidgey','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'pidgey','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'pidgey/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'pidgey','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'pidgey.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'alert bird survey; early torso and head brace; delayed folded feathers settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'pidgey',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/pidgey_sculpt.py',
        'animation_contract':'identity floor root; fixed pads and digits; rigid fitted face; articulated torso, head, lower bill, wings, flight feathers, tail fan and short crest; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_pidgey(blob):
    if len(blob)>MAX_BYTES or blob!=export_pidgey():raise ValueError('noncanonical Pidgey GLB')
    return sculpture()


def read_pidgey(path):
    with Path(path).open('rb') as source:return decode_pidgey(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/pidgey.glb'))
    args=parser.parse_args();blob=export_pidgey();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    model=sculpture()
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; '+
          f'{sum(len(p["positions"])//3 for p in model["primitives"])} vertices; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__':main()
