#!/usr/bin/env python3
"""Original Spearow standard glTF skin, authored from a compact volume recipe."""
import argparse,hashlib,math
from pathlib import Path
from functools import lru_cache
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
from spearow_sculpt import sculpture,HEIGHT,PALETTE
FILE='spearow.glb'
MAX_BYTES=250000
CLIPS=(('idle',2.8,73),('attack',1.,65),('hit',1.,65))
@lru_cache(maxsize=1)
def authoring_bundle():return sculpture(with_transform=True)
def normalized(p):
 lo,k=authoring_bundle()[1];return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(p))
def sub(a,b):return tuple(x-y for x,y in zip(a,b))
JOINTS=tuple((n,parent,(0.,0.,0.) if parent is None else normalized(p)) for n,parent,p in (
 ('root',None,(0.,0.,0.)),('torso',0,(0.,.220,-.035)),('head',1,(0.,.374,.060)),
 ('lower_beak',2,(0.,.433,.177)),('wing_left',1,(-.115,.350,-.015)),
 ('flight_feathers_left',4,(-.216,.269,-.115)),('wing_right',1,(.115,.350,-.015)),
 ('flight_feathers_right',6,(.216,.269,-.115)),('tail_fan',1,(0.,.212,-.174)),('crown',2,(0.,.529,.025))))


def skin_weights(part, position):
    x,y,z=position
    if part.startswith('Feet /'): return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Crown /'): return [9,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Beak / lower'): return [3,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Head /','Eye ','Beak /')): return [2,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Tail /'): return [8,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Wing '):
        shoulder,wrist=(4,5) if 'left' in part else (6,7)
        # Each layered flight-feather assembly is rigid paper. Its color
        # crease must use exactly the same hinge, avoiding thin-fold inversion.
        joint=wrist if 'flight feather' in part or 'ivory covert tips' in part else shoulder
        return [joint,0,0,0],[1.,0.,0.,0.]
    # Lower belly is planted with the foot-root. The chest can settle forward
    # without detaching from the stationary ankle attachments.
    lo,k=authoring_bundle()[1];upper=smoothstep(.110,.230,y/k+lo)
    return [0,1 if upper else 0,0,0],[1.-upper,upper,0.,0.]


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.: raise ValueError('invalid clip or time')
    if t in (0.,1.): return [(0.,0.,0.)]*len(JOINTS)
    a,b=math.sin(math.tau*t),math.sin(2*math.tau*t)
    pulse=.5-.5*math.cos(math.tau*t)
    if clip=='idle':
        # Quick avian glance, chest breath, alternating folded-wing adjustment,
        # and a delayed fan twitch. The stance stays alert rather than flapping.
        glance=envelope(t,((0.,0.),(.16,0.),(.28,1.),(.42,.85),(.57,0.),(.73,-.48),(.90,0.),(1.,0.)))
        return [(0,0,0),(.017*a,0,.008*b),(-.040*pulse,.105*glance,-.018*glance),
                (.022*pulse,0,0),(-.038*pulse,-.018*a,-.048*pulse),(.025*a,0,-.029*pulse),
                (-.028*pulse,.015*a,.037*pulse),(-.018*b,0,.023*pulse),
                (.025*a,.040*b,0),(.022*b,0,-.016*a)]
    if clip=='attack':
        # Source attack progress already owns the move's onset. Open the beak
        # and balance with the wings as the peck starts, without a reverse
        # anticipation that competes with the source's actual displacement.
        peck=envelope(t,((0.,0.),(.18,1.),(.39,.78),(.55,.38),(.77,.04),(1.,0.)))
        spread=envelope(t,((0.,0.),(.12,1.),(.34,.88),(.58,.54),(.84,.06),(1.,0.)))
        snap=envelope(t,((0.,0.),(.12,1.),(.30,.65),(.52,.10),(.74,0.),(1.,0.)))
        return [(0,0,0),(.115*peck,0,0),(.230*peck,.021*peck,0),(.11*snap,0,0),
                (-.13*spread,-.13*spread,-.30*spread),(.08*spread,-.035*spread,-.10*spread),
                (-.11*spread,.13*spread,.28*spread),(.065*spread,.035*spread,.09*spread),
                (-.11*peck,.018*spread,0),(-.105*peck,0,0)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.38,.42),(.62,-.13),(.83,.035),(1.,0.)))
    lag=envelope(t,((0.,0.),(.23,1.),(.44,.47),(.71,-.10),(1.,0.)))
    return [(0,0,0),(-.10*recoil,.020*recoil,.012*recoil),(-.17*recoil,-.04*recoil,-.024*recoil),
            (.065*lag,0,0),(.11*recoil,-.04*lag,-.17*recoil),(-.09*lag,0,-.065*lag),
            (.09*recoil,.04*lag,.13*recoil),(-.07*lag,0,.05*lag),
            (.09*lag,-.04*lag,0),(.095*lag,0,.023*lag)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_spearow(model=None):
    model=sculpture() if model is None else model
    g=Glb(); g.doc['asset']['generator']='Geothite original Spearow volume and skin v2'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'spearow/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None: nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot
        inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'spearow','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'spearow','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'spearow','skeleton':0,'joints':list(range(len(JOINTS))),
                     'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=part['color']
        if color not in colors:
            colors[color]=len(colors)
            g.doc['materials'].append({'name':'spearow/'+color,'pbrMetallicRoughness':{
                'baseColorFactor':part['base_color'],'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
        ids,weights=[],[]
        for i in range(0,len(part['positions']),3):
            joints,ws=skin_weights(part['part'],part['positions'][i:i+3]); ws=quantized_weights(ws)
            ids.extend(j if w else 0 for j,w in zip(joints,ws));weights.extend(ws)
        primitives.append({'attributes':{
            'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
            'NORMAL':g.accessor(part['normals'],'VEC3',target=34962),
            'JOINTS_0':g.accessor(ids,'VEC4',5121,34962),
            'WEIGHTS_0':g.accessor(weights,'VEC4',5121,34962,normalized=True)},
            'indices':g.accessor(part['indices'],'SCALAR',5123,34963),
            'material':colors[color],'mode':4,'extras':{'part':part['part']}})
    g.doc['meshes']=[{'name':'spearow','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True)
        channels,samplers=[],[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'spearow.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
                'purpose':'Spearow alert glance, folded-wing counterbalance, short peck and startled recoil; game-owned cues'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'spearow',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'animation_contract':'identity root and grounded toes; rotation-only presentation clips; source battle cues own timing'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def read_spearow(path):
    # The recipe is canonical. Readers reject edited storage or accidental clip
    # changes instead of pretending an unknown GLB matches this sculpture.
    with Path(path).open('rb') as source:
        blob=source.read(MAX_BYTES+1)
    if len(blob)>MAX_BYTES or blob!=export_spearow(): raise ValueError('noncanonical Spearow GLB')
    return sculpture()


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/spearow.glb'))
    args=parser.parse_args(); model=sculpture();blob=export_spearow(model)
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(blob)
    print(f'{args.output}: {len(blob)} bytes; {len(model["primitives"])} anatomy parts; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__': main()
