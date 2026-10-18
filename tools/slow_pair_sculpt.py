#!/usr/bin/env python3
"""Local fitted surfaces and selective round sampling for the two shell-wearing bipeds."""
from sculpt_geometry import ellipsoid as sphere,fitted_adaptive_contour
from lugia_sculpt import paddle as swept
from cubone_sculpt import skin_shading
from bovine_sculpt import smooth
from goldfish_sculpt import surface

def ellipsoid(name,color,center,radii,segments=28,rings=18):
 cap=(32,20) if name.startswith(('Head /','Muzzle /','Jaw /','Crown brim /')) else (28,18) if name.startswith('Thigh /') else (24,16)
 return sphere(name,color,center,radii,min(segments,cap[0]),min(rings,cap[1]))
def paddle(name,color,points,widths,depths,**kwargs):
 segments=kwargs.pop('segments',20 if name.startswith('Foot claw /') else 24)
 return swept(name,color,points,widths,depths,segments=segments,**kwargs)
def forelimb(name,color,points,widths,depths,segments=20):
 # Smooth centerline and both radius profiles together, as on the turtles.
 # One closed surface carries the elbow, wrist and complete pink palm.
 path,_=smooth(points,[1.]*len(points),6)
 def profile(values):return [p[0] for p in smooth([(v,0.,0.) for v in values],[1.]*len(values),6)[0]]
 return swept(name,color,path,profile(widths),profile(depths),segments=segments,steps=1)

def paint(name,color,xy,parent,th=.006,sub=3):
 return fitted_adaptive_contour(name,color,xy,None,th,surface_at=surface(parent),initial_subdivisions=1)
def fitted_shading(parts):
 def find(prefix,side=None):return next(p for p in parts if p['part'].startswith(prefix) and (side is None or p['part'].endswith(' '+side)))
 has_head=any(p['part'].startswith('Head /') for p in parts)
 for i,p in enumerate(parts):
  n=p['part'];side=n.split()[-1];parent=None
  for child,owner in (('Belly band /','Body /'),('Eye /','Head /' if has_head else 'Body /'),('Pupil /','Eye /'),('Ear inset /','Ear /'),('Ear curl /','Ear inset /'),('Nostril /','Muzzle /'),('Mouth seam /','Muzzle /'),('Shell socket /','Shell body /'),('Shell eye /','Shell socket /'),('Shell pupil /','Shell eye /'),('Jewel rim /','Crown body /'),('Jewel /','Jewel rim /'),('Jewel glint /','Jewel /')):
   if n.startswith(child):parent=find(owner,side if owner in ('Eye /','Ear /','Ear inset /','Shell socket /','Shell eye /') else None)
  if parent:parts[i]=skin_shading(p,parent)
 return parts
