#!/usr/bin/env python3
"""Original Girafarig and Stantler with distinct quadruped anatomy."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch as _fitted_patch,crease_normals
from abra_sculpt import surface,polygon_caps
from bovine_sculpt import oval,curved_tube,reverse_front
from wing_insect_sculpt import membrane,contour
from skin_glb import COORDINATES
SPECIES=('girafarig','stantler');HEIGHT=1.28
GOLD=(.95,.74,.28);BROWN=(.27,.20,.19);PINK=(.88,.43,.54);IVORY=(.97,.95,.80);WHITE=(.98,.98,.95);INK=(.10,.12,.13)
TAN=(.69,.52,.44);NOSE=(.46,.31,.23);HOOF=(.27,.24,.30);ANTLER=(.93,.82,.43);ORB=(.35,.27,.40)

def fitted_patch(name,color,outline,skin,thickness=.008):
    # Preserve the original centroid and signed zeros in newer Blender Python.
    outline=list(outline);area=0.
    for a,b in zip(outline,outline[1:]+outline[:1]):area+=a[0]*b[1]-b[0]*a[1]
    if area<0:outline.reverse()
    cx=cy=0.
    for x,y in outline:cx+=x;cy+=y
    return _fitted_patch(name,color,outline,skin,thickness,center=(cx/len(outline),cy/len(outline)))

def trunk(name,color,rows):
    n=32;v=[]
    for z,w,h,y in rows:
        for i in range(n):a=i*math.tau/n;v.append((w*math.sin(a),y+h*math.cos(a),z))
    f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
    for j in range(len(rows)-1):
        for i in range(n):a=j*n+i;b=j*n+(i+1)%n;f.append((a,b,b+n,a+n))
    return crease_normals(part(name,color,v,f),40.)

def side_skin(p,side):
    q=dict(p);q['positions']=[v for i in range(0,len(p['positions']),3) for v in (-side*p['positions'][i+2],p['positions'][i+1],side*p['positions'][i])]
    return surface(q)

def flank_patch(name,color,outline,mesh,side,thickness=.003):
    q=fitted_patch(name,color,outline,side_skin(mesh,side),thickness)
    for field in ('positions','normals'):q[field]=[v for i in range(0,len(q[field]),3) for v in (side*q[field][i+2],q[field][i+1],-side*q[field][i])]
    return q

def legs(p,giraffe):
    for side in (-1,1):
        for row,z in enumerate((.18,-.20)):
            color=GOLD if giraffe and row==0 else BROWN if giraffe else TAN
            pts=[(side*.075,.57,z),(side*.132,.47,z),(side*.14,.30,z+.015),(side*.14,.08,z+.034)] if row==0 else [(side*.06,.57,z),(side*.143,.49,z),(side*.16,.34,z-.065),(side*.14,.20,z+.020),(side*.145,.070,z-.002)]
            radii=[.035,.061,.036,.026] if row==0 else [.036,.074,.055,.032,.028]
            p.append(curved_tube(f'Leg / sculpted supporting shin {side} {row}',color,pts,radii,20))
            x=pts[-1][0];zz=pts[-1][2]+.018
            hoof=ellipsoid(f'Hoof / grounded dark cloven pad {side} {row}',HOOF,(x,.044,zz),(.054,.044,.074),24,16);p.append(hoof);hs=surface(hoof)
            p.append(tube(f'Hoof groove / fitted central cleft {side} {row}',INK,[(x,y,hs(y,x)+.001) for y in (.022,.042,.061)],[.0013]*3,8))

def ears(p,base_y,base_z,color,inner):
    for side in (-1,1):
        outline=contour([((side*.088,base_y),(side*.13,base_y+.065),(side*.265,base_y+.102),(side*.277,base_y+.089)),((side*.277,base_y+.089),(side*.265,base_y+.02),(side*.17,base_y-.002),(side*.088,base_y))],7)
        root=base_z+.06*.14-.025*(base_y-.70)**2
        p.append(membrane('Ear / broad pointed pinna '+str(side),color,outline,1,root_z=root,thickness=.027))
        p.append(membrane('Ear / fitted inner pinna '+str(side),inner,outline,1,root_z=root,inset=.68,offset=.028,thickness=.003))

def eyes(p,head,giraffe):
    hs=surface(head)
    for side in (-1,1):
        x=side*(.065 if giraffe else .075);y=1.148 if giraffe else .943
        outline=oval(x,y,.036,.050,26) if giraffe else [(x-side*.036,y+.009),(x+side*.037,y+.034),(x+side*.030,y-.029),(x-side*.028,y-.021)]
        socket=fitted_patch('Eye / fitted dark socket '+str(side),INK,outline,hs,.009);p.append(socket)
        cx=sum(a for a,b in outline)/len(outline);cy=sum(b for a,b in outline)/len(outline)
        white=fitted_patch('Eye / fitted ivory white '+str(side),WHITE,[(cx+(a-cx)*.86,cy+(b-cy)*.84) for a,b in outline],surface(socket),.004);p.append(white)
        iris=fitted_patch('Eye / dark vertical pupil '+str(side),INK,oval(x,y+.001,.011 if giraffe else .006,.025 if giraffe else .015,18),surface(white),.002);p.append(iris)
        if giraffe:p.append(fitted_patch('Eye / small white glint '+str(side),WHITE,oval(x-.003,y+.010,.003,.006,12),surface(iris),.001))
        else:
            pts=[(x+side*dx,y+.049+dx*.28,0) for dx in (-.029,0,.030)]
            pts=[(xx,yy,hs(yy,xx)+.0015) for xx,yy,z in pts]
            p.append(tube('Brow / fitted stern deer brow '+str(side),NOSE,pts,[.003,.004,.002],10))

def mane_plate(name,points):
    if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(points,points[1:]+points[:1]))<0:points=list(reversed(points))
    v=[(x,y,z) for x in (-.018,.018) for y,z in points];n=len(points);caps=polygon_caps(points);f=[tuple(reversed(q)) for q in caps]+[tuple(i+n for i in q) for q in caps]
    f.extend((i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n));return part(name,PINK,v,f)

def girafarig():
    rows=curved_profiles(((-.34,.025,.035,.49),(-.26,.15,.15,.49),(-.10,.177,.169,.49),(0,.177,.17,.49),(.14,.15,.18,.49),(.25,.06,.115,.52),(.28,.025,.034,.54)),4)
    split=next(i for i,r in enumerate(rows) if r[0]==0)
    rear=trunk('Body / continuous brown hindquarters',BROWN,rows[:split+1]);front=trunk('Body / continuous golden forequarters',GOLD,rows[split:]);p=[front,rear]
    neck=loft('Neck / long sculpted golden giraffe neck',GOLD,curved_profiles(((.48,.06,.09,.155),(.61,.105,.107,.174),(.76,.071,.077,.22),(.93,.067,.076,.248),(1.065,.085,.077,.258)),4),30);p.append(neck)
    head=loft('Head / tapered golden giraffe face',GOLD,curved_profiles(((.952,.025,.028,.31),(1.01,.083,.11,.289),(1.09,.13,.14,.252),(1.18,.12,.106,.236),(1.215,.026,.03,.225)),4),36);p.append(head);hs=surface(head)
    ears(p,1.172,.236,GOLD,PINK);eyes(p,head,True)
    muzzle=ellipsoid('Muzzle / short broad pink nose',PINK,(0,1.105,.375),(.089,.042,.048),28,16);p.append(muzzle)
    mouth=fitted_patch('Mouth / fitted open giraffe smile',BROWN,[(-.049,1.052),(.061,1.058),(.041,.989),(-.034,.993)],hs,.010);p.append(mouth)
    p.append(fitted_patch('Mouth / pink tongue',PINK,[(-.02,1.020),(.034,1.022),(.019,1.003),(-.013,1.004)],surface(mouth),.003))
    for side in (-1,1):
        p.append(curved_tube('Horn / short ivory ossicone '+str(side),IVORY,[(side*.050,1.19,.234),(side*.064,1.307,.232),(side*.073,1.38,.231)],[.023,.018,.016],18))
        p.append(ellipsoid('Horn tip / rounded ivory bulb '+str(side),IVORY,(side*.073,1.385,.231),(.031,.032,.03),22,14))
        for j,(z,y,w,h) in enumerate(((.11,.54,.045,.061),(.19,.64,.033,.054))):
            p.append(flank_patch(f'Spot / fitted brown shoulder marking {side} {j}',BROWN,oval(-side*z,y,w,h,20),front if y<.61 else neck,side,.004))
        for j,(y,w,h) in enumerate(((.79,.025,.046),(.91,.025,.04))):
            p.append(flank_patch(f'Spot / fitted brown neck marking {side} {j}',BROWN,oval(-side*(.22 if y<.85 else .248),y,w,h,20),neck,side,.004))
        for j,(z,y,w,h) in enumerate(((-.14,.60,.034,.046),(-.24,.58,.030,.039))):
            p.append(flank_patch(f'Spot / fitted gold hindquarter marking {side} {j}',GOLD,oval(-side*z,y,w,h,20),rear,side,.004))
    for j,(y,z) in enumerate(((.67,.070),(.79,.139),(.91,.169),(1.025,.183))):
        p.append(mane_plate('Mane / pink neck crest '+str(j),[(y-.035,z+.026),(y+.041,z+.03),(y+.003,z-.056)]))
    for j,z in enumerate((-.20,-.08,.04)):
        p.append(mane_plate('Mane / pink back crest '+str(j),[(.625,z-.036),(.642,z+.04),(.747,z-.022)]))
    legs(p,True)
    stem=[(0,.51,-.285),(.008,.67,-.405),(.03,.855,-.512),(.042,.977,-.552)]
    p.append(curved_tube('Tail / long slender brown second neck',BROWN,stem,[.024,.026,.021,.024],18))
    tail=ellipsoid('Tail head / rounded sentient brown head',BROWN,(.042,1.010,-.559),(.094,.085,.094),30,18);p.append(tail);back=reverse_front(tail);bs=surface(back)
    for side in (-1,1):
        x=-.042+side*.042;y=1.035
        eye=fitted_patch('Tail eye / gold oval '+str(side),GOLD,oval(x,y,.025,.032,22),bs,.006);p.append(reverse_front(eye))
        p.append(reverse_front(fitted_patch('Tail eye / dark vertical pupil '+str(side),INK,oval(x,y,.007,.024,16),surface(eye),.002)))
    grin=fitted_patch('Tail mouth / fitted ivory jagged grin',IVORY,[(-.11,.995),(.022,.995),(.001,.966),(-.08,.966)],bs,.005);p.append(reverse_front(grin));gs=surface(grin)
    for j,x in enumerate((-.079,-.050,-.02)):
        p.append(reverse_front(fitted_patch('Tail tooth / fitted dark divider '+str(j),BROWN,[(x-.002,.991),(x+.002,.991),(x+.009,.971),(x+.005,.971)],gs,.0015)))
    # Crystal's front spots have a dark center inside their pink border.
    for outer in list(p):
        name=outer['part']
        if not name.startswith(('Spot / fitted brown shoulder','Spot / fitted brown neck')):continue
        side,j=map(int,name.split()[-2:])
        if 'shoulder' in name:z,y,w,h=((.11,.54,.045,.061),(.19,.64,.033,.054))[j];region='shoulder'
        else:z,y,w,h=((.22,.79,.025,.046),(.248,.91,.025,.040))[j];region='neck'
        p.append(flank_patch(f'Spot center / fitted dark {region} marking {side} {j}',INK,oval(-side*z,y,w*.62,h*.62,20),outer,side,.002))
    return p

def closed_antler(name,points,radius=.019):
    controls=points[:-1];path=[];n=18
    for i in range(len(controls)):
        a,b,c,d=[controls[j%len(controls)] for j in (i-1,i,i+1,i+2)]
        for j in range(4):
            t=j/4;path.append(tuple(.5*(2*b[k]+(-a[k]+c[k])*t+(2*a[k]-5*b[k]+4*c[k]-d[k])*t*t+(-a[k]+3*b[k]-3*c[k]+d[k])*t*t*t) for k in range(3)))
    v=[];f=[]
    for j,q in enumerate(path):
        prev=path[j-1];next=path[(j+1)%len(path)];dx=next[0]-prev[0];dy=next[1]-prev[1];length=math.hypot(dx,dy)
        for k in range(n):
            a=k*math.tau/n;v.append((q[0]-radius*dy/length*math.cos(a),q[1]+radius*dx/length*math.cos(a),q[2]+radius*math.sin(a)))
    for j in range(len(path)):
        for k in range(n):
            a=j*n+k;b=j*n+(k+1)%n;c=((j+1)%len(path))*n+(k+1)%n;d=((j+1)%len(path))*n+k;f.append((a,b,c,d))
    return crease_normals(part(name,ANTLER,v,f),40.)

def stantler():
    body=trunk('Body / continuous lean tan deer trunk',TAN,curved_profiles(((-.35,.025,.03,.55),(-.26,.151,.172,.56),(-.11,.176,.174,.55),(.10,.147,.168,.54),(.235,.071,.104,.57),(.27,.022,.029,.59)),4));p=[body]
    neck=loft('Neck / sculpted upright deer neck',TAN,curved_profiles(((.51,.055,.09,.185),(.65,.100,.109,.21),(.78,.075,.081,.244),(.91,.080,.085,.265)),4),30);p.append(neck)
    head=loft('Head / long stern deer forehead and cheek folds',TAN,curved_profiles(((.764,.034,.048,.332),(.83,.12,.132,.298),(.92,.135,.134,.260),(1.007,.113,.103,.240),(1.054,.031,.035,.237)),4),36);p.append(head);hs=surface(head)
    ears(p,1.008,.245,TAN,NOSE);eyes(p,head,False)
    muzzle=ellipsoid('Muzzle / broad projecting brown deer nose',NOSE,(0,.840,.409),(.091,.086,.077),30,18);p.append(muzzle);ms=surface(muzzle)
    for side in (-1,1):p.append(fitted_patch('Nostril / fitted dark deer fleck '+str(side),INK,oval(side*.032,.824,.009,.006,14),ms,.002))
    p.append(fitted_patch('Chest / fitted cream deer bib',IVORY,oval(0,.659,.042,.070,24),surface(neck),.007))
    for side in (-1,1):
        p.append(curved_tube('Antler stem / pale rooted brow stalk '+str(side),ANTLER,[(side*.067,1.031,.235),(side*.095,1.112,.25)],[.021,.019],18))
        loop=[(side*.095,1.112,.25),(side*.22,1.12,.25),(side*.295,1.222,.25),(side*.275,1.316,.25),(side*.204,1.350,.25),(side*.137,1.322,.25),(side*.095,1.247,.25),(side*.095,1.112,.25)]
        p.append(closed_antler('Antler / open branching oval loop '+str(side),loop))
        for j,pts in enumerate(([(side*.275,1.316,.25),(side*.365,1.335,.25),(side*.386,1.37,.25)],[(side*.204,1.350,.25),(side*.209,1.403,.25),(side*.226,1.430,.25)],[(side*.137,1.322,.25),(side*.108,1.372,.25),(side*.11,1.40,.25)])):
            p.append(curved_tube(f'Antler tine / rounded branch {side} {j}',ANTLER,pts,[.018,.014,.009],16))
        p.append(ellipsoid('Antler orb / dark violet lower loop orb '+str(side),ORB,(side*.192,1.155,.276),(.032,.032,.032),24,16))
        p.append(fitted_patch('Brow spot / fitted pale forehead oval '+str(side),IVORY,oval(side*.046,1.01,.015,.024,20),hs,.004))
        for j,(z,y,w,h) in enumerate(((-.13,.65,.016,.030),(-.07,.627,.012,.022))):p.append(flank_patch(f'Spot / fitted pale deer flank marking {side} {j}',IVORY,oval(-side*z,y,w,h,18),body,side,.004))
    legs(p,False)
    p.append(curved_tube('Tail / short supporting deer tail base',TAN,[(0,.57,-.277),(0,.60,-.36),(0,.64,-.402)],[.045,.043,.032],18))
    p.append(ellipsoid('Tail / large soft curled tan tail',TAN,(0,.664,-.43),(.119,.128,.118),32,20))
    return p

def sculpture(species,normalize=True):
    if species not in SPECIES:raise ValueError('unsupported psychic hoofed species')
    p=girafarig() if species=='girafarig' else stantler()
    if species=='stantler':
        # Normal palette from the user-supplied local pack; no shiny verification.
        from chikorita_sculpt import linear
        for q in p:
            n=q['part']
            rgb=(0,0,0) if n.startswith(('Eye / fitted dark','Eye / dark','Nostril /','Hoof groove /')) else (33,41,132) if n.startswith(('Muzzle /','Brow /','Hoof /','Ear / fitted','Antler orb /')) else (255,255,255) if n.startswith(('Antler /','Antler stem /','Antler tine /','Eye / fitted ivory','Chest /','Brow spot /','Spot /')) else (189,123,57)
            q['base_color']=linear(tuple(v/255 for v in rgb))
    if species=='girafarig':
        # Smooth the fitted curved panels without changing any triangle positions.
        p=[crease_normals(q,35.) if q['part'].startswith(('Spot /','Spot center /','Eye /','Tail eye /')) else q for q in p]
        from chikorita_sculpt import linear
        for q in p:
            n=q['part']
            rgb=(255,255,255) if n.startswith(('Eye / fitted ivory','Eye / small','Horn /','Horn tip /','Tail mouth /')) else (231,82,132) if n.startswith(('Muzzle /','Ear / fitted','Mane /','Hoof /','Mouth / pink','Spot / fitted brown')) else (0,0,0) if n.startswith(('Spot center /','Eye / fitted dark','Eye / dark','Hoof groove /','Body / continuous brown','Tail /','Tail head /','Tail tooth /','Tail eye / dark','Mouth / fitted')) or (n.startswith('Leg /') and n.endswith('1')) else (255,181,33)
            q['base_color']=linear(tuple(v/255 for v in rgb))
    if not normalize:return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))
if __name__=='__main__':main()
