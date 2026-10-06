#!/usr/bin/env python3
"""Canonical Bayleef sculpture, anatomical skin, and source-cue-relative clips.

Rebuild uses only Python's standard library. Stored glTF animation articulates
pelvis/chest/long neck/head, four supporting legs, curled collar pairs, a tapered
tail chain, and the three-section notched crown. The floor root stays identity.
"""
import argparse
import hashlib
import math
from pathlib import Path

from bayleef_sculpt import BODY, BODY_HEIGHT, FULL_HEIGHT, LEAF, STEM, TAIL, sculpt, sub
from skin_glb import COORDINATES, Glb, envelope, packed, quantized_weights, quaternion_xyz, smoothstep

FILE='bayleef.glb'
PARTS=tuple(p['part'] for p in sculpt()['primitives'])
MAX_BYTES=512000
JOINTS=(('root',None,(0.,0.,0.)),
        ('pelvis',0,(0.,.259,-.141)),('chest',1,(0.,.357,.147)),
        ('neck_base',2,(0.,.536,.171)),('head',3,(0.,.771,.231)),
        ('foreleg_left',2,(-.178,.254,.205)),('foreleg_right',2,(.178,.254,.205)),
        ('hindleg_left',1,(-.178,.241,-.245)),('hindleg_right',1,(.178,.241,-.245)),
        ('tail_base',1,(0.,.336,-.313)),('tail_tip',9,(.014,.384,-.452)),
        ('crown_petiole',4,(0.,.968,.254)),('crown_mid_fold',11,(.045,1.126,.134)),
        ('crown_tip_fold',12,(.150,1.113,-.136)),
        ('collar_left',3,(-.073,.554,.171)),('collar_right',3,(.073,.554,.171)))
CLIPS=(('idle',3.8,73),('attack',1.,65),('hit',1.,65))


def body_weights(position):
    x,y,z=position
    planted=smoothstep(.060,.213,y)
    head=smoothstep(.709,.778,y)
    neck=smoothstep(.434,.617,y)*(1.-head)
    leg=(1.-smoothstep(.178,.355,y))*smoothstep(.070,.153,abs(x))
    legjoint=(5 if x<0 else 6) if z>-.020 else (7 if x<0 else 8)
    chest=smoothstep(-.126,.186,z)
    active=[(j,w) for j,w in ((0,1.-planted),(4,planted*head),(3,planted*neck),
             (legjoint,planted*(1.-head-neck)*leg),
             (2,planted*(1.-head-neck)*(1.-leg)*chest),
             (1,planted*(1.-head-neck)*(1.-leg)*(1.-chest))) if w>1e-12]
    # The neck/head bands don't overlap the leg blend, so at most four
    # anatomical influences exist at any vertex of this authored surface.
    assert len(active)<=4,(position,active)
    total=sum(w for _,w in active)
    return [j for j,_ in active]+[0]*(4-len(active)),[w/total for _,w in active]+[0.]*(4-len(active))


def skin_weights(part,position):
    x,y,z=position
    if part.startswith('Claw /'):return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Eye /','Muzzle /')):return [4,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Collar /'):
        index=int(part.rsplit(' ',1)[-1])-1
        joint=14 if index in (2,3,4,5) else 15
        flex=smoothstep(.078,.235,math.sqrt(x*x+(z-.171)**2))
        return [3,joint,0,0],[1.-flex,flex,0.,0.]
    if part==TAIL:
        tip=smoothstep(-.382,-.460,z)
        return [9,10,0,0],[1.-tip,tip,0.,0.]
    if part==STEM:return [11,0,0,0],[1.,0.,0.,0.]
    if part==LEAF:
        middle=smoothstep(.190,.065,z);tip=smoothstep(-.066,-.240,z)
        active=[(j,w) for j,w in ((11,1.-middle),(12,middle*(1.-tip)),(13,middle*tip)) if w>0.]
        return [j for j,_ in active]+[0]*(4-len(active)),[w for _,w in active]+[0.]*(4-len(active))
    return body_weights(position)


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Bayleef clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    if clip=='idle':
        def lag(phase):return math.sin(math.tau*t-phase)+math.sin(phase)
        return [(0.,0.,0.),(.008*wave,0.,.003*wave),(-.012*breath,.012*wave,-.003*wave),
                (-.020*breath,.032*wave,.005*wave),(.024*breath,.028*wave,-.006*wave),
                (-.012*breath,0.,.008*wave),(-.012*breath,0.,-.008*wave),
                (.010*breath,0.,.006*wave),(.010*breath,0.,-.006*wave),
                (.016*wave,.035*wave,0.),(.026*lag(.4),.028*lag(.5),0.),
                (.020*lag(.2),.012*wave,.016*wave),(.038*lag(.55),0.,-.016*lag(.45)),
                (.052*lag(.95),.014*wave,.020*lag(.75)),
                (.015*lag(.35),0.,.032*lag(.4)),(.015*lag(.50),0.,-.032*lag(.55))]
    if clip=='attack':
        # Cue-relative fractions preserve the move's authoritative duration.
        # A brief rearward preparation, planted forward neck drive, and delayed
        # blade/collar settle read across the species' different source moves.
        brace=envelope(t,((0.,0.),(.16,-.19),(.44,1.),(.58,.82),(.82,.08),(1.,0.)))
        neck=envelope(t,((0.,0.),(.21,-.20),(.49,1.),(.65,.60),(.86,-.045),(1.,0.)))
        blade=envelope(t,((0.,0.),(.27,-.12),(.55,1.),(.74,-.16),(.91,.035),(1.,0.)))
        tip=envelope(t,((0.,0.),(.33,-.07),(.62,1.),(.81,-.18),(1.,0.)))
        ruff=envelope(t,((0.,0.),(.25,-.14),(.52,1.),(.75,-.20),(1.,0.)))
        return [(0.,0.,0.),(.028*brace,0.,0.),(.066*brace,.010*brace,0.),
                (.092*neck,-.020*neck,0.),(.072*neck,.044*neck,0.),
                (-.080*brace,0.,-.014*brace),(-.080*brace,0.,.014*brace),
                (.035*brace,0.,-.010*brace),(.035*brace,0.,.010*brace),
                (-.045*brace,-.020*brace,0.),(-.062*blade,.035*blade,0.),
                (-.078*blade,0.,-.028*blade),(-.104*blade,.014*blade,.034*blade),
                (-.145*tip,.014*tip,.028*tip),
                (.047*ruff,0.,.085*ruff),(.038*ruff,0.,-.085*ruff)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.38,.40),(.63,-.11),(.84,.025),(1.,0.)))
    neck=envelope(t,((0.,0.),(.21,1.),(.44,.30),(.70,-.105),(1.,0.)))
    blade=envelope(t,((0.,0.),(.29,1.),(.53,.18),(.78,-.12),(1.,0.)))
    tip=envelope(t,((0.,0.),(.36,1.),(.60,.05),(.83,-.07),(1.,0.)))
    return [(0.,0.,0.),(-.025*recoil,0.,.006*recoil),(-.045*recoil,.013*recoil,.007*recoil),
            (-.073*neck,-.026*neck,-.007*neck),(-.097*neck,-.022*neck,-.013*neck),
            (.072*recoil,0.,.016*recoil),(.065*recoil,0.,-.016*recoil),
            (-.032*recoil,0.,.014*recoil),(-.036*recoil,0.,-.014*recoil),
            (.065*blade,0.,.012*blade),(.077*tip,-.022*tip,0.),
            (.100*blade,0.,.025*blade),(.125*tip,0.,-.030*tip),(.155*tip,0.,-.030*tip),
            (-.052*blade,0.,-.090*blade),(-.052*blade,0.,.090*blade)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_bayleef(model=None):
    canonical=sculpt()
    if model is None:model=canonical
    if (model.get('name')!='bayleef' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Bayleef anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite continuous paper Bayleef sculpture and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'bayleef/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'bayleef','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'bayleef','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'bayleef','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'bayleef/paper_'+str(colors[color]),
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
    g.doc['meshes']=[{'name':'bayleef','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'bayleef.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'planted stance; long-neck brace and recoil; delayed crown tail and collar settle; source move cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'bayleef',
        'source_sha256':geometry_digest(model),'reference_span':FULL_HEIGHT,
        'body_reference_height':BODY_HEIGHT,'full_silhouette_height':FULL_HEIGHT,
        'geometry_source':'tools/bayleef_sculpt.py',
        'animation_contract':'identity floor root; fixed soles and claws; continuous long-neck quadruped; rigid fitted face; articulated collar crown and tail; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def decode_bayleef(blob):
    if len(blob)>MAX_BYTES or blob!=export_bayleef():raise ValueError('noncanonical Bayleef GLB')
    return sculpt()


def read_bayleef(path):
    with Path(path).open('rb') as source:return decode_bayleef(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--input',type=Path)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/bayleef.glb'))
    args=parser.parse_args();model=read_bayleef(args.input) if args.input else sculpt();blob=export_bayleef(model)
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(blob)
    print(f'{args.output}: {len(blob)} bytes; {len(model["primitives"])} parts; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; sha256 {hashlib.sha256(blob).hexdigest()}')


if __name__=='__main__':main()
