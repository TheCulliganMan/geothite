#!/usr/bin/env python3
"""Original honey cub and powerful ring bear with fitted lunar markings."""
import argparse,json,math,struct
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from wing_insect_sculpt import membrane,contour
from skin_glb import COORDINATES
SPECIES=('teddiursa','ursaring');HEIGHT=1.14
ORANGE=(.91,.58,.32);BROWN=(.49,.35,.29);CREAM=(.97,.89,.58);IVORY=(.93,.97,.93);INK=(.12,.12,.14);TONGUE=(.78,.43,.42)

def marking(name,mesh,cx,cy,outer,inner,offset=(0,0),crescent=False):
    # Closed sampled annular strip: the skin remains visible in the center/gap.
    skin=surface(mesh);n=48;rows=6;xy=[]
    for j in range(rows):
        t=j/(rows-1)
        for i in range(n):
            a=.42+(math.tau-.84)*i/(n-1) if crescent else math.tau*i/n
            x=cx+outer[0]*math.cos(a)*(1-t)+(offset[0]+inner[0]*math.cos(a))*t
            y=cy+outer[1]*math.sin(a)*(1-t)+(offset[1]+inner[1]*math.sin(a))*t;xy.append((x,y))
    count=len(xy);v=[(x,y,skin(y,x)+d) for d in (.010,-.0035) for x,y in xy];f=[]
    for j in range(rows-1):
        for i in range(n-1 if crescent else n):a=j*n+i;b=j*n+(i+1)%n;f.append((a,b,b+n,a+n))
    f += [tuple(i+count for i in reversed(face)) for face in list(f)]
    for j in (0,rows-1):
        for i in range(n-1 if crescent else n):
            a=j*n+i;b=j*n+(i+1)%n;f.append((b,a,a+count,b+count) if j==0 else (a,b,b+count,a+count))
    if crescent:
        for i in (0,n-1):
            for j in range(rows-1):a=j*n+i;b=a+n;f.append((a,b,b+count,a+count) if i==0 else (b,a,a+count,b+count))
    return part(name,CREAM,v,f)

def ears(p,head,big):
    c=BROWN if big else ORANGE;x=.129 if big else .198;y=1.170 if big else 1.026
    for s in (-1,1):
        ear=ellipsoid('Ear / round bear pinna '+str(s),c,(s*x,y,-.008),(.057,.071,.043) if big else (.087,.097,.063),28,20);p.append(ear)
        p.append(fitted_patch('Ear / fitted dark inner pinna '+str(s),CREAM if big else INK,oval(s*x,y,.028 if big else .044,.040 if big else .060,26),surface(ear),.004))

def paws(p,big):
    c=BROWN if big else ORANGE
    for s in (-1,1):
        legpts=[(s*.18,.355,-.014),(s*.228,.217,.003),(s*.246,.088,.047)] if big else [(s*.105,.217,-.002),(s*.133,.128,.005),(s*.150,.064,.040)]
        p.append(curved_tube('Leg / heavy bent bear shin '+str(s),c,legpts,[.109,.083,.061] if big else [.056,.047,.036],22))
        foot=ellipsoid('Foot / broad grounded bear paw '+str(s),c,(s*(.25 if big else .152),.052 if big else .046,.094),(.113,.052,.145) if big else (.085,.046,.114),28,18);p.append(foot)
        for j,dx in enumerate((-.044,0,.044) if big else (-.030,0,.030)):
            x=s*(.25 if big else .152)+dx
            p.append(curved_tube(f'Foot claw / ivory forward talon {s} {j}',IVORY,[(x,.036,.199 if big else .178),(x,.025,.238 if big else .211),(x,.013,.269 if big else .233)],[.021,.013,.0015] if big else [.014,.009,.0015],16))
        if big:
            pts=[(s*.177,.810,.005),(s*.294,.695,.025),(s*.316,.536,.070)];rr=[.092,.082,.068]
        elif s<0:pts=[(-.104,.576,.010),(-.224,.614,.090),(-.162,.705,.225),(-.120,.760,.274)];rr=[.043,.047,.041,.036]
        else:pts=[(.109,.580,.005),(.219,.601,.028),(.286,.567,.091)];rr=[.043,.039,.032]
        p.append(curved_tube('Arm / sculpted bear forelimb '+str(s),c,pts,rr,22))
        hand=ellipsoid('Hand / broad rounded bear palm '+str(s),c,pts[-1],(.079,.077,.082) if big else (.071,.084,.060) if s<0 else (.063,.058,.066),28,20);p.append(hand)
        for j,dx in enumerate((-.045,-.015,.015,.045) if big else (-.027,0,.027)):
            x=pts[-1][0]+dx;y=pts[-1][1];z=pts[-1][2]
            path=[(x,y-.040,z+.054),(x,y-.083,z+.071),(x,y-.139,z+.076)] if big else [(x,y+.046,z+.037),(x,y+.077,z+.048),(x,y+.097,z+.047)] if s<0 else [(x,y-.020,z+.045),(x,y-.044,z+.081),(x,y-.066,z+.088)]
            p.append(curved_tube(f'Hand claw / ivory curved talon {s} {j}',IVORY,path,[.017,.014,.0015] if big else [.011,.008,.0015],16))

        if big:
            p.append(curved_tube('Hand claw / ivory curved talon '+str(s)+' 4',IVORY,[(s*.375,.546,.127),(s*.404,.507,.170),(s*.421,.466,.176)],[.017,.013,.0015],16))

def teddiursa():
    body=loft('Body / compact round honey cub torso',ORANGE,curved_profiles(((.219,.04,.05,-.007),(.282,.146,.113,-.012),(.39,.179,.143,-.005),(.515,.157,.121,-.003),(.615,.110,.091,.010),(.675,.049,.045,.012)),4),36)
    neck=ellipsoid('Neck / short orange supporting neck',ORANGE,(0,.660,.012),(.086,.084,.077),28,18)
    head=ellipsoid('Head / oversized round orange cub face',ORANGE,(0,.848,.011),(.247,.213,.187),40,28);p=[body,neck,head];ears(p,head,False);hs=surface(head)
    p.append(marking('Moon / fitted open forehead crescent',head,-.006,.977,(.078,.072),(.060,.049),(.012,-.004),True))
    for s in (-1,1):
        eye=fitted_patch('Eye / large round dark cub eye '+str(s),INK,oval(s*.091,.875,.031,.038,30),hs,.009);p.append(eye)
        p.append(fitted_patch('Eye / bright ivory cub glint '+str(s),IVORY,oval(s*.091-.009,.890,.012,.013,22),surface(eye),.003))
    muzzle=ellipsoid('Muzzle / broad cream honey cub cheeks',CREAM,(0,.791,.194),(.147,.078,.073),32,22);p.append(muzzle);ms=surface(muzzle)
    p.append(ellipsoid('Nose / small dark honey cub button',INK,(0,.825,.256),(.033,.020,.024),26,18))
    mouth=fitted_patch('Mouth / small dark honey cub smile',INK,oval(.033,.757,.030,.018,24),ms,.004);p.append(mouth)
    p.append(fitted_patch('Mouth / fitted little rosy tongue',TONGUE,oval(.037,.752,.016,.006,18),surface(mouth),.002))
    paws(p,False)
    p.append(ellipsoid('Tail / soft round orange tail',ORANGE,(.027,.318,-.173),(.066,.061,.066),28,18))
    # Short cub legs retain the same grounded feet; lower the entire upper pose.
    for q in p:
        if not q['part'].startswith(('Leg /','Foot /','Foot claw /')):
            q['positions']=[v-.065 if i%3==1 else v for i,v in enumerate(q['positions'])]
    return p

def ursaring():
    body=loft('Body / massive pear-shaped brown bear trunk',BROWN,curved_profiles(((.258,.083,.069,-.016),(.34,.236,.165,-.020),(.484,.282,.198,-.012),(.636,.272,.192,-.006),(.79,.231,.158,.004),(.895,.143,.106,.012),(.975,.075,.065,.015)),4),40)
    neck=loft('Neck / thick upright brown bear neck',BROWN,curved_profiles(((.848,.122,.091,.01),(.957,.114,.094,.017),(1.067,.097,.090,.020)),4),30)
    head=ellipsoid('Head / small stern adult bear crown',BROWN,(0,1.065,.031),(.151,.153,.128),36,24);p=[body,neck,head];ears(p,head,True);hs=surface(head)
    p.append(marking('Ring / fitted cream annular chest marking',body,0,.617,(.180,.205),(.149,.170)))
    for s in (-1,1):
        x=s*.068;y=1.111;outline=[(x-s*.035,y+.004),(x+s*.044,y+.015),(x+s*.030,y-.019),(x-s*.015,y-.019)]
        eye=fitted_patch('Eye / fitted stern ivory adult eye '+str(s),IVORY,outline,hs,.006);p.append(eye)
        p.append(fitted_patch('Pupil / narrow dark adult bear pupil '+str(s),INK,oval(x,y-.004,.004,.010,16),surface(eye),.003))
    muzzle=ellipsoid('Muzzle / broad projecting cream adult snout',CREAM,(0,1.033,.147),(.119,.071,.082),32,22);p.append(muzzle);ms=surface(muzzle)
    p.append(ellipsoid('Nose / broad dark adult bear nose',INK,(0,1.071,.209),(.033,.022,.025),28,18))
    p.append(tube('Mouth / dark fitted adult frown',INK,[(x,y,ms(y,x)+.001) for x,y in ((-.058,1.001),(0,1.020),(.058,1.001))],[.0023,.0023,.0023],10))
    # Closed layered shoulder fur creates the adult ruff's radial silhouette.
    for s in (-1,1):
        for j in range(4):
            x=s*(.068+.045*j);y=.946-.017*j
            outline=contour([((x-s*.035,y),(x+s*.020,y-.012),(x+s*.080,y-.085),(x+s*.061,y-.125)),((x+s*.061,y-.125),(x+s*.012,y-.103),(x-s*.021,y-.038),(x-s*.035,y))],7)
            fur=membrane(f'Ruff / pointed layered shoulder fur {s} {j}',BROWN,outline,1,root_z=.070,thickness=.024)
            # Sample real chest/neck depth; projecting tips use the nearest side
            # skin so they extend the silhouette instead of sinking into it.
            bs,ns=surface(body),surface(neck)
            def support(y,x):
                xx=x
                for attempt in range(50):
                    values=[]
                    for skin in (bs,ns):
                        try:values.append(skin(y,xx))
                        except ValueError:pass
                    if values:return max(values)
                    xx*=.94
                raise ValueError('ruff outside supporting bear height')
            p.append(fitted_contour(fur['part'],BROWN,outline,support,.024))
    paws(p,True)
    p.append(ellipsoid('Tail / small rounded brown tail',BROWN,(0,.354,-.192),(.075,.065,.070),28,18));return p

def sculpture(species,normalize=True):
    if species not in SPECIES:raise ValueError('unsupported bear')
    p=teddiursa() if species=='teddiursa' else ursaring()
    # Smooth the curved adult marking and fitted ruff while retaining their
    # sharp perimeter creases, exact surface positions and triangle order.
    if species=='ursaring':p=[crease_normals(q,35.) if q['part'].startswith(('Ring /','Ruff /')) else q for q in p]
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    # The local normal palettes contain two authored colors plus white/black.
    # Keep the original geometry and part names while using those exact colors.
    from sculpt_geometry import linear
    coat=(165,90,0) if species=='ursaring' else (181,115,49)
    marking_color=(255,255,49) if species=='ursaring' else (255,255,255)
    for q in p:
        name=q['part']
        rgb=(0,0,0) if name.startswith(('Nose /','Pupil /','Mouth / dark','Mouth / small','Eye / large')) else (115,57,33) if name.startswith(('Ear / fitted','Mouth / fitted little')) and species=='teddiursa' else marking_color if name.startswith(('Moon /','Ring /','Muzzle /','Ear / fitted')) else (255,255,255) if name.startswith(('Eye /','Hand claw /','Foot claw /')) else coat
        q['base_color']=linear(tuple(v/255 for v in rgb))
    if normalize:
        for q in p:
            q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
            # Reuse exact stored position/normal pairs without smoothing creases
            # or changing a single indexed triangle corner. Preserve signed zero.
            vertices={};remap=[];positions=[];normals=[]
            for i in range(0,len(q['positions']),3):
                point=q['positions'][i:i+3];normal=q['normals'][i:i+3];key=struct.pack('<6f',*point,*normal)
                if key not in vertices:
                    vertices[key]=len(positions)//3;positions.extend(point);normals.extend(normal)
                remap.append(vertices[key])
            q['positions']=positions;q['normals']=normals;q['indices']=[remap[i] for i in q['indices']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))
if __name__=='__main__':main()
