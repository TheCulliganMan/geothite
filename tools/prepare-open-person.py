"""Prepare a locally supplied Kenney rig for the external people asset path.

Run with Blender (the same authoring tool used by the existing model recipes):
  blender -b --python tools/prepare-open-person.py -- \
    --pack /absolute/ignored/kenney-protagonists --skin skaterMaleA \
    --out /absolute/ignored/people/skater-male.glb

Preserves the deform skeleton, skin weights, UVs and evaluated source motion.
Corrects differing FBX rest poses through the supplied targeting pose.
Never writes assets into the tracked catalog. This is offline authoring only.
"""
import argparse
import sys
from pathlib import Path

import bpy


def arguments():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--skin', required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    args.pack = args.pack.resolve()
    args.out = args.out.resolve()
    if args.skin not in ('skaterMaleA', 'skaterFemaleA', 'criminalMaleA',
                         'cyborgFemaleA', 'humanMaleA', 'humanFemaleA'):
        parser.error('choose an explicit supplied human skin')
    if args.out.suffix != '.glb' or 'content-packs' not in args.out.parts:
        parser.error('converted people must stay in ignored content-packs as GLB')
    for path in (args.pack / 'Model/characterMedium.fbx',
                 args.pack / 'Skins' / (args.skin + '.png'), args.pack / 'License.txt'):
        if not path.is_file():
            parser.error(f'missing supplied asset: {path}')
    return args


def import_clip(pack, armature, name):
    before_objects = set(bpy.data.objects)
    before_actions = set(bpy.data.actions)
    bpy.ops.import_scene.fbx(filepath=str(pack / 'Animations' / (name.lower() + '.fbx')))
    imported = set(bpy.data.objects) - before_objects
    imported_actions = set(bpy.data.actions) - before_actions
    matches = [a for a in imported_actions if a.name.split('|')[-1].split('.')[0] == name]
    if len(matches) != 1:
        raise ValueError(f'expected one authored {name} action, found {[a.name for a in matches]}')
    source_armatures = [o for o in imported if o.type == 'ARMATURE']
    if len(source_armatures) != 1 or set(source_armatures[0].data.bones.keys()) != set(armature.data.bones.keys()):
        raise ValueError(f'{name} skeleton does not match the model')
    source = source_armatures[0]
    targeting = [a for a in imported_actions if a.name.split('|')[-1].split('.')[0] == '0']
    if len(targeting) != 1:
        raise ValueError('missing supplied targeting pose for rest-pose correction')
    source.animation_data.action = targeting[0]
    source.animation_data.action_slot = targeting[0].slots[0]
    bpy.context.scene.frame_set(int(targeting[0].frame_range[0]))
    reference = {b.name: b.matrix.copy() for b in source.pose.bones}
    source_action = matches[0]
    source.animation_data.action = source_action
    source.animation_data.action_slot = source_action.slots[0]
    action = bpy.data.actions.new(name)
    armature.animation_data.action = action
    ordered = sorted(armature.pose.bones, key=lambda b: len(b.parent_recursive))
    first, last = map(int, source_action.frame_range)
    # FBX stores each clip's first pose as its rest skeleton. Retarget evaluated
    # global transforms through the supplied targeting pose onto the body's
    # bind matrices. Baking retains the source poses, weights and hierarchy.
    for frame in range(first, last + 1):
        bpy.context.scene.frame_set(frame)
        desired = {b.name: source.pose.bones[b.name].matrix @ reference[b.name].inverted() @ b.bone.matrix_local
                   for b in ordered}
        for bone in ordered:
            kwargs = {}
            if bone.parent:
                kwargs = dict(parent_matrix=desired[bone.parent.name],
                              parent_matrix_local=bone.parent.bone.matrix_local)
            bone.matrix_basis = bone.bone.convert_local_to_pose(
                desired[bone.name], bone.bone.matrix_local, invert=True, **kwargs)
            bone.rotation_mode = 'QUATERNION'
            for channel in ('location', 'rotation_quaternion', 'scale'):
                bone.keyframe_insert(channel, frame=frame, group=bone.name)
    armature.animation_data.action = None
    track = armature.animation_data.nla_tracks.new()
    track.name = name
    track.strips.new(name, first, action)
    track.mute = True
    for obj in imported:
        bpy.data.objects.remove(obj, do_unlink=True)
    for unused in imported_actions - {action}:
        if unused.users == 0:
            bpy.data.actions.remove(unused)


def main():
    args = arguments()
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete(use_global=False)
    bpy.ops.import_scene.fbx(filepath=str(args.pack / 'Model/characterMedium.fbx'))
    armatures = [o for o in bpy.context.scene.objects if o.type == 'ARMATURE']
    meshes = [o for o in bpy.context.scene.objects if o.type == 'MESH']
    if len(armatures) != 1 or not meshes:
        raise ValueError('expected one weighted character armature and a visible body')
    armature = armatures[0]
    deform = [b for b in armature.data.bones if b.use_deform]
    if not deform or not all(o.vertex_groups and any(m.type == 'ARMATURE' and m.object == armature for m in o.modifiers) for o in meshes):
        raise ValueError('source is not a weighted rig')
    for bone in armature.pose.bones:
        if bone.constraints:
            raise ValueError('unexpected live rig constraints in exported FBX')
    armature.animation_data_create()
    armature.animation_data.action = None
    for track in list(armature.animation_data.nla_tracks):
        armature.animation_data.nla_tracks.remove(track)
    for name in ('Idle', 'Run', 'Jump'):
        import_clip(args.pack, armature, name)
    # Bind evaluation must remain separate from the authored actions. All three
    # NLA clips export independently; none is selected as a gameplay default.
    for track in armature.animation_data.nla_tracks:
        track.mute = True
    image = bpy.data.images.load(str(args.pack / 'Skins' / (args.skin + '.png')))
    image.pack()
    material = bpy.data.materials.new('Kenney ' + args.skin)
    material.use_nodes = True
    shader = material.node_tree.nodes.get('Principled BSDF')
    shader.inputs['Roughness'].default_value = 0.88
    texture = material.node_tree.nodes.new('ShaderNodeTexImage')
    texture.image = image
    material.node_tree.links.new(texture.outputs['Color'], shader.inputs['Base Color'])
    for obj in meshes:
        obj.data.materials.clear()
        obj.data.materials.append(material)
    bpy.context.view_layer.update()
    points = [o.matrix_world @ v.co for o in meshes for v in o.data.vertices]
    height = max(p.z for p in points) - min(p.z for p in points)
    if height <= 0:
        raise ValueError('invalid character extent')
    print(f'SOURCE: {len(deform)} deform bones, {sum(len(o.data.vertices) for o in meshes)} vertices, height {height:.6f}', flush=True)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    for track in armature.animation_data.nla_tracks:
        track.mute = False
    bpy.ops.export_scene.gltf(filepath=str(args.out), export_format='GLB',
        export_animations=True, export_animation_mode='NLA_TRACKS',
        export_skins=True, export_def_bones=True, export_all_influences=True,
        export_force_sampling=True)
    print(f'EXTERNAL GLB: {args.out}', flush=True)


if __name__ == '__main__':
    main()
