#!/usr/bin/env python3
"""Original Girafarig and Stantler with distinct quadruped anatomy."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch as _fitted_patch,crease_normals,fitted_triangle_contour
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

def deer_mark(name,color,outline,skin,thickness=.003):
    return fitted_triangle_contour(name,color,outline,skin,thickness,precision=8)


def deer_legs(p):
    for side in (-1,1):
        for row in range(2):
            pts=[(side*.075,.575,.175),(side*.125,.43,.192),(side*.125,.24,.194),(side*.125,.034,.197)] if row==0 else [(side*.065,.575,-.215),(side*.13,.455,-.232),(side*.135,.32,-.295),(side*.13,.205,-.250),(side*.13,.034,-.235)]
            radii=[.035,.046,.029,.023] if row==0 else [.035,.057,.038,.026,.024]
            p.append(tube(f'Leg / sculpted supporting shin {side} {row}',TAN,pts,radii,18))
            x,_,z=pts[-1]
            # Flat weight-bearing sole, sloping pastern and two cloven toe tips.
            outline=[(-.035,-.035),(-.035,.018),(-.026,.070),(-.005,.078),(0.,.061),(.005,.078),(.026,.070),(.035,.018),(.035,-.035)]
            from abra_sculpt import polygon_caps
            n=len(outline);v=[(x+xx,y,z+zz) for y in (0.,.067) for xx,zz in outline]
            v[n:]=[(x+(xx-x)*.72,max(.018,min(.067,.067*(.070-(zz-z))/.105)),z+(zz-z)*.72-.008) for xx,yy,zz in v[n:]]
            caps=polygon_caps(outline);faces=[tuple(q) for q in caps]+[tuple(i+n for i in reversed(q)) for q in caps]
            faces += [(j,(j+1)%n,(j+1)%n+n,j+n) for j in range(n)]
            hoof=part(f'Hoof / grounded dark cloven pad {side} {row}',HOOF,v,faces);p.append(hoof)
            hs=surface(hoof)
            p.append(tube(f'Hoof groove / fitted central cleft {side} {row}',INK,[(x,y,hs(y,x)+.0008) for y in (.012,.027,.042)],[.0012]*3,8))


def angular_antler(name,points,radius=.014):
    # Deliberate corners and tapered branch junctions, rather than a spline oval.
    n=12;v=[];f=[]
    for j,q in enumerate(points):
        a=points[j-1];b=points[(j+1)%len(points)];dx=b[0]-a[0];dy=b[1]-a[1];l=math.hypot(dx,dy)
        for k in range(n):
            t=k*math.tau/n;v.append((q[0]-radius*dy/l*math.cos(t),q[1]+radius*dx/l*math.cos(t),q[2]+radius*math.sin(t)))
    for j in range(len(points)):
        for k in range(n):
            a=j*n+k;b=j*n+(k+1)%n;c=((j+1)%len(points))*n+(k+1)%n;d=((j+1)%len(points))*n+k;f.append((a,b,c,d))
    return crease_normals(part(name,ANTLER,v,f),35.)


def stantler():
    body=trunk('Body / continuous lean tan deer trunk',TAN,curved_profiles(((-.35,.025,.03,.55),(-.26,.138,.151,.56),(-.11,.157,.148,.55),(.10,.138,.151,.54),(.235,.071,.104,.57),(.27,.022,.029,.59)),4));p=[body]
    neck=loft('Neck / sculpted upright deer neck',TAN,curved_profiles(((.51,.055,.09,.185),(.65,.088,.102,.21),(.78,.075,.081,.244),(.94,.080,.085,.265)),4),30);p.append(neck)
    head=loft('Head / long deer forehead and padded cheeks',TAN,curved_profiles(((.775,.043,.058,.338),(.83,.111,.130,.315),(.92,.127,.137,.268),(1.015,.111,.105,.240),(1.069,.031,.035,.237)),4),36);p.append(head)
    for side in (-1,1):
        outline=[(side*.084,1.018),(side*.153,1.043),(side*.267,.973),(side*.245,.957),(side*.126,.986)]
        ear=membrane('Ear / broad pointed pinna '+str(side),TAN,outline,1,root_z=.251,thickness=.027);p.append(ear)
        p.append(membrane('Ear / fitted inner pinna '+str(side),NOSE,outline,1,root_z=.251,inset=.68,offset=.028,thickness=.003))
        x=side*.079;y=.956
        outline=[(x-side*.033,y+.017),(x+side*.028,y+.005),(x+side*.023,y-.025),(x-side*.023,y-.019)]
        socket=deer_mark('Eye / fitted dark socket '+str(side),INK,outline,head,.003);p.append(socket)
        white=deer_mark('Eye / fitted ivory white '+str(side),WHITE,[(x+(xx-x)*.86,y+(yy-y)*.78) for xx,yy in outline],socket,.002);p.append(white)
        p.append(deer_mark('Eye / dark vertical pupil '+str(side),INK,oval(x-side*.008,y-.005,.006,.013,18),white,.0015))
    muzzle=loft('Muzzle pad / broad tan projecting cheeks',TAN,curved_profiles(((.765,.030,.040,.366),(.79,.086,.070,.375),(.835,.100,.087,.380),(.884,.079,.068,.363),(.915,.025,.025,.335)),3),28);p.append(muzzle)
    nose=ellipsoid('Muzzle / small dark deer nose',NOSE,(0,.827,.470),(.049,.044,.044),28,16);p.append(nose)
    for side in (-1,1):p.append(deer_mark('Nostril / fitted dark deer fleck '+str(side),INK,oval(side*.022,.822,.006,.004,14),nose,.0015))
    p.append(deer_mark('Chest / fitted cream deer bib',IVORY,[(-.020,.762),(-.047,.720),(-.055,.642),(-.033,.576),(0.,.563),(.033,.576),(.055,.642),(.047,.720),(.020,.762)],neck,.005))
    for side in (-1,1):
        p.append(tube('Antler stem / pale rooted brow stalk '+str(side),ANTLER,[(side*.064,1.042,.235),(side*.10,1.128,.25)],[.022,.017],16))
        loop=[(side*.10,1.128,.25),(side*.223,1.160,.25),(side*.284,1.261,.25),(side*.265,1.345,.25),(side*.149,1.290,.25),(side*.126,1.225,.25)]
        p.append(angular_antler('Antler / open angular branching loop '+str(side),loop,.014))
        for j,pts in enumerate(([(side*.284,1.261,.25),(side*.302,1.356,.25),(side*.352,1.386,.25)],[(side*.265,1.345,.25),(side*.253,1.431,.25)],[(side*.149,1.290,.25),(side*.127,1.348,.25)])):
            p.append(tube(f'Antler tine / angular tapered branch {side} {j}',ANTLER,pts,[.015,.011,.006] if len(pts)==3 else [.014,.006],12))
        p.append(ellipsoid('Antler orb / dark violet lower loop orb '+str(side),ORB,(side*.185,1.191,.253),(.031,.030,.030),20,12))
        p.append(deer_mark('Brow spot / fitted pale forehead oval '+str(side),IVORY,oval(side*.043,1.016,.011,.018,18),head,.003))
        for j,(z,y,w,h) in enumerate(((-.13,.65,.016,.030),(-.07,.627,.012,.022))):p.append(flank_patch(f'Spot / fitted pale deer flank marking {side} {j}',IVORY,oval(-side*z,y,w,h,18),body,side,.004))
        rotated=dict(body);rotated['positions']=[v for i in range(0,len(body['positions']),3) for v in (-side*body['positions'][i+2],body['positions'][i+1],side*body['positions'][i])]
        for j in range(3):
            y=.515-j*.047;outline=[(-side*z,yy) for z,yy in ((-.20,y-.005),(.07,y+.025),(.09,y+.012),(-.20,y-.020))]
            rib=deer_mark(f'Rib / fitted cream flank stripe {side} {j}',IVORY,outline,rotated,.003)
            for field in ('positions','normals'):rib[field]=[v for i in range(0,len(rib[field]),3) for v in (side*rib[field][i+2],rib[field][i+1],-side*rib[field][i])]
            p.append(rib)
    deer_legs(p)
    p.append(tube('Tail / short supporting deer tail base',TAN,[(0,.57,-.277),(0,.60,-.36),(0,.64,-.402)],[.045,.043,.032],18))
    p.append(ellipsoid('Tail / large soft curled tan tail',TAN,(0,.664,-.43),(.119,.128,.118),32,20))
    return p

def sculpture(species,normalize=True):
    if species not in SPECIES:raise ValueError('unsupported psychic hoofed species')
    p=girafarig() if species=='girafarig' else stantler()
    if species=='stantler':
        # Full-color Battle Revolution palette. The pack's four-color shadow
        # indices do not mean a blue nose or pure white antler pigment.
        from chikorita_sculpt import linear
        for q in p:
            n=q['part']
            rgb=(30,25,29) if n.startswith(('Eye / fitted dark','Eye / dark','Nostril /','Hoof groove /')) else (100,62,42) if n.startswith(('Muzzle /','Ear / fitted')) else (80,51,95) if n.startswith(('Hoof /','Antler orb /')) else (242,218,143) if n.startswith(('Antler /','Antler stem /','Antler tine /')) else (245,239,217) if n.startswith(('Eye / fitted ivory','Chest /','Brow spot /','Spot /','Rib /')) else (200,156,120)
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
    if species=='stantler':
        from chikorita_sculpt import f32
        for i,q in enumerate(p):
            if not q['part'].startswith(('Eye /','Nostril /','Chest /','Brow spot /','Rib /')):continue
            vs=[tuple(f32(v) for v in q['positions'][j:j+3]) for j in range(0,len(q['positions']),3)]
            faces=[tuple(q['indices'][j:j+3]) for j in range(0,len(q['indices']),3)]
            r=part(q['part'],(0.,0.,0.),vs,faces);r['base_color']=q['base_color'];p[i]=r
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))
if __name__=='__main__':main()
