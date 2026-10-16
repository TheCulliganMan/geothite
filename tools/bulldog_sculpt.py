#!/usr/bin/env python3
"""Original Snubbull and Granbull with distinct jowls, ears, collars and teeth."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch as _fitted_patch,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from wing_insect_sculpt import membrane,contour
from skin_glb import COORDINATES
SPECIES=('snubbull','granbull');HEIGHT=1.06
PINK=(.91,.53,.64);PURPLE=(.66,.48,.66);BLUE=(.42,.70,.84);CREAM=(.95,.87,.65);WHITE=(.98,.97,.93);INK=(.17,.16,.19);TONGUE=(.79,.43,.46)

def fitted_patch(name,color,outline,surface,thickness=.008):
    # Python 3.12 changed float summation. Keep the original left-to-right
    # centroid, including centerline signed zero, in Blender's newer Python.
    outline=list(outline);area=0.
    for a,b in zip(outline,outline[1:]+outline[:1]):area+=a[0]*b[1]-b[0]*a[1]
    if area<0:outline.reverse()
    cx=cy=0.
    for x,y in outline:cx+=x;cy+=y
    return _fitted_patch(name,color,outline,surface,thickness,center=(cx/len(outline),cy/len(outline)))

def ears(p,big):
    c=PURPLE if big else PINK;base=1.015 if big else .891
    for s in (-1,1):
        outline=contour([((s*.18,base),(s*.185,base+.15),(s*.30,base+.215),(s*.37,base+.153)),((s*.37,base+.153),(s*.42,base+.121),(s*.452,base+.058),(s*.449,base+.002)),((s*.449,base+.002),(s*.357,base+.045),(s*.29,base+.022),(s*.18,base))],10)
        e=membrane('Ear / broad folded bulldog flap '+str(s),c,outline,1,root_z=.001,thickness=.035)
        # Lift the center ridge and fold the outer third toward the muzzle.
        v=[tuple(e['positions'][i:i+3]) for i in range(0,len(e['positions']),3)]
        v=[(x,y,z+.058*math.sin(math.pi*max(0,min(1,(abs(x)-.18)/.27)))) for x,y,z in v]
        e=crease_normals(part(e['part'],c,v,[tuple(e['indices'][i:i+3]) for i in range(0,len(e['indices']),3)]),35.);p.append(e)
        if not big:
            es=surface(e);patch=[(s*.35,base+.140),(s*.389,base+.129),(s*.425,base+.075),(s*.431,base+.033),(s*.376,base+.049)]
            p.append(fitted_patch('Ear marking / fitted dark outer tip '+str(s),INK,patch,es,.009))

def eyes(p,head,big):
    hs=surface(head);y=.976 if big else .852
    for s in (-1,1):
        x=s*(.122 if big else .123);outline=[(x-s*.050,y+.002),(x+s*.062,y+.041),(x+s*.045,y-.030),(x-s*.028,y-.038)]
        socket=fitted_patch('Eye / fitted stern dark socket '+str(s),INK,outline,hs,.012);p.append(socket)
        cx=sum(a for a,b in outline)/4;cy=sum(b for a,b in outline)/4
        white=fitted_patch('Eye / tilted ivory white '+str(s),WHITE,[(cx+(a-cx)*.87,cy+(b-cy)*.83) for a,b in outline],surface(socket),.004);p.append(white)
        pupil=fitted_patch('Pupil / small dark canine pupil '+str(s),INK,oval(x,y-.011,.010,.013,20),surface(white),.003);p.append(pupil)
        p.append(fitted_patch('Eye / small ivory glint '+str(s),WHITE,oval(x-.003,y-.006,.0025,.003,12),surface(pupil),.001))

def limbs(p,body,big):
    c=PURPLE if big else PINK
    for s in (-1,1):
        pts=[(s*.15,.335,-.010),(s*.193,.209,-.005),(s*.222,.085,.041)] if big else [(s*.116,.237,0),(s*.132,.124,.009),(s*.154,.066,.048)]
        p.append(curved_tube('Leg / bent muscular supporting shin '+str(s),c,pts,[.095,.068,.054] if big else [.047,.041,.034],22))
        foot=ellipsoid('Foot / broad grounded bulldog paw '+str(s),c if big else CREAM,(s*(.224 if big else .157),.052 if big else .047,.092),(.101,.052,.134) if big else (.080,.047,.112),28,18);p.append(foot);fs=surface(foot)
        for j,dx in enumerate((-.032,.032) if big else (-.025,.025)):
            x=s*(.224 if big else .157)+dx;p.append(tube(f'Toe groove / fitted cleft {s} {j}',INK,[(x,y,fs(y,x)+.001) for y in (.025,.045,.067)],[.0014]*3,8))
        pts=[(s*.14,.643,-.002),(s*.294,.671,.014),(s*.426,.640,.090)] if big else [(s*.099,.495,0),(s*.193,.565,.021),(s*.278,.637,.078)]
        p.append(curved_tube('Arm / curved reaching bulldog forelimb '+str(s),c,pts,[.070,.067,.056] if big else [.037,.033,.027],22))
        if big:p.append(curved_tube('Wrist / dark snug cuff '+str(s),INK,[(s*.372,.652,.059),(s*.397,.646,.073)],[.063,.061],22))
        hand=ellipsoid('Hand / broad rounded bulldog fist '+str(s),c,pts[-1],(.077,.064,.078) if big else (.048,.035,.048),26,18);p.append(hand)
        if big:
            hs=surface(hand)
            for j,dx in enumerate((-.022,.022)):
                x=pts[-1][0]+dx;p.append(tube(f'Hand fold / fitted knuckle cleft {s} {j}',INK,[(x,y,hs(y,x)+.001) for y in (.608,.630,.652)],[.0014]*3,8))
    p.append(curved_tube('Tail / short blunt canine tail',c,[(0,.332 if big else .293,-.105),(0,.383 if big else .330,-.207),(0,.423 if big else .365,-.259)],[.039,.026,.002],20))

def snubbull():
    body=loft('Body / continuous pink dress-shaped torso',PINK,curved_profiles(((.178,.207,.137,-.005),(.221,.239,.161,-.005),(.286,.206,.142,-.006),(.391,.148,.108,0),(.474,.137,.107,0),(.555,.080,.073,.005)),4),48)
    v=[tuple(body['positions'][i:i+3]) for i in range(0,len(body['positions']),3)]
    v=[(x,y+.018*math.cos(6*math.atan2(x,z+.005))*max(0,1-(y-.178)/.13),z) for x,y,z in v]
    body=crease_normals(part(body['part'],PINK,v,[tuple(body['indices'][i:i+3]) for i in range(0,len(body['indices']),3)]),40.)
    neck=loft('Collar / continuous blue ruffled neck band',BLUE,curved_profiles(((.505,.112,.095,.001),(.54,.134,.108,.001),(.576,.097,.084,.002)),4),32)
    head=ellipsoid('Head / oversized round pink bulldog crown',PINK,(0,.771,.013),(.279,.219,.190),40,28);p=[body,neck,head];eyes(p,head,False);ears(p,False)
    for s in (-1,1):p.append(ellipsoid('Jowl / broad rounded pink cheek '+str(s),PINK,(s*.135,.708,.165),(.139,.107,.085),28,20))
    mouth=fitted_patch('Mouth / fitted dark bulldog pout',INK,oval(0,.656,.095,.045,28),surface(head),.008);p.append(mouth)
    jaw=ellipsoid('Jaw / projecting round pink lower lip',PINK,(0,.640,.233),(.105,.076,.073),32,22);p.append(jaw)
    nose=ellipsoid('Nose / short black canine button',INK,(0,.747,.228),(.043,.021,.034),26,18);p.append(nose)
    for s in (-1,1):p.append(curved_tube('Fang / small downward ivory canine '+str(s),WHITE,[(s*.073,.696,.233),(s*.066,.675,.284),(s*.057,.650,.328)],[.021,.013,.0015],18))
    for j,(x,y,w,h) in enumerate(((-.105,.390,.037,.060),(.105,.390,.037,.060),(-.066,.260,.039,.045),(.066,.260,.039,.045))):p.append(fitted_patch('Spot / fitted blue dress oval '+str(j),BLUE,oval(x,y,w,h,28),surface(body),.009))
    limbs(p,body,False);return p

def granbull():
    body=loft('Body / powerful purple chest belly and hips',PURPLE,curved_profiles(((.234,.110,.078,-.006),(.29,.219,.149,-.012),(.399,.240,.178,-.012),(.526,.217,.165,-.002),(.645,.185,.125,.009),(.724,.104,.095,.013)),4),36)
    neck=ellipsoid('Neck / strong purple supporting neck',PURPLE,(0,.732,.015),(.107,.123,.090),28,18)
    head=loft('Head / high canine forehead and heavy jaw support',PURPLE,curved_profiles(((.643,.044,.051,.10),(.706,.198,.158,.028),(.822,.261,.173,.015),(.965,.249,.176,-.006),(1.067,.189,.139,-.020),(1.102,.043,.032,-.020)),4),40)
    p=[body,neck,head];eyes(p,head,True);ears(p,True)
    bib=fitted_patch('Collar / fitted dark triangular chest bib',INK,[(-.120,.685),(.120,.685),(.079,.560),(0,.528),(-.079,.560)],surface(body),.012);p.append(bib)
    mouth=fitted_patch('Mouth / broad open dark canine mouth',INK,oval(0,.790,.163,.094,36),surface(head),.010);p.append(mouth)
    p.append(fitted_patch('Mouth / fitted rosy tongue',TONGUE,oval(0,.735,.089,.035,28),surface(mouth),.006))
    jaw=ellipsoid('Jaw / heavy projecting purple lower jaw',PURPLE,(0,.696,.198),(.185,.065,.116),36,22);p.append(jaw)
    muzzle=ellipsoid('Muzzle / short upper purple canine snout',PURPLE,(0,.889,.194),(.125,.058,.080),32,20);p.append(muzzle)
    p.append(ellipsoid('Nose / broad black canine nose',INK,(0,.907,.260),(.046,.027,.034),28,18))
    for s in (-1,1):
        p.append(curved_tube('Tusk / enormous upward ivory lower canine '+str(s),WHITE,[(s*.147,.710,.235),(s*.192,.782,.267),(s*.215,.882,.247),(s*.199,.981,.183),(s*.178,1.036,.150)],[.045,.048,.034,.013,.0015],24))
        p.append(curved_tube('Fang / small downward upper canine '+str(s),WHITE,[(s*.062,.854,.238),(s*.068,.823,.260)],[.013,.0015],16))
    limbs(p,body,True);return p

def sculpture(species,normalize=True):
    if species not in SPECIES:raise ValueError('unsupported bulldog')
    p=snubbull() if species=='snubbull' else granbull();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    from sculpt_geometry import linear
    # Exact local Crystal normal RGB555 colors, expanded to sRGB8.
    for q in p:
        name=q['part'];big=species=='granbull'
        rgb=(123,33,140) if name.startswith('Mouth / fitted rosy') else (0,0,0) if name.startswith(('Nose /','Pupil /','Eye / fitted','Ear marking /','Mouth /','Wrist /','Hand fold /','Toe groove /','Collar / fitted')) else (57,82,107) if name.startswith(('Spot /','Collar / continuous')) else (255,255,255) if name.startswith(('Fang /','Tusk /','Eye /')) or (not big and name.startswith('Foot /')) else ((189,99,181) if big else (231,115,107))
        q['base_color']=linear(tuple(v/255 for v in rgb))
    if normalize:
        for q in p:
            q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))
if __name__=='__main__':main()
