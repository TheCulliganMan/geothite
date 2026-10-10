#!/usr/bin/env python3
"""Render canonical neutral JSON/GLB models for art review, never game screenshots.

Requires NumPy and Pillow. Output must remain in ignored target/. Uses actual
stored geometry, linear materials, normals and an orthographic depth buffer.
GLB node transforms are applied; animations and the game camera are not sampled.
"""
import argparse,json,math,struct
from pathlib import Path
import numpy as np
from PIL import Image,ImageDraw
from animated_glb import parse_glb


def read_models(path):
    if path.suffix!='.glb':return [(path.stem.removesuffix('.mesh'),json.loads(path.read_bytes())['primitives'])]
    d,b=parse_glb(path.read_bytes())
    def values(index):
        a=d['accessors'][index];v=d['bufferViews'][a['bufferView']]
        code={5126:'f',5125:'I',5123:'H'}[a['componentType']];width={'VEC3':3,'SCALAR':1}[a['type']]
        return list(struct.unpack_from('<'+code*(width*a['count']),b,v.get('byteOffset',0)+a.get('byteOffset',0)))
    def node_matrix(n):
        if 'matrix' in n:return np.array(n['matrix']).reshape(4,4).T
        x,y,z,w=n.get('rotation',[0,0,0,1]);s=np.array(n.get('scale',[1,1,1]))
        m=np.eye(4);m[:3,:3]=np.array([[1-2*y*y-2*z*z,2*x*y-2*z*w,2*x*z+2*y*w],[2*x*y+2*z*w,1-2*x*x-2*z*z,2*y*z-2*x*w],[2*x*z-2*y*w,2*y*z+2*x*w,1-2*x*x-2*y*y]])@np.diag(s)
        m[:3,3]=n.get('translation',[0,0,0]);return m
    result=[]
    for scene in d['scenes']:
        parts=[]
        def visit(i,parent):
            n=d['nodes'][i];matrix=parent@node_matrix(n)
            if 'mesh' in n:
                for p in d['meshes'][n['mesh']]['primitives']:
                    pos=np.array(values(p['attributes']['POSITION'])).reshape(-1,3)
                    nor=np.array(values(p['attributes']['NORMAL'])).reshape(-1,3)
                    pos=pos@matrix[:3,:3].T+matrix[:3,3]
                    nor=nor@np.linalg.inv(matrix[:3,:3]);nor/=np.linalg.norm(nor,axis=1)[:,None]
                    parts.append({'positions':pos.ravel().tolist(),'normals':nor.ravel().tolist(),'indices':values(p['indices']),'base_color':d['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']})
            for child in n.get('children',[]):visit(child,matrix)
        for root in scene['nodes']:visit(root,np.eye(4))
        result.append((scene.get('name',path.stem),parts))
    return result


def render(parts,size=240,yaw=.65,pitch=.12,fit=None):
    right=np.array([math.cos(yaw),0,-math.sin(yaw)])
    look=np.array([math.sin(yaw)*math.cos(pitch),math.sin(pitch),math.cos(yaw)*math.cos(pitch)])
    basis=np.array([right,-np.cross(look,right),look]);light=np.array([-.4,.8,.6]);light/=np.linalg.norm(light)
    points=np.concatenate([np.array(p['positions']).reshape(-1,3) for p in parts])
    lo,hi=(points.min(0),points.max(0)) if fit is None else fit
    center=(lo+hi)/2;extent=(hi-lo);scale=size*.72/max(extent[1],math.hypot(extent[0],extent[2]))
    rgb=np.full((size,size,3),[238,233,224],dtype=np.uint8);depth=np.full((size,size),-np.inf)
    for part in parts:
        points=np.array(part['positions']).reshape(-1,3);proj=(points-center)@basis.T
        proj[:,:2]=proj[:,:2]*scale+size/2
        norms=np.array(part['normals']).reshape(-1,3);base=np.array(part['base_color'][:3])
        for ids in np.array(part['indices']).reshape(-1,3):
            tri=proj[ids];normal=norms[ids].mean(0)
            if normal@look<=0:continue
            low=np.maximum(np.floor(tri[:,:2].min(0)).astype(int),[0,0]);high=np.minimum(np.ceil(tri[:,:2].max(0)).astype(int),[size-1,size-1])
            if np.any(low>high):continue
            xx,yy=np.meshgrid(np.arange(low[0],high[0]+1)+.5,np.arange(low[1],high[1]+1)+.5)
            a,b,c=tri;den=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
            if abs(den)<1e-9:continue
            u=((b[1]-c[1])*(xx-c[0])+(c[0]-b[0])*(yy-c[1]))/den
            v=((c[1]-a[1])*(xx-c[0])+(a[0]-c[0])*(yy-c[1]))/den
            w=1-u-v;z=u*a[2]+v*b[2]+w*c[2];sl=np.s_[low[1]:high[1]+1,low[0]:high[0]+1]
            mask=(u>=0)&(v>=0)&(w>=0)&(z>depth[sl]);depth[sl][mask]=z[mask]
            # Interpolate the authored corner normals as the GPU does. A
            # triangle-average light made smooth meshes look artificially flat.
            corner_normals=norms[ids]
            field=u[...,None]*corner_normals[0]+v[...,None]*corner_normals[1]+w[...,None]*corner_normals[2]
            field/=np.maximum(np.linalg.norm(field,axis=2,keepdims=True),1e-12)
            diffuse=np.maximum(0,field@light)
            color=np.clip(base*(.38+.62*diffuse[...,None]),0,1)
            srgb=np.where(color<=.0031308,12.92*color,1.055*color**(1/2.4)-.055)
            rgb[sl][mask]=(srgb[mask]*255).astype(np.uint8)
    return Image.fromarray(rgb)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('paths',type=Path,nargs='+');p.add_argument('--out',type=Path,required=True);p.add_argument('--size',type=int,default=240);p.add_argument('--yaw',type=float,default=.65);a=p.parse_args()
    if not 80<=a.size<=1024: p.error('size must be 80–1024')
    if 'target' not in a.out.parts:p.error('review output belongs under target/')
    files=[]
    for path in a.paths:files.extend(sorted(path.rglob('*.mesh.json'))+sorted(path.rglob('*.glb')) if path.is_dir() else [path])
    models=[m for path in files for m in read_models(path)];a.out.mkdir(parents=True,exist_ok=True)
    for first in range(0,len(models),24):
        batch=models[first:first+24];columns=min(6,len(batch));sheet=Image.new('RGB',(a.size*columns,(a.size+24)*math.ceil(len(batch)/columns)),(238,233,224))
        for i,(name,parts) in enumerate(batch):
            x=i%columns*a.size;y=i//columns*(a.size+24);sheet.paste(render(parts,a.size,a.yaw),(x,y));ImageDraw.Draw(sheet).text((x+8,y+a.size+3),name,fill=(30,30,30))
        dest=a.out/f'review-{first//24:02d}.png';sheet.save(dest);print(dest,flush=True)

if __name__=='__main__':main()
