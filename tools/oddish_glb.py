#!/usr/bin/env python3
"""Canonical Oddish standard glTF skin and source-cue body response."""
import argparse
import hashlib
import math
from pathlib import Path
from oddish_sculpt import sculpture,HEIGHT,LEAVES,authoring_transform
from sculpt_geometry import sub
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
PARTS=tuple(p['part'] for p in sculpture()['primitives'])
MAX_BYTES=1024000
_floor,_scale=authoring_transform()
def normalized(point):return tuple((v-(_floor if i==1 else 0.))*_scale for i,v in enumerate(point))
JOINTS=[('root',None,(0.,0.,0.)),('bulb',0,normalized((0.,.287,0.)))]
for j,(start,end,_) in enumerate(LEAVES):
    base=len(JOINTS);JOINTS.append(('leaf_'+str(j)+'_base',1,normalized(start)))
    JOINTS.append(('leaf_'+str(j)+'_tip',base,normalized(tuple(a+(b-a)*.52 for a,b in zip(start,end)))))
JOINTS=tuple(JOINTS)
CLIPS=(('idle',3.6,85),('attack',1.,65),('hit',1.,65))


def skin_weights(part,position):
    x,y,z=position
    if part.startswith('Foot /'):return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Eye /','Eye glint /','Mouth /','Tongue /')):return [1,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Leaf '):
        j=int(part.split()[1]);start,end,_=LEAVES[j];a=normalized(start);b=normalized(end)
        axis=[v-u for u,v in zip(a,b)];t=sum((v-u)*d for v,u,d in zip(position,a,axis))/sum(d*d for d in axis)
        tip=smoothstep(.34,.72,t);base=2+2*j
        return [base,base+1,0,0],[1.-tip,tip,0.,0.]
    flex=smoothstep(.095,.185,y)
    return [0,1,0,0],[1.-flex,flex,0.,0.]


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Oddish clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    pose=[(0.,0.,0.),(.012*breath,.015*wave,.006*wave) if clip=='idle' else (.034*brace,0.,0.) if clip=='attack' else (-.027*recoil,.009*recoil,0.)]
    for j in range(5):
        sign=-1 if j<2 else 1
        if clip=='idle':
            lag=math.sin(math.tau*t-.20*j)+math.sin(.20*j)
            pose.extend(((.020*lag,.012*wave,sign*.014*breath),(.027*lag,.008*wave,sign*.016*breath)))
        elif clip=='attack':pose.extend(((.060*brace,.015*sign*brace,sign*.018*brace),(.075*trail,.012*sign*trail,sign*.026*trail)))
        else:pose.extend(((-.055*recoil,.010*sign*recoil,sign*.023*recoil),(-.069*trail,0.,sign*.020*trail)))
    return pose


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_oddish(model=None):
    canonical=sculpture()
    if model is None:model=canonical
    if (model.get('name')!='oddish' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Oddish anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Oddish blue bulb and leaf skin and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'oddish/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'oddish','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'oddish','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'oddish','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'oddish/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'oddish','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'oddish.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'gentle bulb sway; early bulb and leaf-base brace; delayed leaf tips settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'oddish',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/oddish_sculpt.py',
        'animation_contract':'identity floor root; planted blue root feet; fitted rigid bulb face; five articulated two-joint leaf chains; rotation-only source-cue clips; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_oddish(blob):
    if len(blob)>MAX_BYTES or blob!=export_oddish():raise ValueError('noncanonical Oddish GLB')
    return sculpture()


def read_oddish(path):
    with Path(path).open('rb') as source:return decode_oddish(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/oddish.glb'))
    args=parser.parse_args();blob=export_oddish();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    model=sculpture()
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; '+
          f'{sum(len(p["positions"])//3 for p in model["primitives"])} vertices; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__':main()
