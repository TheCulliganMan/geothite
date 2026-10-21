#!/usr/bin/env python3
"""Original compact Spearow volume from inspected game orbits and Crystal frames."""
import math,struct
from sculpt_geometry import loft,ellipsoid,tube,fitted_triangle_contour,fitted_contour,crease_normals
from pidgeotto_sculpt import frame,join,vane
from bovine_sculpt import oval,reverse_front
from lugia_sculpt import paddle
from murkrow_sculpt import bill
from skin_glb import COORDINATES
HEIGHT=.75
PALETTE={
 'russet':(.71,.43,.17),'crown_ridge':(.80,.51,.22),'cream':(.97,.90,.71),
 'cream_shadow':(.81,.70,.49),'coral':(.87,.40,.44),'coral_light':(.94,.47,.50),
 'wine_fold':(.76,.32,.36),'pink_beak':(.96,.67,.59),'pink_feet':(.86,.51,.46),
 'beak_shadow':(.83,.49,.46),'ivory':(.99,.97,.86),'ink':(.12,.08,.07),
 'tail':(.53,.32,.13),'claw':(.94,.90,.77),
}
PARTS=(
 'Torso / warm buff continuous breast and belly','Torso / fitted chestnut back mantle',
 'Head / chestnut cheek and forehead','Crown / four swept chestnut paper points','Crown / warm folded highlights',
 'Head / swept cheek and throat feather points','Eye left / angular ivory sclera','Eye left / forward square black pupil',
 'Head / sharp russet brow rims','Wing left / broad coral upper coverts','Wing left / ivory covert tips',
 'Wing left / four overlapping flight feathers','Wing left / flight feather crease shadows','Feet / planted rose ankles',
 'Feet / six splayed forward toes','Feet / ivory grounded toe claws','Feet / two backward gripping toes',
 'Eye right / angular ivory sclera','Eye right / forward square black pupil','Wing right / broad coral upper coverts',
 'Wing right / ivory covert tips','Wing right / four overlapping flight feathers','Wing right / flight feather crease shadows',
 'Beak / short rose upper wedge','Beak / lower hinged rose wedge','Beak / inset mouth seam',
 'Tail / three squared chestnut fan feathers','Tail / warm raised fold ridges',
)
def mark(name,color,outline,parent,th=.003):
 return fitted_triangle_contour(name,PALETTE[color],outline,parent,th,precision=6)

def flight_sections(root,tip,width):
 """Two closed pigment volumes meet at one exact moving feather section."""
 dy,dz=tip[1]-root[1],tip[2]-root[2];length=math.hypot(dy,dz);uy,uz=-dz/length,dy/length
 outline=[(-root[2]-t*dz-w*uz*width,root[1]+t*dy+w*uy*width,t)
  for t,w in ((0,-.35),(.24,-.92),(.74,-1.),(1.,-.10),(1.,.10),(.74,1.),(.24,.92),(0.,.35))]
 result=[];cut=.77
 for terminal,color in ((False,'coral_light'),(True,'ivory')):
  clipped=[]
  for a,b in zip(outline,outline[1:]+outline[:1]):
   inside=lambda q:q[2]>=cut if terminal else q[2]<=cut
   if inside(a):clipped.append(a[:2])
   if inside(a)!=inside(b):
    f=(cut-a[2])/(b[2]-a[2]);clipped.append(tuple(a[i]+f*(b[i]-a[i]) for i in range(2)))
  q=fitted_contour('flight section',PALETTE[color],clipped,lambda y,x:root[0],.010,0)
  for field in ('positions','normals'):
   q[field]=[v for x,y,z in zip(q[field][::3],q[field][1::3],q[field][2::3]) for v in (z,y,-x)]
  result.append(q)
 return result

def sculpture(*,with_transform=False):
 groups={n:[] for n in PARTS};colors={}
 def put(name,color,p):
  colors[name]=color;groups[name].append(p)
 body=loft(PARTS[0],PALETTE['cream'],[(.082,.034,.047,-.045),(.125,.104,.130,-.050),(.205,.157,.195,-.055),(.295,.155,.185,-.030),(.373,.117,.128,.018),(.422,.061,.066,.046)],14)
 head=loft(PARTS[2],PALETTE['russet'],[(.306,.012,.025,.113),(.351,.073,.086,.080),(.405,.112,.121,.065),(.470,.132,.143,.052),(.525,.120,.130,.034),(.566,.083,.099,.008),(.587,.025,.037,-.015)],14)
 put(PARTS[0],'cream',body);put(PARTS[2],'russet',head)
 rear=reverse_front(body)
 mantle=mark(PARTS[1],'russet',[(-.050,.412),(-.112,.372),(-.157,.293),(-.158,.205),(-.101,.130),(0.,.110),(.101,.130),(.158,.205),(.157,.293),(.112,.372),(.050,.412)],rear,.004)
 put(PARTS[1],'russet',reverse_front(mantle))
 for j,x in enumerate((-.086,-.027,.035,.093)):
  crest=paddle(PARTS[3],PALETTE['russet'],[(x*.65,.529,.025),(x,.585,-.030),(x*1.13,.650-.11*abs(x),-.130-.014*j)],[.028,.029,.0015],[.022,.019,.0015],segments=6,steps=1)
  put(PARTS[3],'russet',crest)
  # A fitted warm ridge follows the actual stored front of the swept feather.
  if j in (1,2):
   ridge=mark(PARTS[4],'crown_ridge',[(x-.007,.568),(x-.006,.590),(x+.006,.592),(x+.007,.569)],crest,.0015)
   put(PARTS[4],'crown_ridge',ridge)
 for side,label in ((-1,'left'),(1,'right')):
  for y,z,dy,dz in ((.474,-.020,.010,-.130),(.405,.015,-.025,-.085)):
   q=paddle(PARTS[5],PALETTE['russet'],[(side*.089,y,z+.045),(side*.120,y+.005,z+.010),(side*.125,y+dy,z+dz)],[.030,.033,.0015],[.022,.025,.0015],segments=6,steps=1)
   put(PARTS[5],'russet',q)
  put(PARTS[5],'russet',paddle(PARTS[5],PALETTE['russet'],[(side*.053,.388,.145),(side*.059,.350,.157),(side*.064,.327,.156)],[.026,.024,.0015],[.018,.015,.0015],segments=6,steps=1))
  put(PARTS[5],'russet',paddle(PARTS[5],PALETTE['russet'],[(side*.090,.408,.110),(side*.100,.382,.140),(side*.102,.352,.143)],[.020,.021,.0015],[.016,.014,.0015],segments=6,steps=1))
  angle=side*math.radians(30);local=frame(head,angle,True)
  uv=lambda ps:[(side*x,y) for x,y in ps]
  socket=mark('socket','ink',uv(((-.047,.493),(.003,.520),(.049,.521),(.047,.489),(.003,.481),(-.042,.482))),local,.003)
  eye=mark('eye','ivory',uv(((-.034,.499),(.006,.512),(.039,.513),(.036,.496),(.002,.490),(-.030,.489))),socket,.002)
  pupil=mark('pupil','ink',oval(-side*.012,.500,.008,.011,8),eye,.0015)
  put(f'Eye {label} / angular ivory sclera','ivory',frame(eye,angle))
  put(f'Eye {label} / forward square black pupil','ink',frame(join('gaze',[socket,pupil]),angle))
  brow=mark(PARTS[8],'russet',uv(((-.049,.501),(.003,.528),(.055,.528),(.052,.518),(.006,.516),(-.044,.493))),local,.006)
  put(PARTS[8],'russet',frame(brow,angle))
  wing=paddle(f'Wing {label} / broad coral upper coverts',PALETTE['coral'],[(side*.115,.350,-.015),(side*.205,.287,-.090),(side*.207,.174,-.218)],[.042,.051,.015],[.053,.064,.020],segments=8,steps=2)
  put(wing['part'],'coral',wing)
  for j in range(4):
   root=(side*(.214+.004*j),.292-.013*j,-.061-.020*j);tip=(side*(.230+.004*j),.130+.012*j,-.190-.035*j)
   q,tipmark=flight_sections(root,tip,.040);put(f'Wing {label} / four overlapping flight feathers','coral_light',q)
   shade=vane('shade',PALETTE['wine_fold'],(root[0]-side*.007,root[1],root[2]),(tip[0]-side*.007,tip[1]+.020,tip[2]+.012),.034,.009)
   put(f'Wing {label} / flight feather crease shadows','wine_fold',shade)
   # The white end uses the real vane outline, rather than a wider floating cap.
   put(f'Wing {label} / ivory covert tips','ivory',tipmark)
  x=side*.074
  put(PARTS[13],'pink_feet',tube('shank',PALETTE['pink_feet'],[(x,.147,.005),(x,.087,.005),(x,.034,.005)],[.018,.014,.018],4))
  for j,dx in enumerate((-.028,0.,.028)):
   end=(x+dx,.017,.118 if j==1 else .104)
   put(PARTS[14],'pink_feet',tube('front toe',PALETTE['pink_feet'],[(x+dx*.20,.030,.012),(x+dx*.65,.021,.066),end],[.012,.011,.008],4))
   put(PARTS[15],'claw',paddle('claw',PALETTE['claw'],[(end[0],.018,end[2]-.005),(end[0],.002,end[2]+.027)],[.008,.0015],[.008,.0015],segments=4,steps=1))
  put(PARTS[16],'pink_feet',tube('hallux',PALETTE['pink_feet'],[(x,.028,-.006),(x+side*.010,.019,-.044),(x+side*.013,.012,-.067)],[.012,.009,.005],4))
 put(PARTS[23],'pink_beak',bill(PARTS[23],PALETTE['pink_beak'],[(.173,.044,.451,.491),(.211,.052,.456,.493),(.260,.031,.443,.477),(.288,.002,.435,.448)]))
 put(PARTS[24],'beak_shadow',bill(PARTS[24],PALETTE['beak_shadow'],[(.174,.040,.405,.429),(.211,.044,.402,.425),(.255,.024,.423,.438),(.276,.002,.432,.436)]))
 put(PARTS[25],'ink',ellipsoid(PARTS[25],PALETTE['ink'],(0.,.447,.185),(.041,.027,.021),8,4))
 for j,x in enumerate((-.048,0.,.048)):
  tail=paddle(PARTS[26],PALETTE['tail'],[(x*.38,.212,-.174),(x,.191,-.298),(x*1.35,.174,-.421+abs(x)*.25)],[.024,.033,.017],[.011,.009,.004],segments=6,steps=1)
  put(PARTS[26],'tail',tail)
  if j==1:put(PARTS[27],'crown_ridge',mark('tail ridge','crown_ridge',[(x-.006,.201),(x-.005,.205),(x+.005,.205),(x+.006,.201)],tail,.0015))
 parts=[]
 for name,ps in groups.items():
  if not ps:raise ValueError('unbuilt Spearow anatomy: '+name)
  q=join(name,ps);q['color']=colors[name];parts.append(q)
 lo=min(v for q in parts for v in q['positions'][1::3]);hi=max(v for q in parts for v in q['positions'][1::3]);k=HEIGHT/(hi-lo)
 for q in parts:
  q['positions']=[round((v-(lo if i%3==1 else 0.))*k,7) for i,v in enumerate(q['positions'])]
  q['positions']=list(struct.unpack('<'+'f'*len(q['positions']),struct.pack('<'+'f'*len(q['positions']),*q['positions'])))
  q.update(crease_normals(q,40.));q['normals']=[round(v,7) or 0. for v in q['normals']]
 model=dict(name='spearow',version=1,coordinate_system=COORDINATES,primitives=parts)
 return (model,(lo,k)) if with_transform else model
