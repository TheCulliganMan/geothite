#!/usr/bin/env python3
"""Original sly Murkrow with feather hat, hooked bill and bound broom tail."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch,crease_normals,translate
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from wing_insect_sculpt import membrane,contour
from skarmory_sculpt import blade
from skin_glb import COORDINATES
SPECIES=('murkrow',);HEIGHT=.78
COAT=(.18,.29,.38);FEATHER=(.22,.34,.44);DARK=(.105,.17,.23);GOLD=(.91,.81,.49);WHITE=(.98,.97,.92);INK=(.07,.09,.12);RED=(.81,.29,.24)

def bill(name,color,rows):
    n=8;v=[]
    for z,w,low,high in rows:
        mid=(low+high)/2
        v.extend((x,y,z) for x,y in ((-w*.6,low),(w*.6,low),(w,mid),(w*.75,high-.18*(high-low)),(w*.3,high),(-w*.3,high),(-w*.75,high-.18*(high-low)),(-w,mid)))
    f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
    for j in range(len(rows)-1):
        for i in range(n):a=j*n+i;b=j*n+(i+1)%n;f.append((a,b,b+n,a+n))
    return crease_normals(part(name,color,v,f),40.)

def sculpture(species='murkrow'):
    if species not in SPECIES:raise ValueError('unsupported witch crow')
    body=loft('Body / rounded dark crow breast and abdomen',COAT,curved_profiles(((.201,.026,.031,-.012),(.26,.122,.122,-.013),(.365,.184,.166,-.016),(.465,.174,.154,-.007),(.55,.131,.117,.001),(.612,.067,.067,.008)),4),36)
    neck=loft('Neck / narrow dark crow neck',COAT,curved_profiles(((.535,.065,.060,.007),(.645,.050,.051,.019),(.738,.058,.060,.020)),4),28)
    head=ellipsoid('Head / sly rounded dark crow head',COAT,(0,.800,.019),(.166,.158,.145),36,24);p=[body,neck,head];hs=surface(head)
    # A broad shallow brim, sloping forward over the eyes, is actual geometry.
    brim=ellipsoid('Hat / broad sloping feather brim',COAT,(0,.929,-.008),(.287,.026,.195),48,14)
    v=[tuple(brim['positions'][i:i+3]) for i in range(0,len(brim['positions']),3)];v=[(x,y-.16*z,z) for x,y,z in v]
    p.append(crease_normals(part(brim['part'],COAT,v,[tuple(brim['indices'][i:i+3]) for i in range(0,len(brim['indices']),3)]),40.))
    crowns=[contour([((-.078,.939),(-.076,1.082),(.055,1.268),(.109,1.299)),((.109,1.299),(.11,1.171),(.075,1.011),(.078,.941)),((.078,.941),(.024,.930),(-.039,.927),(-.078,.939))],8),
      contour([((-.136,.939),(-.155,1.054),(-.152,1.147),(-.125,1.196)),((-.125,1.196),(-.100,1.136),(-.064,1.025),(-.028,.947)),((-.028,.947),(-.071,.930),(-.103,.93),(-.136,.939))],8),
      contour([((.047,.944),(.114,1.057),(.178,1.081),(.224,1.095)),((.224,1.095),(.204,1.031),(.175,.978),(.133,.946)),((.133,.946),(.102,.932),(.073,.933),(.047,.944))],8)]
    for j,outline in enumerate(crowns):
        c=COAT if j==0 else FEATHER;crest=membrane('Crest / swept pointed hat feather '+str(j),c,outline,1,root_z=-.037-j*.015,thickness=.028)
        top=max(y for x,y in outline);v=[]
        for k in range(0,len(crest['positions']),3):
            x,y,z=crest['positions'][k:k+3];mid=-.037-j*.015-.06*abs(x)+.025*(y-.70)**2
            v.append((x,y,mid+(z-mid)*(1-.88*max(0,(y-.938)/(top-.938))**2)))
        p.append(crease_normals(part(crest['part'],c,v,[tuple(crest['indices'][k:k+3]) for k in range(0,len(crest['indices']),3)]),35.))
    for s in (-1,1):
        x=s*.093;outline=[(x-s*.037,.890),(x+s*.037,.879),(x+s*.029,.816),(x-s*.027,.823)]
        eye=fitted_patch('Eye / fitted sly dark socket '+str(s),INK,outline,hs,.008);p.append(eye)
        cx=sum(a for a,b in outline)/4;cy=sum(b for a,b in outline)/4
        white=fitted_patch('Eye / fitted ivory white '+str(s),WHITE,[(cx+(a-cx)*.88,cy+(b-cy)*.90) for a,b in outline],surface(eye),.003);p.append(white)
        iris=fitted_patch('Eye / red vertical iris '+str(s),RED,oval(x,.849,.013,.024,24),surface(white),.003);p.append(iris)
        pupil=fitted_patch('Pupil / narrow dark crow pupil '+str(s),INK,oval(x,.849,.005,.018,20),surface(iris),.002);p.append(pupil)
        p.append(fitted_patch('Eye / tiny ivory glint '+str(s),WHITE,oval(x-.003,.862,.002,.004,12),surface(iris),.004))
    upper=bill('Bill / broad hooked golden upper beak',GOLD,[(.135,.068,.774,.823),(.215,.091,.753,.827),(.294,.062,.734,.794),(.340,.015,.708,.753),(.348,.0017,.702,.708)]);p.append(upper)
    p.append(bill('Bill / narrow dark beak seam',INK,[(.139,.065,.768,.773),(.207,.076,.751,.756),(.273,.038,.740,.745),(.30,.002,.739,.743)]))
    p.append(bill('Bill / short golden lower beak',GOLD,[(.137,.062,.726,.767),(.209,.071,.729,.753),(.265,.032,.735,.741),(.289,.002,.738,.740)]))
    us=surface(upper)
    for s in (-1,1):p.append(fitted_patch('Nostril / subtle fitted beak fleck '+str(s),DARK,oval(s*.042,.786,.005,.003,16),us,.002))
    # Four pointed throat feathers bridge the narrow neck into the breast.
    for j,x in enumerate((-.071,-.027,.027,.071)):
        p.append(membrane('Throat / pointed dark feather '+str(j),COAT,[(x-.020,.598),(x-.043,.679),(x+.007,.648),(x+.031,.676),(x+.020,.596)],1,root_z=.063,thickness=.022))
    for s in (-1,1):
        p.append(translate(loft('Wing / broad folded crow wing '+str(s),COAT,curved_profiles(((.241,.010,.016,-.095),(.31,.052,.078,-.058),(.42,.071,.126,-.014),(.53,.040,.065,.035),(.561,.013,.02,.048)),4),28),s*.150,0,0))
        for j in range(3):
            zz=.040-j*.043;y=.487-j*.030
            feather=[(zz+.054,y+.046),(zz-.070,y-.043),(zz-.123,y-.125),(zz-.103,y-.166),(zz-.056,y-.117),(zz+.037,y-.015)]
            p.append(blade(f'Wing feather / layered folded vane {s} {j}',FEATHER if j==1 else COAT,feather,lambda z,y:-s*(.193+j*.005),.010,vertical=True))
        p.append(curved_tube('Leg / slender golden crow shin '+str(s),GOLD,[(s*.071,.262,.001),(s*.074,.133,.008),(s*.075,.036,.024)],[.018,.016,.017],18))
        foot=ellipsoid('Foot / grounded golden toe junction '+str(s),GOLD,(s*.075,.020,.028),(.047,.020,.044),24,16);p.append(foot)
        for j,dx in enumerate((-.026,0,.026)):
            start=(s*.075+dx*.35,.027,.045);end=(s*.075+dx*1.5,.018,.110)
            p.append(curved_tube(f'Toe / three golden forward toes {s} {j}',GOLD,[start,(s*.075+dx,.022,.079),end],[.009,.009,.007],14))
            p.append(curved_tube(f'Talon / white forward toe tip {s} {j}',WHITE,[(end[0],end[1]+.001,end[2]-.006),(end[0],.013,.136),(end[0],.006,.152)],[.009,.006,.0012],14))
        p.append(curved_tube('Toe / short golden rear toe '+str(s),GOLD,[(s*.075,.027,.011),(s*.078,.017,-.036),(s*.082,.015,-.053)],[.009,.008,.006],14))
        p.append(curved_tube('Talon / white rear toe tip '+str(s),WHITE,[(s*.082,.016,-.047),(s*.084,.007,-.076)],[.007,.0012],14))
    p.append(curved_tube('Tail / short supporting broom stem',COAT,[(0,.383,-.125),(0,.415,-.222),(0,.474,-.315)],[.033,.030,.026],20))
    p.append(curved_tube('Binding / narrow red broom band',RED,[(0,.414,-.218),(0,.432,-.249)],[.033,.031],22))
    fan=[(-.035,.461),(-.223,.562),(-.306,.658),(-.158,.643),(-.151,.724),(-.028,.664),(.044,.740),(.091,.661),(.269,.674),(.184,.566),(.237,.527),(.080,.466)]
    broom=membrane('Tail fan / broad jagged broom feather fan',COAT,fan,1,root_z=-.305,thickness=.020);p.append(broom);bs=surface(broom)
    for j,(x,y) in enumerate(((-.211,.594),(-.110,.645),(-.017,.640),(.072,.632),(.166,.575))):
        xy=[(x*.17,.483),(x*.55,.483+(y-.483)*.5),(x,y)]
        p.append(tube('Tail fold / fitted radiating broom crease '+str(j),FEATHER,[(xx,yy,bs(yy,xx)+.001) for xx,yy in xy],[.0016,.0018,.0008],10))
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'murkrow.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
