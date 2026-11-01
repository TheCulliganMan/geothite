#!/usr/bin/env python3
"""Canonical mature Meganium paper sculpture and species-specific body clips.

Only standard-library Python is needed. All joints and rotation keys are
stored in the GLB: a two-stage long neck, six cupping flower petals, independent
paired antenna chains, four planted legs, and a low tapered tail. Original
projectile, flash and sound programs remain authoritative for attack timing.
"""
import argparse
import hashlib
import math
import struct
from pathlib import Path
from meganium_sculpt import (BODY, BODY_HEIGHT, FULL_HEIGHT, TAIL, COLLAR, JAW, ANTENNA_LEFT,
    ANTENNA_RIGHT, FLOWER_CENTER, PETAL_ANGLES, flower_basis, sculpt, sub, dot)
from skin_glb import COORDINATES, Glb, envelope, packed, quantized_weights, quaternion_xyz, smoothstep

FILE='meganium.glb'
PARTS=tuple(p['part'] for p in sculpt()['primitives'])
MAX_BYTES=640000
# Reviewed shipped container. Platform libm differences can split one collar
# normal at signed near-zero without changing any expanded triangle position.
# Keep that exact asset accepted; modified containers still require a fresh
# deterministic authoring export rather than a permissive GLB comparison.
REVIEWED_SHA256='fbffaa7d38de96f3d55a6bf6a1bc22461eaab82f09db67aa201b60372415f76e'
JOINTS=(('root',None,(0.,0.,0.)),
        ('pelvis',0,(0.,.335,-.225)),('chest',1,(0.,.485,.180)),
        ('neck_base',2,(0.,.734,.220)),('neck_upper',3,(0.,.997,.300)),
        ('head',4,(0.,1.130,.310)),
        ('foreleg_left',2,(-.250,.310,.300)),('foreleg_right',2,(.250,.310,.300)),
        ('hindleg_left',1,(-.250,.300,-.400)),('hindleg_right',1,(.250,.300,-.400)),
        ('tail_base',1,(0.,.440,-.440)),('tail_tip',10,(0.,.590,-.800)),
        ('antenna_left_base',5,(-.065,1.270,.310)),('antenna_left_tip',12,(-.135,1.465,.060)),
        ('antenna_right_base',5,(.065,1.270,.310)),('antenna_right_tip',14,(.135,1.465,.060)),
        *tuple(('flower_petal_'+str(i+1),3,tuple(FLOWER_CENTER[k]+.119*flower_basis(a)[0][k] for k in range(3))) for i,a in enumerate(PETAL_ANGLES)),
        ('lower_jaw',5,(0.,1.130,.310)))
CLIPS=(('idle',4.2,81),('attack',1.,65),('hit',1.,65))


def body_weights(position):
    x,y,z=position;planted=smoothstep(.080,.284,y)
    head=smoothstep(1.049,1.113,y)
    upper=(1.-head)*smoothstep(.863,1.040,y)
    neck=(1.-head-upper)*smoothstep(.571,.803,y)
    trunk=1.-head-upper-neck
    leg=(1.-smoothstep(.235,.470,y))*smoothstep(.083,.194,abs(x))
    legjoint=(6 if x<0 else 7) if z>-.030 else (8 if x<0 else 9)
    chest=smoothstep(-.178,.230,z)
    active=[(j,w) for j,w in ((0,1.-planted),(5,planted*head),(4,planted*upper),
        (3,planted*neck),(legjoint,planted*trunk*leg),
        (2,planted*trunk*(1.-leg)*chest),(1,planted*trunk*(1.-leg)*(1.-chest))) if w>1e-12]
    assert len(active)<=4,(position,active)
    total=sum(w for _,w in active)
    return [j for j,_ in active]+[0]*(4-len(active)),[w/total for _,w in active]+[0.]*(4-len(active))


def skin_weights(part,position):
    x,y,z=position
    if part.startswith('Claw /'):return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Eye /','Muzzle /')):return [5,0,0,0],[1.,0.,0.,0.]
    if part==JAW or part.startswith('Tongue /'):return [22,0,0,0],[1.,0.,0.,0.]
    if part==COLLAR:return [3,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Flower / folded','Flower / pale','Flower / stamen')):
        # A folded petal and its fitted front/back tissue hinge together.
        # Independent interpolated bend fields bury clipped artwork between
        # the larger parent facets even when all rest positions fit exactly.
        index=int(part.rsplit(' ',1)[-1])-1
        return [16+index,0,0,0],[1.,0.,0.,0.]
    if part==TAIL:
        tip=smoothstep(-.660,-.860,z);return [10,11,0,0],[1.-tip,tip,0.,0.]
    if part in (ANTENNA_LEFT,ANTENNA_RIGHT):
        base=12 if part==ANTENNA_LEFT else 14;tip=smoothstep(.180,-.240,z)
        return [base,base+1,0,0],[1.-tip,tip,0.,0.]
    return body_weights(position)


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Meganium clip/time')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(phase):return math.sin(math.tau*t-phase)+math.sin(phase)
    if clip=='idle':
        pose=[(0.,0.,0.),(.005*wave,0.,.002*wave),(-.009*breath,.009*wave,-.002*wave),
            (-.013*breath,.024*wave,.003*wave),(.008*breath,.028*wave,-.004*wave),
            (.018*breath,.024*wave,-.004*wave),
            (-.010*breath,0.,.005*wave),(-.010*breath,0.,-.005*wave),
            (.009*breath,0.,.004*wave),(.009*breath,0.,-.004*wave),
            (.010*wave,.022*wave,0.),(.017*lag(.5),.030*lag(.7),0.),
            (.016*lag(.3),0.,.027*lag(.4)),(.034*lag(.9),.012*wave,.035*lag(.95)),
            (.016*lag(.45),0.,-.027*lag(.55)),(.034*lag(1.05),-.012*wave,-.035*lag(1.1))]
        for i,angle in enumerate(PETAL_ANGLES):
            _,tangent=flower_basis(angle);flex=.027*lag(.35+.10*i)
            pose.append(tuple(v*flex for v in tangent))
        pose.append((.014*breath,0.,0.))
        return pose
    if clip=='attack':
        # An immediate rooted brace lifts the neck; the flower cups forward while
        # antenna tips lag behind. Fractions adapt to the existing move cue.
        brace=envelope(t,((0.,0.),(.18,1.),(.39,.78),(.58,.36),(.80,.04),(1.,0.)))
        neck=envelope(t,((0.,0.),(.18,1.),(.40,.76),(.62,.28),(.86,-.06),(1.,0.)))
        head=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.62,.24),(.89,-.035),(1.,0.)))
        bloom=envelope(t,((0.,0.),(.25,1.),(.46,.70),(.69,.18),(.85,-.11),(1.,0.)))
        trailing=envelope(t,((0.,0.),(.32,1.),(.52,.66),(.79,-.18),(1.,0.)))
        pose=[(0.,0.,0.),(.019*brace,0.,0.),(.044*brace,.010*brace,0.),
            (.061*neck,-.025*neck,0.),(.079*neck,-.021*neck,0.),(.052*head,.062*head,0.),
            (-.067*brace,0.,-.011*brace),(-.067*brace,0.,.011*brace),
            (.028*brace,0.,-.009*brace),(.028*brace,0.,.009*brace),
            (-.033*brace,-.015*brace,0.),(-.047*trailing,.028*trailing,0.),
            (-.098*bloom,0.,-.047*bloom),(-.144*trailing,.024*trailing,-.061*trailing),
            (-.098*bloom,0.,.047*bloom),(-.144*trailing,-.024*trailing,.061*trailing)]
        for i,angle in enumerate(PETAL_ANGLES):
            _,tangent=flower_basis(angle)
            # The lower lobe opens away from the advancing chest; an inward
            # cup would bury its back fold during the neck drive.
            flex=(-.030 if i==4 else .148+.035*math.sin(angle))*bloom
            pose.append(tuple(v*flex for v in tangent))
        pose.append((.28*max(0.,head),0.,0.))
        return pose
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.32),(.64,-.11),(.84,.025),(1.,0.)))
    neck=envelope(t,((0.,0.),(.21,1.),(.45,.25),(.70,-.095),(1.,0.)))
    bloom=envelope(t,((0.,0.),(.29,1.),(.54,.12),(.78,-.14),(1.,0.)))
    trailing=envelope(t,((0.,0.),(.36,1.),(.61,.03),(.84,-.075),(1.,0.)))
    pose=[(0.,0.,0.),(-.020*recoil,0.,.004*recoil),(-.035*recoil,.012*recoil,.006*recoil),
        (-.059*neck,-.023*neck,-.005*neck),(-.068*neck,-.019*neck,-.008*neck),
        (-.078*neck,-.020*neck,-.010*neck),
        (.058*recoil,0.,.014*recoil),(.053*recoil,0.,-.014*recoil),
        (-.027*recoil,0.,.010*recoil),(-.029*recoil,0.,-.010*recoil),
        (.046*bloom,0.,.009*bloom),(.064*trailing,-.020*trailing,0.),
        (.120*bloom,0.,.055*bloom),(.155*trailing,-.026*trailing,.090*trailing),
        (.120*bloom,0.,-.055*bloom),(.155*trailing,.026*trailing,-.090*trailing)]
    for i,angle in enumerate(PETAL_ANGLES):
        _,tangent=flower_basis(angle);flex=-(.140+.025*math.sin(angle))*bloom
        pose.append(tuple(v*flex for v in tangent))
    pose.append((.18*max(0.,neck),0.,0.))
    return pose


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def pack_parts(model):
    """Batch the six gold stamens without losing anatomical ownership/ranges."""
    parts=[];stamens=None
    for order,p in enumerate(model['primitives']):
        if not p['part'].startswith('Flower / stamen '):
            parts.append((p,{'source_order':order}));continue
        if stamens is None:
            stamens=(dict(part='Flower / complete gold stamens',positions=[],normals=[],indices=[],base_color=p['base_color']),{'source_parts':[]})
            parts.append(stamens)
        q,meta=stamens;start=len(q['positions'])//3
        meta['source_parts'].append(dict(part=p['part'],order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
        q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
    return parts


def export_meganium(model=None):
    canonical=sculpt()
    if model is None:model=canonical
    if (model.get('name')!='meganium' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES
        or [p.get('part') for p in model.get('primitives',[])]!=list(PARTS)
        or geometry_digest(model)!=geometry_digest(canonical)):
        raise ValueError('unreviewed Meganium anatomy')
    g=Glb();g.doc['asset']['generator']='Geothite mature paper Meganium sculpture and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'meganium/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot;inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'meganium','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'meganium','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'meganium','skeleton':0,'joints':list(range(len(JOINTS))),
                    'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[];shared={};parts=pack_parts(model)
    def accessor(values,kind,component=5126,target=None,normalized=False):
        key=(packed(values,component),kind,component,target,normalized)
        if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
        return shared[key]
    for part,meta in parts:
        color=tuple(part['base_color'])
        if color not in colors:
            colors[color]=len(colors);g.doc['materials'].append({'name':'meganium/paper_'+str(colors[color]),
                'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
        ids,weights=[],[]
        for source in meta.get('source_parts',[dict(part=part['part'],vertex_start=0,vertex_count=len(part['positions'])//3)]):
            for i in range(source['vertex_start']*3,(source['vertex_start']+source['vertex_count'])*3,3):
                js,ws=skin_weights(source['part'],part['positions'][i:i+3]);qs=quantized_weights(ws)
                ids.extend(j if w else 0 for j,w in zip(js,qs));weights.extend(qs)
        primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
            'NORMAL':accessor(part['normals'],'VEC3',target=34962),
            'JOINTS_0':accessor(ids,'VEC4',5121,34962),
            'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,normalized=True)},
            'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],
            'mode':4,'extras':{'part':part['part']}})
    g.doc['meshes']=[{'name':'meganium','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'meganium.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
            'purpose':'planted mature stance; two-stage long-neck drive; six flower petals open and settle; curved antennae and low tail follow through; source move cues own timing'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'meganium',
        'source_sha256':geometry_digest(model),'reference_span':FULL_HEIGHT,
        'source_part_ranges':[meta for part,meta in parts],
        'packed_sha256':geometry_digest({'primitives':[part for part,meta in parts]}),
        'body_reference_height':BODY_HEIGHT,'full_silhouette_height':FULL_HEIGHT,
        'geometry_source':'tools/meganium_sculpt.py',
        'animation_contract':'identity floor root; fixed soles and claws; continuous mature quadruped; two-stage long neck; rigid fitted eyes and recessed mouth; hinged lower jaw; six articulated flower petals; paired two-stage antennae and tapered tail; rotation-only cue-relative clips'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def reviewed_geometry_matches(blob, model):
    """Bind the reviewed container to the current recipe despite libm splits."""
    if (model.get('name')!='meganium' or model.get('version')!=1
        or model.get('coordinate_system')!=COORDINATES):return False
    from animated_glb import parse_glb
    doc,binary=parse_glb(blob)
    def values(index):
        accessor=doc['accessors'][index];view=doc['bufferViews'][accessor['bufferView']]
        code={5126:'f',5123:'H'}[accessor['componentType']]
        width={'VEC3':3,'SCALAR':1}[accessor['type']]
        return struct.unpack_from('<'+code*(width*accessor['count']),binary,
            view.get('byteOffset',0)+accessor.get('byteOffset',0))
    stored=[]
    for primitive,meta in zip(doc['meshes'][0]['primitives'],doc['extras']['source_part_ranges']):
        indices=values(primitive['indices']);positions=values(primitive['attributes']['POSITION']);normals=values(primitive['attributes']['NORMAL'])
        sources=meta['source_parts'] if 'source_parts' in meta else [dict(part=primitive['extras']['part'],order=meta['source_order'],vertex_start=0,vertex_count=len(positions)//3,index_start=0,index_count=len(indices))]
        for source in sources:
            a=source['vertex_start'];b=a+source['vertex_count'];i=source['index_start'];j=i+source['index_count']
            stored.append((source['order'],source['part'],[k-a for k in indices[i:j]],positions[3*a:3*b],normals[3*a:3*b],primitive['material']))
    stored.sort();parts=model['primitives']
    if len(stored)!=len(parts):return False
    for (_,name,indices,positions,normals,material),part in zip(stored,parts):
        if name!=part['part']:return False
        color=doc['materials'][material]['pbrMetallicRoughness']['baseColorFactor']
        if color!=part['base_color']:return False
        authored=part['indices']
        if len(indices)!=len(authored):return False
        for source,key,tolerance in ((positions,'positions',0.),(normals,'normals',1e-14)):
            current=part[key]
            for old,new in zip(indices,authored):
                for axis in range(3):
                    a,b=source[old*3+axis],current[new*3+axis]
                    if not math.isfinite(b) or abs(a-b)>tolerance:return False
    return True


def decode_meganium(blob):
    if len(blob)>MAX_BYTES:raise ValueError('noncanonical Meganium GLB')
    model=sculpt()
    if hashlib.sha256(blob).hexdigest()==REVIEWED_SHA256:
        if not reviewed_geometry_matches(blob,model):raise ValueError('reviewed Meganium recipe changed')
    elif blob!=export_meganium(model):raise ValueError('noncanonical Meganium GLB')
    return model


def read_meganium(path):
    with Path(path).open('rb') as source:return decode_meganium(source.read(MAX_BYTES+1))


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--input',type=Path)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/meganium.glb'))
    args=parser.parse_args();model=read_meganium(args.input) if args.input else sculpt();blob=export_meganium(model)
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(blob)
    print(f'{args.output}: {len(blob)} bytes; {len(model["primitives"])} parts; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; sha256 {hashlib.sha256(blob).hexdigest()}')


if __name__=='__main__':main()
