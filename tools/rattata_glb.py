#!/usr/bin/env python3
"""Canonical Rattata standard glTF skin and source-cue body response."""
import argparse
import hashlib
import math
from pathlib import Path
from rattata_sculpt import sculpture,HEIGHT,authoring_transform,TAIL
from sculpt_geometry import sub
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
PARTS=tuple(p['part'] for p in sculpture()['primitives'])
MAX_BYTES=1024000
_floor,_scale=authoring_transform()
def normalized(point):return tuple((v-(_floor if i==1 else 0.))*_scale for i,v in enumerate(point))
JOINTS=(('root',None,(0.,0.,0.)),('torso',0,normalized((0.,.235,-.075))),
        ('neck',1,normalized((0.,.265,.105))),('head',2,normalized((0.,.325,.205))),
        ('ear_left',3,normalized((-.132,.415,.12))),('ear_right',3,normalized((.132,.415,.12))),
        ('tail_base',1,normalized(TAIL[0])),('tail_curl',6,normalized(TAIL[18])),
        ('tail_tip',7,normalized(TAIL[36])))
CLIPS=(('idle',3.4,85),('attack',1.,65),('hit',1.,65))


def skin_weights(part,position):
    x,y,z=position
    if part.startswith(('Paw /','Toes /','Leg /')):return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Ear /','Ear lining /')):return [4 if x<0 else 5,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Neck /'):return [2,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Tail /'):
        points=[normalized(p) for p in TAIL]
        index=min(range(len(points)),key=lambda i:sum((a-b)**2 for a,b in zip(position,points[i])))
        t=index/(len(points)-1)
        if t<.16:
            blend=smoothstep(0.,.16,t);return [1,6,0,0],[1.-blend,blend,0.,0.]
        if t<.57:
            blend=smoothstep(.27,.57,t);return [6,7,0,0],[1.-blend,blend,0.,0.]
        blend=smoothstep(.65,.89,t);return [7,8,0,0],[1.-blend,blend,0.,0.]
    if part.startswith(('Body /','Belly /')):return [1,0,0,0],[1.,0.,0.,0.]
    return [3,0,0,0],[1.,0.,0.,0.]


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Rattata clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(p):return math.sin(math.tau*t-p)+math.sin(p)
    if clip=='idle':return [(0.,0.,0.),(.009*breath,.009*wave,0.),(.007*breath,.014*wave,0.),
        (.009*breath,.018*wave,0.),(.017*lag(.2),0.,-.009*breath),(.015*lag(.5),0.,.009*breath),
        (0.,.019*wave,0.),(.012*lag(.4),.027*lag(.4),0.),(.016*lag(.8),.033*lag(.8),0.)]
    brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
    if clip=='attack':return [(0.,0.,0.),(.025*brace,0.,0.),(.026*brace,0.,0.),(.035*brace,0.,0.),
        (.037*trail,0.,-.021*trail),(.032*trail,0.,.021*trail),(.015*brace,.032*brace,0.),
        (.024*trail,.045*trail,0.),(.031*trail,.059*trail,0.)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    return [(0.,0.,0.),(-.019*recoil,.006*recoil,0.),(-.021*recoil,0.,0.),(-.027*recoil,0.,0.),
        (-.023*trail,0.,.015*trail),(-.021*trail,0.,-.015*trail),(-.012*recoil,-.024*recoil,0.),
        (-.019*trail,-.038*trail,0.),(-.024*trail,-.047*trail,0.)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_rattata(model=None):
    canonical=sculpture()
    if model is None:model=canonical
    if (model.get('name')!='rattata' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Rattata anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Rattata purple quadruped and curled tail skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'rattata/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'rattata','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'rattata','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'rattata','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'rattata/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'rattata','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'rattata.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'small rodent breathing; early head brace; delayed curled tail settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'rattata',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/rattata_sculpt.py',
        'animation_contract':'identity floor root; four planted paws; coherent fitted face and incisors; articulated head and ears; three-joint curled tail; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_rattata(blob):
    if len(blob)>MAX_BYTES or blob!=export_rattata():raise ValueError('noncanonical Rattata GLB')
    return sculpture()


def read_rattata(path):
    with Path(path).open('rb') as source:return decode_rattata(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/rattata.glb'))
    args=parser.parse_args();blob=export_rattata();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    model=sculpture()
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; '+
          f'{sum(len(p["positions"])//3 for p in model["primitives"])} vertices; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__':main()
