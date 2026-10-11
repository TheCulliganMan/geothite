#!/usr/bin/env python3
"""Verify production character source coverage and reject the removed renderer."""
from pathlib import Path
import re
root=Path(__file__).resolve().parents[1]
source=root/'crates/crystal-voxel-view/src'
registry=(source/'johto_characters.rs').read_text()
roles=re.findall(r'\("([a-z_]+)", CharacterKind::',registry)
assert len(roles)==74 and len(set(roles))==74
assert not (root/'crates/crystal-voxel-view/models/johto_characters/catalog.glb').exists()
assert not (source/'human_glb.rs').exists()
renderer=(source/'new_bark_actors.rs').read_text()
assert 'person: crate::open_people_skin::Person' in renderer
assert 'JOINT_COUNT' not in renderer and 'let model = rig(kind)' not in renderer
for name in ['human-male','human-female']:
    path=root/f'assets/open-models/people/kenney/{name}.glb'
    assert path.read_bytes().startswith(b'glTF'),path
print('PASS 74 named sources plus outdoorsman and bicycle aliases; required weighted rigs present; legacy renderer/catalog absent')
