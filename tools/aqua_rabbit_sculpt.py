#!/usr/bin/env python3
"""Original Marill/Azumarill papercraft anatomy; canonical meshes only."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,tube,fitted_patch,crease_normals,curved_profiles
from abra_sculpt import surface
from chikorita_sculpt import linear
from skin_glb import COORDINATES
HEIGHT=1.02
BLUE=(.23,.70,.85);WHITE=(.97,.97,.92);RED=(.85,.39,.40)
INK=(.08,.13,.16);PINK=(.90,.57,.63)


def oval(cx,cy,rx,ry,n=16):return [(cx+rx*math.cos(i*math.tau/n),cy+ry*math.sin(i*math.tau/n)) for i in range(n)]


def body(adult):
    # Both colors belong to the same anatomical surface and share seam edges.
    profiles=([(.018,.018,.024,-.015),(.052,.145,.115,-.015),(.160,.211,.173,-.018),(.282,.265,.223,-.009),(.368,.290,.250,-.004),(.505,.300,.255,0.),(.646,.268,.223,.004),(.780,.208,.177,.008),(.867,.123,.112,.005),(.898,.018,.023,0.)] if adult else
              [(.017,.018,.022,0.),(.047,.145,.123,0.),(.130,.211,.179,0.),(.248,.266,.228,0.),(.361,.300,.256,0.),(.491,.280,.239,0.),(.603,.216,.185,0.),(.680,.118,.107,0.),(.706,.016,.022,0.)])
    seam_y=profiles[3][0];profiles=curved_profiles(profiles);seam_index=next(i for i,p in enumerate(profiles) if p[0]==seam_y)
    n=40;v=[];groups=[[],[]]
    for k,(y,w,d,z) in enumerate(profiles):
        for i in range(n):
            a=math.tau*i/n;front=(1+math.cos(a))/2
            seam=(.013*math.cos(a*6) if adult else 0.)+.018*(1-front)
            v.append((w*math.sin(a),y+(seam if k==seam_index else 0.),z+d*math.cos(a)))
    groups[0].append(tuple(reversed(range(n))));groups[1].append(tuple((len(profiles)-1)*n+i for i in range(n)))
    for k in range(len(profiles)-1):
        for i in range(n):
            a=k*n+i;b=k*n+(i+1)%n;groups[0 if k<seam_index else 1].append((a,b,b+n,a+n))
    whole=crease_normals(part('Body / joined water rabbit',BLUE,v,groups[0]+groups[1]));split=sum(len(f)-2 for f in groups[0])*3
    out=[]
    for name,color,start,end in [('Body / welded pale lower belly',WHITE,0,split),('Body / welded blue upper form',BLUE,split,len(whole['indices']))]:
        ids=whole['indices'][start:end];used=sorted(set(ids));remap={old:new for new,old in enumerate(used)}
        out.append({'part':name,'base_color':linear(color),'positions':[c for old in used for c in whole['positions'][old*3:old*3+3]],'normals':[c for old in used for c in whole['normals'][old*3:old*3+3]],'indices':[remap[i] for i in ids]})
    return out,whole


def ribbon(name,color,path,widths,depths):
    # Broad elliptical cross sections follow the actual bent paddle/ear path.
    n=12;v=[]
    for k,(x,y,z) in enumerate(path):
        a=path[max(0,k-1)];b=path[min(len(path)-1,k+1)];dx=b[0]-a[0];dy=b[1]-a[1];length=math.hypot(dx,dy)
        nx,ny=dy/length,-dx/length
        for j in range(n):
            angle=math.tau*j/n;w=widths[k]*math.cos(angle)
            v.append((x+nx*w,y+ny*w,z+depths[k]*math.sin(angle)))
    fs=[tuple(reversed(range(n))),tuple((len(path)-1)*n+i for i in range(n))]
    for k in range(len(path)-1):
        for i in range(n):a=k*n+i;b=k*n+(i+1)%n;fs.append((a,b,b+n,a+n))
    return part(name,color,v,fs)


def ear_lining(name,rows,skin):
    # One closed red strip follows the front skin, including the folded tip.
    dense=[]
    for a,b in zip(rows,rows[1:]):
        for step in range(5):
            t=step/5;dense.append(tuple(a[k]*(1-t)+b[k]*t for k in range(3)))
    dense.append(rows[-1]);columns=9
    xy=[(x+(j/(columns-1)*2-1)*w,y) for x,y,w in dense for j in range(columns)];n=len(xy)
    v=[(x,y,skin(y,x)+.007) for x,y in xy]+[(x,y,skin(y,x)-.001) for x,y in xy]
    fs=[]
    for k in range(len(dense)-1):
        for j in range(columns-1):a=k*columns+j;fs.append((a,a+1,a+columns+1,a+columns))
    fs += [tuple(i+n for i in reversed(f)) for f in list(fs)]
    edge=list(range(columns))+[k*columns+columns-1 for k in range(1,len(dense))]+list(range(n-2,n-columns-1,-1))+[k*columns for k in reversed(range(1,len(dense)-1))]
    fs += [(b,a,a+n,b+n) for a,b in zip(edge,edge[1:]+edge[:1])]
    return part(name,RED,v,fs)


def sculpture(species):
    if species not in ('marill','azumarill'):raise ValueError(species)
    adult=species=='azumarill';p,skin=body(adult);surf=surface(skin)
    for s in (-1,1):
        y=.723 if adult else .527;x=s*(.105 if adult else .108)
        p.append(fitted_patch('Eye / fitted dark oval '+str(s),INK,oval(x,y,.024,.032),surf,.005))
        p.append(fitted_patch('Eye / small white glint '+str(s),WHITE,oval(x-.006,y+.011,.009,.010,12),surf,.010))
        if adult:
            if s==-1:
                path=[(-.124,.838,.008),(-.172,.989,.012),(-.218,1.149,.026),(-.243,1.216,.061),(-.333,1.226,.151),(-.379,1.226,.181)]
                widths=[.052,.063,.073,.080,.055,.009];depths=[.031,.032,.034,.035,.025,.009]
                rows=[(-.139,.897,.027),(-.172,.987,.037),(-.214,1.132,.041),(-.240,1.187,.037),(-.266,1.210,.028)]
            else:
                path=[(.124,.838,.008),(.182,1.006,.010),(.258,1.180,.012),(.323,1.278,.010),(.346,1.297,.010)]
                widths=[.052,.072,.082,.065,.012];depths=[.031,.034,.035,.031,.010]
                rows=[(.144,.892,.025),(.181,1.005,.042),(.252,1.159,.049),(.300,1.238,.030),(.315,1.259,.010)]
            ear=ribbon('Ear / bent rabbit silhouette '+str(s),BLUE,path,widths,depths);p.append(ear)
            p.append(ear_lining('Ear / fitted red lining '+str(s),rows,surface(ear)))
        else:
            ear=ellipsoid('Ear / round mouse silhouette '+str(s),BLUE,(s*.233,.663,.003),(.101,.108,.056),20,10);p.append(ear)
            p.append(fitted_patch('Ear / fitted red inset '+str(s),RED,oval(s*.235,.667,.066,.075,20),surface(ear),.004))
        if adult:
            path=[(s*.263,.447,.008),(s*.354,.393,.030),(s*.435,.382,.064),(s*.466,.393,.077)];width=[.053,.063,.052,.009];depth=[.043,.030,.024,.009]
        else:
            path=[(s*.251,.341,.023),(s*.313,.315,.042),(s*.364,.335,.085),(s*.381,.345,.100)];width=[.043,.048,.032,.008];depth=[.035,.027,.021,.008]
        p.append(ribbon('Arm / flattened curved paddle '+str(s),BLUE,path,width,depth))
        p.append(ellipsoid('Foot / planted blue paddle '+str(s),BLUE,(s*(.180 if adult else .154),.040,.105),(.091,.040,.123) if adult else (.062,.040,.084),16,10))
    y=.629 if adult else .442
    # Small open smile is fitted into the face instead of being a projecting rod.
    outline=[(-.052,y+.014),(.052,y+.014),(.031,y-.031),(0.,y-.043),(-.031,y-.031)]
    p.append(fitted_patch('Mouth / fitted open smile',INK,outline,surf,.004))
    p.append(fitted_patch('Mouth / quiet pink tongue',PINK,[(-.026,y-.023),(.026,y-.023),(.018,y-.035),(0.,y-.038),(-.018,y-.035)],surf,.008))
    p.append(fitted_patch('Nose / tiny fitted dot',INK,oval(0.,y+.052,.005,.003,12),surf,.004))
    if adult:
        for i,(x,y,r) in enumerate(((-.204,.397,.028),(-.116,.470,.046),(.051,.485,.038),(.164,.521,.043),(-.114,.568,.020),(.180,.402,.015))):
            p.append(fitted_patch('Belly / fitted white spot '+str(i),WHITE,oval(x,y,r,r,16),surf,.004))
    # A visible zigzag wire exits the rear flank; the ball is its actual endpoint.
    tail=[(.219,.217,-.151),(.350,.190,-.229),(.329,.292,-.248),(.473,.266,-.277),(.453,.374,-.300),(.586,.361,-.307)] if adult else [(-.218,.262,-.127),(-.318,.226,-.211),(-.376,.361,-.234),(-.447,.301,-.244),(-.489,.461,-.252),(-.559,.499,-.256)]
    p.append(tube('Tail / continuous zigzag cord',INK,tail,[.008]*len(tail),8))
    end=tail[-1];p.append(ellipsoid('Tail / blue buoy at cord endpoint',BLUE,end,(.087,.087,.087) if adult else (.100,.100,.100),18,10))
    p=[q if q['part'].startswith('Body /') else crease_normals(q) for q in p]
    floor=min(v for q in p for v in q['positions'][1::3]);height=max(v for q in p for v in q['positions'][1::3])-floor
    for q in p:
        q['positions']=[round((v-(floor if i%3==1 else 0.))*HEIGHT/height,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}


def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--out',type=Path,default=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species');a=a.parse_args()
    for name in ('marill','azumarill'):
        m=sculpture(name);(a.out/(name+'.mesh.json')).write_text(json.dumps(m,separators=(',',':')));print(name,len(m['primitives']),sum(len(p['indices'])//3 for p in m['primitives']))
if __name__=='__main__':main()
