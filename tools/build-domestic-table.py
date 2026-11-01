"""Author the shared four-leg timber table without regenerating other furnishings.

blender -b --python tools/build-domestic-table.py -- target/domestic-table
The editable Blender source and raw mesh contain no game content.
"""
from pathlib import Path

kit = Path(__file__).with_name('build-johto-interiors.py').read_text()
exec(compile(kit.split("\nstool();stool('arcade_stool'", 1)[0],
             'domestic_furnishing_vocabulary', 'exec'))
exporter = 'def export(name,col):' + kit.split('def export(name,col):', 1)[1].split(
    '\nbpy.context.view_layer.update()', 1)[0]
exec(compile(exporter, 'domestic_furnishing_exporter', 'exec'))
table('dining_table')
for obj in ASSETS['dining_table'].objects:
    if obj.name.startswith('Individual tabletop plank'):
        obj.data.materials.clear()
        obj.data.materials.append(MATS['timber_gold'])
bpy.context.view_layer.update()
export('dining_table', ASSETS['dining_table'])
scene['Asset source'] = 'Original four-leg timber table; named editable objects; no game content.'
bpy.ops.wm.save_as_mainfile(filepath=str(P / 'domestic-table.blend'), compress=False)
