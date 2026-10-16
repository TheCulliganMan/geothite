#!/usr/bin/env python3
"""Original Natu orb and Xatu ceremonial cloak with fitted symbolic markings."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,fitted_patch,fitted_contour,crease_normals,tube
from abra_sculpt import surface
from bovine_sculpt import fitted_patch,oval,curved_tube,reverse_front
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
SPECIES=('natu','xatu');HEIGHT=1.04
# Local normal RGB555 for both birds: green (10,23,13), red (20,5,6).
GREEN=(82/255,189/255,107/255);RED=(165/255,41/255,49/255);GOLD=(1.,1.,1.);IVORY=GOLD;INK=(0.,0.,0.);GLINT=GOLD

def transform(p,sign,undo=False):
    q=dict(p);angle=sign*.95;c=math.cos(angle);s=math.sin(angle)
    for field in ('positions','normals'):
        out=[];v=p[field]
        for i in range(0,len(v),3):
            x,y,z=v[i:i+3]
            if undo:
                if field=='positions':x-=sign*.221;z+=.021
                xx=x*c-z*s;zz=x*s+z*c;x=sign*xx;z=zz
            else:
                x*=sign;xx=x*c+z*s;zz=-x*s+z*c;x=xx;z=zz
                if field=='positions':x+=sign*.221;z-=.021
            out.extend((x,y,z))
        q[field]=out
    # Mirroring local X reverses the winding on either transform.
    if sign<0:q['indices']=[j for i in range(0,len(p['indices']),3) for j in (p['indices'][i],p['indices'][i+2],p['indices'][i+1])]
    return q

def eyes(p,head,y,x,width,height):
    skin=surface(head)
    for sign in (-1,1):
        cx=sign*x;xy=contour([((cx-width,y),(cx-width*.6,y+height),(cx+width*.5,y+height),(cx+width,y)),((cx+width,y),(cx+width*.5,y-height*.6),(cx-width*.5,y-height*.7),(cx-width,y))],10)
        eye=fitted_contour('Eye / calm ivory almond '+str(sign),IVORY,xy,skin,.005);p.append(eye)
        iris=fitted_patch('Pupil / broad dark watchful oval '+str(sign),INK,oval(cx,y+.003,width*.43,height*.45,32),surface(eye),.003);p.append(iris)
        p.append(fitted_patch('Eye glint / soft upper silver reflection '+str(sign),GLINT,oval(cx-.008,y+height*.18,width*.19,height*.13,24),surface(iris),.002))

def feet(p,root_y):
    for sign in (-1,1):
        x=sign*.097;p.append(curved_tube('Leg / short supporting red bird ankle '+str(sign),RED,[(x,root_y,-.010),(x,.077,.015)],[.024,.025],20))
        for j,dx in enumerate((-.036,0,.036)):
            p.append(curved_tube('Toe / long splayed red bird digit '+str(sign)+' '+str(j),RED,[(x,.077,.015),(x+dx,.043,.103),(x+dx*1.4,.025,.165)],[.023,.018,.011],16))
            p.append(curved_tube('Claw / pointed ivory bird talon '+str(sign)+' '+str(j),IVORY,[(x+dx*1.27,.029,.150),(x+dx*1.65,.012,.212)],[.014,.001],16))
        p.append(curved_tube('Toe / curved red rear bird digit '+str(sign),RED,[(x,.070,.010),(x+sign*.023,.036,-.050),(x+sign*.034,.023,-.073)],[.021,.014,.003],16))

def natu():
    body=ellipsoid('Body / single round green Natu orb',GREEN,(0,.332,0),(.255,.255,.236),48,32);p=[body];eyes(p,body,.398,.113,.067,.033);feet(p,.146)
    beak=ellipsoid('Beak / broad rounded golden seed bill',RED,(0,.333,.268),(.089,.037,.102),36,20);p.append(beak)
    smile=contour([((-.060,.328),(-.024,.317),(.023,.317),(.060,.328)),((.060,.328),(.022,.321),(-.025,.321),(-.060,.328))],10)
    p.append(fitted_contour('Bill seam / fitted narrow dark Natu smile',INK,smile,surface(beak),.002))
    plume=loft('Crest / single swept red lance feather',RED,curved_profiles(((.510,.011,.007,-.046),(.552,.029,.013,-.061),(.612,.036,.015,-.093),(.689,.019,.011,-.133),(.725,.001,.001,-.150)),4),28)
    plume['positions']=[v+(.070*(plume['positions'][i-i%3+1]-.510)/.215 if i%3==0 else 0) for i,v in enumerate(plume['positions'])];p.append(plume)
    yellow=contour([((-.070,.239),(-.092,.277),(-.092,.339),(-.059,.389)),((-.059,.389),(-.028,.409),(.002,.397),(.009,.373)),((.009,.373),(.005,.322),(-.012,.273),(-.043,.240)),((-.043,.240),(-.054,.233),(-.063,.232),(-.070,.239))],10)
    red_sections=[[(.017,.386),(.061,.367),(.085,.337),(.080,.330),(.016,.346)],[(.016,.327),(.089,.314),(.080,.280),(.003,.295)],[(.001,.278),(.069,.263),(.045,.207),(-.025,.213)]]
    for sign in (-1,1):
        wing=ellipsoid('Wing / dark outlined rounded Natu shield '+str(sign),INK,(0,.305,0),(.101,.132,.082),40,28);skin=surface(wing);p.append(transform(wing,sign))
        p.append(transform(fitted_contour('Wing gold / curved leading Natu feather '+str(sign),GOLD,yellow,skin,.007,subdivisions=2),sign))
        for j,xy in enumerate(red_sections):p.append(transform(fitted_contour('Wing red / barred trailing Natu feather '+str(sign)+' '+str(j),RED,xy,skin,.006,subdivisions=3),sign))
        back=reverse_front(wing);bs=surface(back)
        p.append(transform(reverse_front(fitted_contour('Wing gold / curved leading Natu feather '+str(sign)+' rear',GOLD,[(-x,y) for x,y in yellow],bs,.007,subdivisions=2)),sign))
        for j,xy in enumerate(red_sections):p.append(transform(reverse_front(fitted_contour('Wing red / barred trailing Natu feather '+str(sign)+' '+str(j)+' rear',RED,[(-x,y) for x,y in xy],bs,.006,subdivisions=3)),sign))
    for sign in (-1,1):
        p.append(curved_tube('Tail / broad swept red Natu feather '+str(sign),RED,[(0,.186,-.133),(sign*.066,.167,-.246),(sign*.116,.145,-.322)],[.043,.044,.002],24))
    return p

def bill():
    rows=[(.118,.049,.957,1.021),(.188,.065,.966,1.029),(.270,.050,.947,1.021),(.320,.025,.929,.970),(.340,.001,.921,.924)];v=[];n=8
    for z,w,low,high in rows:
        mid=(low+high)/2;v.extend((x,y,z) for x,y in ((-w*.6,low),(w*.6,low),(w,mid),(w*.75,high-.005),(w*.4,high),(-w*.4,high),(-w*.75,high-.005),(-w,mid)))
    f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
    for k in range(len(rows)-1):
        for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
    return crease_normals(part('Beak / long hooked golden Xatu bill',GOLD,v,f,True),35.)

def sleeve(sign):
    rows=curved_profiles(((.150,.050,.032,.078),(.168,.092,.052,.087),(.195,.108,.065,.086),(.237,.120,.069,.082),(.340,.124,.074,.074),(.490,.118,.074,.071),(.610,.089,.063,.061),(.710,.054,.039,.052),(.773,.003,.002,.041)),4)
    wing=loft('Wing / long folded ivory ceremonial sleeve '+str(sign),IVORY,rows,40)
    wing['positions']=[v+(sign*(.100+.040*max(0,(wing['positions'][i-i%3+1]-.56)/.213)) if i%3==0 else 0) for i,v in enumerate(wing['positions'])]
    wing['positions']=[v+(.018 if i%3==2 else 0) for i,v in enumerate(wing['positions'])]
    return wing

def band(p,wing,name,color,yl,yh,rear=False):
    support=reverse_front(wing) if rear else wing;rows={}
    for i in range(0,len(support['positions']),3):
        x,y,z=support['positions'][i:i+3]
        if yl-1e-6<=y<=yh+1e-6:rows.setdefault(round(y,7),[]).append(x)
    outline=[]
    for y,xs in sorted(rows.items()):outline.append(((min(xs)+max(xs))/2+(min(xs)-max(xs))*.465,y))
    for y,xs in reversed(sorted(rows.items())):outline.append(((min(xs)+max(xs))/2+(max(xs)-min(xs))*.465,y))
    patch=fitted_contour(name,color,outline,surface(support),.008,subdivisions=2);p.append(reverse_front(patch) if rear else patch)

def xatu():
    body=loft('Body / tall tapered green ceremonial torso',GREEN,curved_profiles(((.133,.148,.088,-.015),(.220,.211,.120,-.019),(.390,.229,.137,-.019),(.552,.187,.129,-.015),(.688,.126,.098,0),(.784,.090,.076,0),(.872,.079,.072,0),(.912,.075,.069,0)),4),40)
    head=ellipsoid('Head / broad rounded green Xatu face',GREEN,(0,.997,.015),(.178,.172,.149),48,28);p=[body,head];eyes(p,head,1.032,.093,.055,.024);feet(p,.168);p.append(bill());skin=surface(body)
    collar=[(-.066,.851),(.066,.851),(.090,.760),(.062,.784),(.032,.748),(0,.789),(-.036,.748),(-.063,.780),(-.090,.760)]
    p.append(fitted_contour('Collar / fitted dark jagged ceremonial ruff',INK,collar,skin,.008))
    for sign in (-1,1):
        eye=fitted_patch('Chest symbol / red watchful almond '+str(sign),RED,oval(sign*.046,.706,.034,.014,32),skin,.007);p.append(eye)
        p.append(fitted_patch('Chest pupil / narrow dark symbol slit '+str(sign),INK,oval(sign*.046,.706,.003,.011,20),surface(eye),.002))
        wing=sleeve(sign);p.append(wing);ws=surface(wing)
        badge=[(sign*.141,.651),(sign*.204,.591),(sign*.178,.561),(sign*.117,.629)];badge=[(x-sign*.016,y) for x,y in badge];badge=fitted_contour('Wing badge / fitted red diagonal sleeve panel '+str(sign),RED,badge,ws,.007);p.append(badge)
        xy=[(sign*.141,.635),(sign*.190,.590),(sign*.177,.578),(sign*.131,.630)]
        xy=[(x-sign*.016,y) for x,y in xy]
        p.append(fitted_contour('Wing badge gold / inset diagonal ceremonial stripe '+str(sign),GOLD,xy,surface(badge),.003))
        for rear in (False,True):
            band(p,wing,'Wing hem / fitted dark lower border '+str(sign)+' '+str(int(rear)),INK,.168,.195,rear)
            band(p,wing,'Wing hem red / fitted red lower ribbon '+str(sign)+' '+str(int(rear)),RED,.195,.237,rear)
        streamer=loft('Streamer / long swept red Xatu head ribbon '+str(sign),RED,curved_profiles(((.362,.017,.008,-.254),(.444,.036,.009,-.224),(.682,.032,.010,-.166),(.926,.025,.011,-.094),(1.111,.012,.010,-.031)),4),24)
        streamer['positions']=[v+(sign*(.156-.100*(streamer['positions'][i-i%3+1]-.362)/.749) if i%3==0 else 0) for i,v in enumerate(streamer['positions'])];p.append(streamer)
    rear=reverse_front(body)
    for j,y in enumerate((.219,.293,.367)):
        xy=[(-.195,y-.012),(.195,y-.012),(.195,y+.012),(-.195,y+.012)]
        p.append(reverse_front(fitted_contour('Back stripe / fitted golden ceremonial bar '+str(j),GOLD,xy,surface(rear),.006,subdivisions=3)))
    return p

def sculpture(species,normalize=True):
    if species not in SPECIES:raise ValueError('unsupported psychic bird')
    p=natu() if species=='natu' else xatu();p=[crease_normals(q,35.) if q['part'].startswith(('Eye /','Pupil /','Eye glint /','Wing gold /','Wing red /','Bill seam /','Collar /','Chest symbol /','Chest pupil /','Wing badge','Wing hem','Back stripe /')) else q for q in p]
    if not normalize:return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for species in SPECIES:(args.output/(species+'.mesh.json')).write_text(json.dumps(sculpture(species),separators=(',',':')))
if __name__=='__main__':main()
