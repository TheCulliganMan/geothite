#!/usr/bin/env python3
"""Original cheerful Wobbuffet: elastic silhouette, salute and watchful tail."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
SPECIES=('wobbuffet',);HEIGHT=1.14
# Local Crystal normal RGB555: body (12,21,31), mouth shade (7,14,16).
# Geometry helpers convert these sRGB values to linear material colors once.
BLUE=(99/255,173/255,1.);BLACK=(0.,0.,0.);WHITE=(1.,1.,1.);MOUTH=BLACK;TONGUE=(57/255,115/255,132/255)

def sculpture(species='wobbuffet',normalize=True):
    if species not in SPECIES:raise ValueError('unsupported patient pokemon')
    body=loft('Body / continuous tapered elastic blue silhouette',BLUE,curved_profiles(((.061,.065,.057,.0),(.14,.107,.090,.0),(.30,.115,.111,-.005),(.49,.169,.147,-.005),(.68,.222,.184,0),(.86,.250,.197,0),(1.014,.222,.172,0),(1.102,.133,.113,0),(1.129,.012,.013,0)),5),48)
    p=[body];bs=surface(body)
    # Broad continuous upper limbs with rounded ends; one folds into a salute.
    p.append(curved_tube('Arm / bent overhead salute',BLUE,[(-.174,.718,-.010),(-.278,.855,-.012),(-.307,1.068,.001),(-.294,1.228,.004),(-.172,1.260,.024),(-.071,1.273,.027)],[.078,.071,.061,.058,.063,.046],28))
    p.append(ellipsoid('Hand / rounded salute tip',BLUE,(-.071,1.273,.027),(.055,.046,.050),28,18))
    p.append(curved_tube('Arm / raised outward elastic arm',BLUE,[(.180,.727,0),(.277,.875,.002),(.327,1.023,.0),(.333,1.183,.004)],[.079,.075,.066,.052],28))
    p.append(ellipsoid('Hand / rounded upright tip',BLUE,(.333,1.185,.004),(.053,.061,.054),28,18))
    # Four soft basal lobes, with no unrelated separate humanoid legs.
    for s in (-1,1):
        p.append(ellipsoid('Foot / broad grounded front lobe '+str(s),BLUE,(s*.103,.059,.071),(.108,.059,.097),32,20))
        p.append(ellipsoid('Foot / small rear lobe '+str(s),BLUE,(s*.095,.069,-.067),(.080,.055,.071),28,18))
    for s in (-1,1):
        xy=[(s*x,y) for x,y in ((.169,1.040),(.098,1.003),(.166,.970),(.171,.984),(.128,1.003),(.174,1.026))]
        p.append(fitted_contour('Eye / fitted squeezed chevron '+str(s),BLACK,xy,bs,.004))
    mouth_outline=[(-.157,.910),(-.127,.940),(-.094,.917),(-.061,.932),(-.029,.904),(.014,.914),(.047,.890),(.084,.898),(.113,.875),(.145,.879),(.168,.852)]
    mouth_outline+=contour([((.168,.852),(.139,.707),(.087,.648),(.015,.638)),((.015,.638),(-.091,.626),(-.160,.704),(-.164,.796)),((-.164,.796),(-.166,.852),(-.163,.887),(-.157,.910))],8)[1:-1]
    mouth=fitted_contour('Mouth / fitted broad zigzag open smile',MOUTH,mouth_outline,bs,.006);p.append(mouth);ms=surface(mouth)
    tongue_outline=contour([((-.125,.733),(-.067,.775),(.058,.764),(.119,.719)),((.119,.719),(.088,.671),(.045,.647),(.015,.649)),((.015,.649),(-.049,.646),(-.102,.677),(-.125,.733))],10)
    p.append(fitted_contour('Tongue / fitted rounded lower mouth',TONGUE,tongue_outline,ms,.003))
    # A black tail grows out of the narrow back and curls into a soft paddle.
    stem=curved_tube('Tail / flexible black rear stem',BLACK,[(.042,.166,-.058),(.164,.166,-.103),(.341,.199,-.116),(.431,.266,-.124)],[.037,.035,.042,.058],24);p.append(stem)
    tail=ellipsoid('Tail paddle / rounded watchful black paddle',BLACK,(.468,.306,-.130),(.136,.094,.059),40,24)
    # Tilt the paddle upward while preserving its front-facing eye surface.
    ang=.45;c=math.cos(ang);sn=math.sin(ang)
    for field in ('positions','normals'):
        vals=[]
        for i in range(0,len(tail[field]),3):
            x,y,z=tail[field][i:i+3]
            if field=='positions':x-=.468;y-=.306
            xx=x*c-y*sn;yy=x*sn+y*c
            if field=='positions':xx+=.468;yy+=.306
            vals.extend((xx,yy,z))
        tail[field]=vals
    p.append(tail);ts=surface(tail)
    for j,(x,y) in enumerate(((.430,.318),(.511,.351))):
        white=fitted_patch('Tail eye / fitted ivory oval '+str(j),WHITE,oval(x,y,.025,.030,28),ts,.004);p.append(white)
        pupil=fitted_patch('Tail pupil / small dark watchful eye '+str(j),BLACK,oval(x+.002,y+.001,.008,.012,20),surface(white),.002);p.append(pupil)
    p=[crease_normals(q,35.) if q['part'].startswith(('Mouth /','Tongue /','Tail eye /','Tail pupil /')) else q for q in p]
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    if normalize:
        for q in p:
            q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'wobbuffet.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
