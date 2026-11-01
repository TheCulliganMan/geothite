#!/usr/bin/env python3
"""Canonical Clefairy standard glTF skin and source-cue body response."""
import argparse
import hashlib
import math
from pathlib import Path
from clefairy_sculpt import sculpture as neutral_sculpture,HEIGHT
from sculpt_geometry import sub
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
def sculpture():
    model=neutral_sculpture();model['name']='clefairy';return model

_MODEL=sculpture()
PARTS=tuple(p['part'] for p in _MODEL['primitives'])
MAX_BYTES=1024000
_body=next(p for p in _MODEL['primitives'] if p['part'].startswith('Body /'))
_scale=(max(_body['positions'][1::3])-min(_body['positions'][1::3]))/(.76-.05)
_offset=min(_body['positions'][1::3])-.05*_scale

def normalized(point):return tuple(v*_scale+(_offset if i==1 else 0.) for i,v in enumerate(point))
JOINTS=(('root',None,(0.,0.,0.)),('body',0,normalized((0.,.38,0.))),
        ('arm_left',1,normalized((-.24,.386,.035))),('arm_right',1,normalized((.24,.386,.035))),
        ('ear_left',1,normalized((-.16,.60,.06))),('ear_right',1,normalized((.16,.60,.06))),
        ('wing_left',1,normalized((-.225,.37,-.162))),('wing_right',1,normalized((.225,.37,-.162))),
        ('forelock',1,normalized((0.,.699,.120))),('tail_base',1,normalized((0.,.25,-.16))),
        ('tail_curl',9,normalized((.15,.29,-.34))))
CLIPS=(('idle',3.8,85),('attack',1.,65),('hit',1.,65))


def skin_weights(part,position):
    x,y,z=position;left=x<0
    if part.startswith(('Foot /','Toe /')):return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Arm /'):
        blend=smoothstep(.23*_scale,.30*_scale,abs(x));return [1,2 if left else 3,0,0],[1.-blend,blend,0.,0.]
    if part.startswith('Hand claw /'):return [2 if left else 3,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Ear /','Ear tip /')):return [4 if left else 5,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Wing /'):return [6 if left else 7,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Tuft /','Tuft crease /')):return [8,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Tail /'):
        rear=-z/_scale
        if rear<.25:
            blend=smoothstep(.16,.25,rear);return [1,9,0,0],[1.-blend,blend,0.,0.]
        blend=smoothstep(.27,.34,rear);return [9,10,0,0],[1.-blend,blend,0.,0.]
    return [1,0,0,0],[1.,0.,0.,0.]


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Clefairy clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(p):return math.sin(math.tau*t-p)+math.sin(p)
    if clip=='idle':return [(0.,0.,0.),(.008*breath,.014*wave,.005*wave),
        (.025*lag(.2),.009*wave,-.015*breath),(.022*lag(.5),-.009*wave,.015*breath),
        (.020*wave,0.,-.012*breath),(.018*lag(.3),0.,.012*breath),
        (.020*lag(.4),0.,-.030*wave),(.020*lag(.7),0.,.030*wave),
        (.012*lag(.5),.013*wave,0.),(.014*wave,.018*wave,0.),(.023*lag(.8),.025*lag(.8),0.)]
    brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
    if clip=='attack':return [(0.,0.,0.),(.032*brace,0.,0.),
        (.063*brace,.014*brace,-.027*brace),(.058*brace,-.014*brace,.027*brace),
        (.037*trail,0.,-.024*trail),(.032*trail,0.,.024*trail),
        (.042*trail,0.,-.054*trail),(.038*trail,0.,.054*trail),
        (.029*trail,0.,0.),(.024*brace,.025*brace,0.),(.045*trail,.037*trail,0.)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    return [(0.,0.,0.),(-.024*recoil,.008*recoil,0.),
        (-.041*recoil,0.,.025*recoil),(-.038*recoil,0.,-.025*recoil),
        (-.029*trail,0.,.025*trail),(-.026*trail,0.,-.025*trail),
        (-.036*trail,0.,.039*trail),(-.032*trail,0.,-.039*trail),
        (-.022*trail,0.,0.),(-.022*recoil,-.019*recoil,0.),(-.039*trail,-.028*trail,0.)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_clefairy(model=None):
    canonical=sculpture()
    if model is None:model=canonical
    if (model.get('name')!='clefairy' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Clefairy anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Clefairy shared fairy body and skin and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'clefairy/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'clefairy','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'clefairy','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'clefairy','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'clefairy/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'clefairy','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'clefairy.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'gentle fairy survey; early body and hand brace; delayed wings and curled tail settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'clefairy',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/clefairy_sculpt.py',
        'animation_contract':'identity floor root; planted feet and toenails; rigid fitted fairy face; articulated hands, ears, scalloped wings, forelock and curled tail; rotation-only source-cue clips; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_clefairy(blob):
    if len(blob)>MAX_BYTES or blob!=export_clefairy():raise ValueError('noncanonical Clefairy GLB')
    return sculpture()


def read_clefairy(path):
    with Path(path).open('rb') as source:return decode_clefairy(source.read(MAX_BYTES+1))


def main_export(directory):
    directory=Path(directory);directory.mkdir(parents=True,exist_ok=True)
    (directory/'icon_clefairy.glb').write_bytes(export_clefairy())
    (directory/'icon_clefairy.mesh.json').unlink(missing_ok=True)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/actor_props/icon_clefairy.glb'))
    args=parser.parse_args();blob=export_clefairy();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    model=sculpture()
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; '+
          f'{sum(len(p["positions"])//3 for p in model["primitives"])} vertices; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__':main()
