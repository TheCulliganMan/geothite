#!/usr/bin/env python3
"""Canonical Lapras standard glTF skin and source-cue body response."""
import argparse
import hashlib
import math
from pathlib import Path
from lapras_sculpt import sculpture,HEIGHT
from sculpt_geometry import sub
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
PARTS=tuple(p['part'] for p in sculpture()['primitives'])
MAX_BYTES=1024000
JOINTS=(('root',None,(0.,0.,0.)),('neck_base',0,(0.,.29,.32)),
        ('neck_upper',1,(0.,.84,.54)),('head',2,(0.,1.17,.60)),
        ('flipper_left',0,(-.31,.13,.16)),('flipper_right',0,(.31,.13,.16)),
        ('rear_left',0,(-.31,.13,-.35)),('rear_right',0,(.31,.13,-.35)),
        ('tail',0,(0.,.17,-.51)))
CLIPS=(('idle',4.,77),('attack',1.,65),('hit',1.,65))


def neck_weights(position):
    x,y,z=position;base=smoothstep(.23,.40,y);upper=smoothstep(.67,.90,y)
    head=smoothstep(1.08,1.18,y)
    return [0,1,2,3],[1.-base,base*(1.-upper),base*upper*(1.-head),base*upper*head]


def skin_weights(part,position):
    x,y,z=position
    if part.startswith(('Head /','Eye /','Brow /','Jaw /','Ear /','Horn /','Nostril /','Mouth /')):
        return [3,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Neck /','Throat /','Neck spot /')):return neck_weights(position)
    if part.startswith(('Fore flipper /','Rear flipper /')):
        joint=(4 if x<0 else 5) if part.startswith('Fore') else (6 if x<0 else 7)
        flex=smoothstep(.33,.61,abs(x));return [0,joint,0,0],[1.-flex,flex,0.,0.]
    if part.startswith('Tail /'):
        flex=smoothstep(.53,.73,-z);return [0,8,0,0],[1.-flex,flex,0.,0.]
    return [0,0,0,0],[1.,0.,0.,0.]


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Lapras clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(p):return math.sin(math.tau*t-p)+math.sin(p)
    if clip=='idle':
        return [(0.,0.,0.),(.006*breath,.012*wave,.007*wave),
                (-.010*breath,.018*lag(.4),-.007*wave),(.008*breath,.025*lag(.7),0.),
                (0.,.026*lag(.3),-.010*breath),(0.,-.026*lag(.3),.010*breath),
                (0.,.018*lag(.6),-.008*breath),(0.,-.018*lag(.6),.008*breath),
                (0.,.029*lag(.8),0.)]
    if clip=='attack':
        brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
        trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
        return [(0.,0.,0.),(.025*brace,0.,0.),(.038*brace,0.,0.),(-.020*trail,.018*trail,0.),
                (0.,.065*brace,-.023*brace),(0.,-.065*brace,.023*brace),
                (0.,.035*trail,-.014*trail),(0.,-.035*trail,.014*trail),(0.,.055*trail,0.)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.50,.23),(.78,-.12),(1.,0.)))
    return [(0.,0.,0.),(-.020*recoil,.010*recoil,0.),(-.036*recoil,0.,.010*recoil),
            (.026*trail,-.020*trail,0.),(0.,-.045*recoil,-.016*recoil),
            (0.,.045*recoil,.016*recoil),(0.,-.028*trail,0.),(0.,.028*trail,0.),(0.,-.045*trail,0.)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_lapras(model=None):
    canonical=sculpture()
    if model is None:model=canonical
    if (model.get('name')!='lapras' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Lapras anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite Lapras swimming body and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'lapras/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'lapras','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'lapras','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'lapras','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'lapras/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'lapras','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'lapras.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'curved neck survey; early neck brace; delayed head and swimming flippers settle; source battle cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'lapras',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
        'geometry_source':'tools/lapras_sculpt.py',
        'animation_contract':'identity floor root; seated shell; fitted rigid face; articulated curved neck, flippers and tail; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_lapras(blob):
    if len(blob)>MAX_BYTES or blob!=export_lapras():raise ValueError('noncanonical Lapras GLB')
    return sculpture()


def read_lapras(path):
    with Path(path).open('rb') as source:return decode_lapras(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/lapras.glb'))
    args=parser.parse_args();blob=export_lapras();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(blob)
    model=sculpture()
    print(f'{args.output}: {len(blob)} bytes; {len(PARTS)} parts; '+
          f'{sum(len(p["positions"])//3 for p in model["primitives"])} vertices; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__':main()
