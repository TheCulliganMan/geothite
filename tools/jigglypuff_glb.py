#!/usr/bin/env python3
"""Canonical Jigglypuff standard glTF skin and source-cue body response."""
import argparse
import hashlib
import math
from pathlib import Path
from jigglypuff_sculpt import sculpture as neutral_sculpture,HEIGHT
from sculpt_geometry import sub
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
def sculpture():
    model=neutral_sculpture();model['name']='jigglypuff';return model

_MODEL=sculpture()
PARTS=tuple(p['part'] for p in _MODEL['primitives'])
MAX_BYTES=1024000
_body=next(p for p in _MODEL['primitives'] if p['part'].startswith('Body /'))
_half=.330*math.cos(math.pi*.025/(28+.05))
_scale=(max(_body['positions'][1::3])-min(_body['positions'][1::3]))/(2*_half)
_offset=min(_body['positions'][1::3])-(.405-_half)*_scale

def normalized(point):return tuple(v*_scale+(_offset if i==1 else 0.) for i,v in enumerate(point))
JOINTS=(('root',None,(0.,0.,0.)),('body',0,normalized((0.,.405,0.))),
        ('arm_left',1,normalized((-.25,.359,.027))),('arm_right',1,normalized((.25,.359,.027))),
        ('ear_left',1,normalized((-.18,.62,.01))),('ear_right',1,normalized((.18,.62,.01))),
        ('forelock',1,normalized((.024,.697,.130))))
CLIPS=(('idle',3.6,85),('attack',1.,65),('hit',1.,65))


def skin_weights(part,position):
    x,y,z=position;left=x<0
    if part.startswith('Foot /'):return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Arm /'):
        return [2 if left else 3,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Ear /','Ear lining /')):return [4 if left else 5,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Tuft /','Tuft crease /')):return [6,0,0,0],[1.,0.,0.,0.]
    return [1,0,0,0],[1.,0.,0.,0.]


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Jigglypuff clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(p):return math.sin(math.tau*t-p)+math.sin(p)
    if clip=='idle':return [(0.,0.,0.),(.009*breath,.017*wave,.008*wave),
        (.026*lag(.2),.010*wave,-.016*breath),(.023*lag(.5),-.010*wave,.016*breath),
        (.022*lag(.3),0.,-.014*breath),(.020*lag(.6),0.,.014*breath),(.020*lag(.8),.014*wave,0.)]
    brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
    if clip=='attack':return [(0.,0.,0.),(.032*brace,0.,0.),
        (.064*brace,.014*brace,-.032*brace),(.059*brace,-.014*brace,.032*brace),
        (.042*trail,0.,-.025*trail),(.037*trail,0.,.025*trail),(.037*trail,.012*trail,0.)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    return [(0.,0.,0.),(-.024*recoil,.009*recoil,0.),
        (-.041*recoil,0.,.027*recoil),(-.038*recoil,0.,-.027*recoil),
        (-.031*trail,0.,.023*trail),(-.028*trail,0.,-.023*trail),(-.028*trail,0.,0.)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_jigglypuff(model=None):
    canonical=sculpture()
    if model is None:model=canonical
    if (model.get('name')!='jigglypuff' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Jigglypuff anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Jigglypuff shared balloon body and skin and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'jigglypuff/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'jigglypuff','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'jigglypuff','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'jigglypuff','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'jigglypuff/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'jigglypuff','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'jigglypuff.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'gentle balloon survey; early body and arm brace; delayed ears and forelock settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'jigglypuff',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/jigglypuff_sculpt.py',
        'animation_contract':'identity floor root; planted balloon feet; rigid fitted layered blue eyes and smile; articulated arms, ears and forelock; rotation-only source-cue clips; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_jigglypuff(blob):
    if len(blob)>MAX_BYTES or blob!=export_jigglypuff():raise ValueError('noncanonical Jigglypuff GLB')
    return sculpture()


def read_jigglypuff(path):
    with Path(path).open('rb') as source:return decode_jigglypuff(source.read(MAX_BYTES+1))


def main_export(directory):
    directory=Path(directory);directory.mkdir(parents=True,exist_ok=True)
    (directory/'icon_jigglypuff.glb').write_bytes(export_jigglypuff())
    (directory/'icon_jigglypuff.mesh.json').unlink(missing_ok=True)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/actor_props/icon_jigglypuff.glb'))
    args=parser.parse_args();blob=export_jigglypuff();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    model=sculpture()
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; '+
          f'{sum(len(p["positions"])//3 for p in model["primitives"])} vertices; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__':main()
