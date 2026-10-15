#!/usr/bin/env python3
"""Original Sunkern seed cup and joyful Sunflora with cupped ray petals."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,fitted_patch,fitted_contour,crease_normals,tube
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube,reverse_front
from wing_insect_sculpt import contour,membrane
from skarmory_sculpt import blade
from skin_glb import COORDINATES
SPECIES=('sunkern','sunflora');HEIGHT=.95
GOLD=(239/255,231/255,8/255);FACE=(1.,1.,1.);PETAL=(214/255,181/255,24/255);BRAC=PETAL
GREEN=(82/255,173/255,0.);LEAF=GREEN;SEED_GREEN=(107/255,148/255,66/255);VEIN=(.24,.45,.25);INK=(0.,0.,0.);WHITE=(1.,1.,1.);MOUTH_INSET=WHITE

def seed_shell():
    n=64;rows=curved_profiles(((.08,.017,.015,0),(.13,.129,.098,0),(.24,.219,.158,0),(.39,.234,.178,0),(.52,.207,.164,0),(.548,.190,.152,0)),4);v=[]
    for y,w,d,z in rows:
        v.extend((w*math.sin(j*math.tau/n),y,d*math.cos(j*math.tau/n)) for j in range(n))
    for j in range(n):
        a=j*math.tau/n;wave=2*abs(2*((j*8/n)%1)-1)-1;v.append((.190*math.sin(a),.625+.062*wave,.152*math.cos(a)))
    outer_rows=len(rows)+1;inner=len(v)
    for y,w,d in ((None,.172,.134),(.535,.155,.118)):
        for j in range(n):
            a=j*math.tau/n;wave=2*abs(2*((j*8/n)%1)-1)-1;v.append((w*math.sin(a),.625+.062*wave if y is None else y,d*math.cos(a)))
    f=[tuple(reversed(range(n)))]
    for k in range(outer_rows-1):
        for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
    rim=(outer_rows-1)*n
    for j in range(n):
        a=inner+j;b=inner+(j+1)%n;f.append((a,b,b+n,a+n));f.append((rim+j,rim+(j+1)%n,inner+(j+1)%n,inner+j))
    f.append(tuple(inner+n+j for j in range(n)))
    return crease_normals(part('Body / hollow gold seed shell and eight serrated crown teeth',GOLD,v,f,True),35.)

def union_skin(parts):
    skins=[surface(p) for p in parts]
    def at(y,x):
        hits=[]
        for skin in skins:
            try:hits.append(skin(y,x))
            except ValueError:pass
        if not hits:raise ValueError(('outside composite skin',x,y))
        return max(hits)
    return at

def midrib(p,leaf,name,xy):
    skin=surface(leaf);p.append(tube(name,VEIN,[(x,y,skin(y,x)+.0015) for x,y in xy],[.0028]*len(xy),12))

def sunkern():
    body=seed_shell();p=[body];bs=surface(body)
    panel=[(-.205,.445),(-.185,.505),(-.157,.477),(-.126,.509),(-.106,.476),(-.073,.508),(-.070,.390),(-.045,.240),(-.014,.108),(-.052,.125),(-.141,.178),(-.202,.275)]
    fronts=[]
    for s in (-1,1):
        xy=[(-s*x,y) for x,y in panel];coat=fitted_contour('Coat / broad fitted dark front seed stripe '+str(s),SEED_GREEN,xy,bs,.008,subdivisions=3);p.append(coat);fronts.append(coat)
        back=fitted_contour('Coat / broad fitted dark rear seed stripe '+str(s),SEED_GREEN,xy,surface(reverse_front(body)),.008,subdivisions=3);p.append(reverse_front(back))
    skin=union_skin([body]+fronts)
    for s in (-1,1):
        x=s*.127;eye=fitted_patch('Eye / large black seed button '+str(s),INK,oval(x,.367,.040,.048,32),skin,.007);p.append(eye);es=surface(eye)
        p.append(fitted_patch('Eye glint / broad ivory seed catchlight '+str(s),WHITE,oval(x-.001,.361,.016,.020,24),es,.002))
        p.append(fitted_patch('Eye glint / tiny upper seed sparkle '+str(s),WHITE,oval(x-.014,.393,.005,.007,16),es,.003))
    smile=contour([((-.072,.309),(-.022,.285),(.031,.301),(.080,.309)),((.080,.309),(.055,.251),(.037,.234),(0,.239)),((0,.239),(-.033,.237),(-.061,.280),(-.072,.309))],10)
    mouth=fitted_contour('Mouth / fitted playful seed grin',INK,smile,skin,.009);p.append(mouth)
    tongue=contour([((-.050,.293),(-.025,.283),(.028,.293),(.054,.300)),((.054,.300),(.038,.253),(.026,.246),(0,.245)),((0,.245),(-.020,.244),(-.042,.265),(-.050,.293))],10)
    p.append(fitted_contour('Tongue / rosy seed smile',MOUTH_INSET,tongue,surface(mouth),.003,subdivisions=3))
    p.append(curved_tube('Sprout / thin green central stem',SEED_GREEN,[(0,.518,0),(0,.628,-.006),(0,.718,-.008)],[.014,.013,.012],18))
    leaf_outline=contour([((0,.704),(.104,.820),(.235,.861),(.335,.817)),((.335,.817),(.297,.704),(.121,.650),(0,.704))],12)
    for s in (-1,1):
        leaf=membrane('Leaf / broad curved seed sprout '+str(s),SEED_GREEN,leaf_outline,s,root_z=-.008,thickness=.015);p.append(leaf)
        midrib(p,leaf,'Leaf vein / fitted seed sprout midrib '+str(s),[(s*x,y) for x,y in ((.018,.712),(.128,.763),(.266,.804))])
    return p

def petal_outline(angle,small):
    if small:
        local=contour([((-.041,.119),(-.055,.195),(-.027,.257),(0,.293)),((0,.293),(.040,.252),(.052,.186),(.040,.120)),((.040,.120),(.014,.111),(-.015,.111),(-.041,.119))],8)
    else:
        local=contour([((-.068,.138),(-.100,.205),(-.110,.310),(-.070,.355)),((-.070,.355),(-.050,.385),(.003,.391),(.027,.391)),((.027,.391),(.075,.391),(.113,.342),(.100,.275)),((.100,.275),(.102,.190),(.035,.134),(-.068,.138))],10)
    c=math.cos(angle);s=math.sin(angle);return [(x*c-y*s,.885+x*s+y*c) for x,y in local]

def sunflora():
    body=loft('Body / plump green bulb and slender rising flower stem',GREEN,curved_profiles(((.080,.041,.035,0),(.151,.112,.081,0),(.276,.156,.114,0),(.387,.137,.101,0),(.484,.079,.059,0),(.609,.043,.038,0),(.730,.038,.035,0)),4),36)
    face=ellipsoid('Face / broad rounded pale yellow sunflower disk',FACE,(0,.885,.025),(.225,.225,.080),48,28);p=[body,face];fs=surface(face)
    for j in range(8):
        a=j*math.tau/8+math.pi/8;outline=petal_outline(a,True)
        p.append(blade('Bract / pointed rear golden ray '+str(j),BRAC,outline,lambda x,y:-.041,.018))
    for j in range(8):
        outline=petal_outline(j*math.tau/8,False)
        p.append(blade('Petal / curved broad golden sunflower ray '+str(j),PETAL,outline,lambda x,y:-.026+.09*(math.hypot(x,y-.885)-.15),.024))
    for s in (-1,1):
        x=s*.082;y=.938
        outline=contour([((x-.034,y-.004),(x-.020,y+.031),(x+.022,y+.031),(x+.034,y-.004)),((x+.034,y-.004),(x+.018,y+.020),(x-.018,y+.020),(x-.034,y-.004))],10)
        p.append(fitted_contour('Eye / fitted joyful closed sunflower arc '+str(s),INK,outline,fs,.004))
    smile=contour([((-.088,.867),(-.053,.841),(.044,.830),(.091,.872)),((.091,.872),(.079,.747),(-.068,.750),(-.088,.867))],12)
    mouth=fitted_contour('Mouth / fitted broad laughing sunflower grin',INK,smile,fs,.006);p.append(mouth)
    cx=sum(x for x,y in smile)/len(smile);cy=sum(y for x,y in smile)/len(smile)
    tongue=[(x*.85,y-.015 if k<12 else y+.011*math.sin(math.pi*(k-12)/12)-.015*abs(2*(k-12)/12-1)**2) for k,(x,y) in enumerate(smile)]
    p.append(fitted_contour('Tongue / fitted rosy sunflower smile',MOUTH_INSET,tongue,surface(mouth),.003))
    left=contour([((-.070,.430),(-.200,.510),(-.380,.470),(-.530,.300)),((-.530,.300),(-.360,.290),(-.230,.250),(-.070,.430))],14)
    right=contour([((.075,.450),(.200,.500),(.330,.400),(.460,.530)),((.460,.530),(.490,.580),(.500,.610),(.490,.650)),((.490,.650),(.570,.410),(.340,.300),(.075,.450))],12)
    for s,outline,xy in ((-1,left,[(-.110,.429),(-.260,.397),(-.430,.331)]),(1,right,[(.100,.451),(.220,.451),(.360,.441),(.494,.560)])):
        leaf=membrane('Leaf arm / cupped asymmetrical green blade '+str(s),LEAF,outline,1,root_z=.012,thickness=.019);p.append(leaf);midrib(p,leaf,'Leaf vein / fitted sunflower arm midrib '+str(s),xy)
        p.append(curved_tube('Leg / curved supporting green root '+str(s),GREEN,[(s*.055,.145,0),(s*.085,.059,.020),(s*.182,.035,.036)],[.043,.033,.026],22))
        foot=ellipsoid('Foot / grounded broad green root pad '+str(s),GREEN,(s*.195,.033,.052),(.110,.033,.076),32,20);p.append(foot)
    return p

def sculpture(species,normalize=True):
    if species not in SPECIES:raise ValueError('unsupported sunflower')
    p=sunkern() if species=='sunkern' else sunflora();p=[crease_normals(q,35.) if q['part'].startswith(('Coat /','Eye /','Eye glint /','Mouth /','Tongue /')) else q for q in p]
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo) if normalize else 1.
    for q in p:
        q['positions']=[round((v-(lo if normalize and i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
        q['positions']=[0. if v==0. else v for v in q['positions']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))
if __name__=='__main__':main()
