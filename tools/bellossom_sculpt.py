#!/usr/bin/env python3
"""Original Bellossom dancer with layered leaves and two recessed crown flowers."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
HEIGHT=.95
GREEN=(.61,.77,.43);LEAF=(.30,.57,.37);GOLD=(.97,.83,.38);UNDER=(.23,.43,.33)
CORAL=(.94,.37,.25);ORANGE=(.99,.48,.30);CENTER=(.99,.85,.40);OLIVE=(.40,.38,.17)
INK=(.07,.11,.13);WHITE=(.98,.98,.91);PINK=(.92,.63,.67);CHEEK=(.89,.45,.26)

def skirt_leaf(name,color,angle,root_y,length,reach,width):
    n=20;rows=25;v=[];c=math.cos(angle);s=math.sin(angle)
    for k in range(rows):
        t=k/(rows-1);r=.052+reach*t**.82;y=root_y-length*t+.025*math.sin(math.pi*t)
        w=.004+width*math.sin(math.pi*t)**.85;d=.003+.014*math.sin(math.pi*t)
        for j in range(n):
            a=j*math.tau/n;u=w*math.sin(a);h=d*math.cos(a)
            v.append((r*s+u*c,y+h,r*c-u*s))
    f=[tuple(reversed(range(n))),tuple((rows-1)*n+j for j in range(n))]
    for k in range(rows-1):
        for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
    return crease_normals(part(name,color,v,f,True),35.)

def flower_center(sign):
    n=48;v=[];cx=sign*.174;cy=.853
    # Closed cup: thick outer wall, raised lip, descending inner wall and floor.
    for r,z in ((.060,.022),(.073,.090),(.075,.108),(.047,.116),(.042,.077)):
        v.extend((cx+r*math.sin(j*math.tau/n),cy+r*math.cos(j*math.tau/n),z) for j in range(n))
    f=[tuple(reversed(range(n))),tuple(4*n+j for j in range(n))]
    for k in range(4):
        for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
    return crease_normals(part('Flower center / recessed golden crown cup '+str(sign),CENTER,v,f,True),40.)

def sculpture():
    body=loft('Body / small continuous dancing green torso',GREEN,curved_profiles(((.160,.043,.042,0),(.239,.143,.112,0),(.350,.154,.121,0),(.438,.096,.087,.001),(.537,.061,.066,0),(.602,.077,.075,0)),4),36)
    head=ellipsoid('Head / broad pale green rounded dancer face',GREEN,(0,.661,.008),(.228,.190,.156),48,28);p=[body,head];hs=surface(head)
    for j in range(5):
        a=j*math.tau/5
        p.append(skirt_leaf('Skirt / dark rear underskirt leaf '+str(j),UNDER,a+.18,.338,.323,.300,.115))
        p.append(skirt_leaf('Skirt / curved golden inner leaf '+str(j),GOLD,a+math.pi/5,.368,.322,.370,.138))
        p.append(skirt_leaf('Skirt / broad green outer leaf '+str(j),LEAF,a,.395,.350,.410,.175))
    # Short rounded leaf-like arms, one raised and one crossing the torso.
    p.append(curved_tube('Arm / raised right dancing leaf',GREEN,[(.081,.480,.005),(.179,.482,.024),(.281,.516,.060),(.337,.550,.074)],[.060,.065,.053,.007],24))
    p.append(curved_tube('Arm / curved left leaf across chest',GREEN,[(-.084,.482,.014),(-.180,.452,.082),(-.152,.415,.164),(-.059,.414,.184),(.023,.437,.184)],[.055,.060,.059,.049,.006],24))
    for sign in (-1,1):
        x=sign*.089;eye=fitted_patch('Eye / glossy dark oval '+str(sign),INK,oval(x,.693,.038,.052,36),hs,.005);p.append(eye);es=surface(eye)
        p.append(fitted_patch('Eye glint / ivory upper catchlight '+str(sign),WHITE,oval(x-.008,.715,.014,.017,24),es,.002))
        p.append(fitted_patch('Eye glint / tiny lower catchlight '+str(sign),WHITE,oval(x+.011,.670,.006,.007,18),es,.0015))
        p.append(fitted_patch('Cheek / fitted orange blush '+str(sign),CHEEK,oval(sign*.135,.632,.033,.010,32),hs,.003))
    smile=contour([((-.054,.627),(-.016,.620),(.025,.624),(.054,.634)),((.054,.634),(.068,.550),(-.039,.551),(-.054,.627))],12)
    mouth=fitted_contour('Mouth / cheerful open fitted smile',INK,smile,hs,.006);p.append(mouth)
    tongue=contour([((-.039,.617),(-.012,.611),(.017,.616),(.042,.622)),((.042,.622),(.049,.565),(-.026,.565),(-.039,.617))],12)
    p.append(fitted_contour('Tongue / rosy dancer smile',PINK,tongue,surface(mouth),.003))
    # Six broad rounded petals on each crown flower, with gently curled tips.
    local=contour([((-.030,.038),(-.075,.066),(-.083,.172),(-.046,.196)),((-.046,.196),(-.017,.215),(.023,.203),(.056,.181)),((.056,.181),(.073,.123),(.064,.066),(.028,.038)),((.028,.038),(.008,.028),(-.010,.028),(-.030,.038))],9)
    for sign in (-1,1):
        cx=sign*.174;cy=.853
        for j in range(6):
            a=j*math.tau/6+sign*.15;c=math.cos(a);s=math.sin(a)
            xy=[(cx+x*c-y*s,cy+x*s+y*c) for x,y in local]
            p.append(fitted_contour('Flower petal / broad curled coral ray '+str(sign)+' '+str(j),CORAL if j%2 else ORANGE,xy,lambda y,x:.034-.055*(math.hypot(x-cx,y-cy)-.08),.030,subdivisions=1))
        cup=flower_center(sign);p.append(cup)
        p.append(fitted_patch('Flower well / deep olive center '+str(sign),OLIVE,oval(cx,cy,.034,.034,36),surface(cup),.0015))
    p=[crease_normals(q,35.) if q['part'].startswith(('Eye /','Eye glint /','Cheek /','Mouth /','Tongue /','Flower well /')) else q for q in p]
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'bellossom','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    (args.output/'bellossom.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
