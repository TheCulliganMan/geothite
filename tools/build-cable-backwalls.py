#!/usr/bin/env python3
"""Original closed-solid Cable Club link console, source-pixel dimensions.

The named editable parts and cached runtime mesh share the same geometry.
No source images, imported meshes or copied game art are used.
"""
from pathlib import Path
import importlib.util,sys
spec=importlib.util.spec_from_file_location('cable_common',Path(__file__).with_name('build-cable-club.py'))
common=importlib.util.module_from_spec(spec);spec.loader.exec_module(common)

def make_console():
 a=common.Asset('link_console')
 # Source's upper-left 8x8 negative space and the entire mobile-warp row
 # remain empty. The slender tether occupies only the northeast source cell.
 a.prism('Raised rear cable return',(13.5,15.7,0,13.0),7.2,8.1,'soft_ivory',.26)
 a.prism('Return inset cable channel',(14.1,15.1,0.05,13.5),7.9,8.35,'blue_inset',.18)
 a.prism('Return elbow housing',(11.4,15.8,10.2,16.3),5.7,8.4,'powder_blue',.8)
 # A continuous lower counter body follows the original three-cell-deep
 # terminal drawing. Every separate part has a closed bottom face.
 a.prism('Continuous console toe plinth',(0,16,8,32),0,1.4,'slate_plinth',.9)
 a.prism('Blue cabinet pedestal',(.45,15.55,8.45,31.55),1.15,8.6,'powder_blue',1.4)
 a.prism('Ivory chamfered counter rim',(0,16,8,32),8.2,9.2,'soft_ivory',1.7)
 a.prism('Pale inset countertop',(.6,15.4,8.6,31.4),9.2,9.5,'edge_highlight',1.5)
 # A softened upright link receiver and deep recessed screen recreate the
 # compact monitor silhouette without treating every native cell as a cube.
 a.prism('Ivory rounded receiver housing',(2.0,14.2,11.7,24.2),9.45,19.4,'soft_ivory',2.0)
 a.prism('Powder blue raised receiver crown',(2.5,13.7,12.2,23.7),19.35,20.0,'powder_blue',1.7)
 a.prism('Inset dark display bezel',(3.2,13.0,24.0,24.6),11.5,18.4,'slate_plinth',.55)
 a.prism('Recessed jade display',(4.0,12.2,24.59,24.77),12.2,17.7,'screen_ink',.28)
 a.prism('Single ivory ready indicator',(8.1,9.5,24.76,24.92),14.7,16.1,'signal_ivory',.18)
 a.prism('Upper metallic receiver lip',(3.3,12.9,24.1,24.8),18.3,18.7,'brushed_metal',.18)
 for side,x0,x1 in [('West',1.82,2.05),('East',14.15,14.38)]:
  a.prism(side+' receiver recessed side',(x0,x1,14.5,21.7),11.1,17.9,'blue_inset',.06)
  for z in [15.0,17.0,19.0]:
   a.prism(side+' physical vent '+str(z),(x0-.02,x1+.02,z,z+.42),12.0,15.8,'slate_plinth',.035)
 # Tactile keys and the counter front align with the existing reception kit.
 a.prism('Control key deck',(4.0,12.0,26.0,30.0),9.45,9.8,'blue_inset',.55)
 for k in range(3):
  a.prism('Physical ivory link key '+str(k),(4.6+k*2.35,6.25+k*2.35,26.6,28.0),9.75,10.2,'signal_ivory',.27)
 a.prism('Brass transfer key',(6.7,9.3,28.6,29.6),9.75,10.15,'brass',.28)
 a.prism('South recessed access panel',(2.1,13.9,31.5,31.8),2.6,6.8,'blue_inset',.14)
 a.prism('South ivory lower bead',(1.5,14.5,31.78,31.94),2.0,2.4,'edge_highlight',.06)
 a.prism('South brass access latch',(7.2,8.8,31.77,32),5.8,6.25,'brass',.04)
 return a

def build_blender(asset,out,skip_preview):
 import bpy
 from mathutils import Vector
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
 scene=bpy.context.scene;mats={}
 for name,color in common.PALETTE.items():
  m=bpy.data.materials.new(name);m.diffuse_color=color;m.use_nodes=True;m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=color;m.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value=.6;mats[name]=m
 col=bpy.data.collections.new('ASSET | link_console');scene.collection.children.link(col)
 for label,mat,vs,faces in asset.parts:
  me=bpy.data.meshes.new(label);me.from_pydata([(x/16,-z/16,y/16)for x,y,z in vs],[],faces);me.update();me.materials.append(mats[mat]);ob=bpy.data.objects.new(label,me);col.objects.link(ob)
 scene['provenance']='Original authored geometry and colors; no source images, ROM, textures or imported meshes'
 scene['rebuild']='tools/build-cable-backwalls.py';scene['source_opening']='Upper-left 8x8 source pixels remain empty; no geometry extends into the mobile warp row'
 bpy.ops.mesh.primitive_plane_add(size=200,location=(.5,-1,-.015));bpy.context.object.name='PREVIEW | Studio floor'
 m=bpy.data.materials.new('Preview neutral slate');m.diffuse_color=(.13,.17,.19,1);bpy.context.object.data.materials.append(m)
 scene.world.color=(.17,.19,.22)
 for name,loc,power,size in [('Key',(-3,-6,8),1100,6),('Fill',(5,4,6),800,5)]:
  bpy.ops.object.light_add(type='AREA',location=loc);o=bpy.context.object;o.name='PREVIEW | '+name;o.data.energy=power;o.data.size=size;o.rotation_euler=(Vector((.5,-1,.6))-o.location).to_track_quat('-Z','Y').to_euler()
 bpy.ops.object.camera_add(location=(4.2,-6,4.1));camera=bpy.context.object;camera.name='PREVIEW | Camera';camera.data.type='ORTHO';camera.data.ortho_scale=3.8;scene.camera=camera;camera.rotation_euler=(Vector((.5,-1,.7))-camera.location).to_track_quat('-Z','Y').to_euler()
 scene.render.engine='CYCLES';scene.cycles.samples=24;scene.cycles.use_denoising=False;scene.render.resolution_x=1000;scene.render.resolution_y=1000;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.view_settings.view_transform='AgX'
 bpy.ops.wm.save_as_mainfile(filepath=str(out/'cable-backwalls.blend'),compress=False)
 if not skip_preview:
  scene.render.filepath=str(out/'link-console-front.png');bpy.ops.render.render(write_still=True)
  camera.location=(-4,3,4);camera.rotation_euler=(Vector((.5,-1,.7))-camera.location).to_track_quat('-Z','Y').to_euler();scene.render.filepath=str(out/'link-console-rear.png');bpy.ops.render.render(write_still=True)

def main():
 args=sys.argv[sys.argv.index('--')+1:] if '--'in sys.argv else sys.argv[1:]
 out=Path(next((a for a in args if not a.startswith('--')),'target/cable-backwalls'));out.mkdir(parents=True,exist_ok=True)
 a=make_console();common.save_runtime(a,out);print(a.name,common.document(a)['triangle_count'],'triangles',common.document(a)['bounds'])
 if '--runtime-only'not in args:build_blender(a,out,'--skip-preview'in args)
if __name__=='__main__':main()
