"""Body shading preserves reviewed anatomy, support weights and animation.

Pinned contracts expand indexed corners so normal-only welding cannot hide
changes to triangle order, fitted artwork, supports, skin weights or clip keys.
"""
import hashlib
import json
import math
import os
from pathlib import Path
import struct
import unittest

from chikorita_sculpt import cross, dot, sub, unit
from skin_glb_test_support import raw_accessor
from test_gengar_glb import Model

ROOT=Path(__file__).resolve().parent.parent
ASSETS={
    'chikorita':ROOT/'crates/crystal-voxel-view/models/actor_props/battle_chikorita.glb',
    'bayleef':ROOT/'crates/crystal-voxel-view/models/battle_species/bayleef.glb',
    'meganium':ROOT/'crates/crystal-voxel-view/models/battle_species/meganium.glb',
}
# Reviewed surfaces after the grass-family reference-volume revision.
CONTRACTS={'chikorita': '31fc906e84e58444aaf7052d073d5af01aff6dfdc89c9b8ab76060a0b2caead3', 'bayleef': 'c484fe980741c48560124bd4a50c73035e226f7672303cdf7e0c7b9c8258ce5e', 'meganium': 'bc961b2820dde5b5d7ce6e3bacc30e234588227b51aad0b58da0be3585877b74'}


def preserved_contract(model):
    h=hashlib.sha256();d=model.doc
    def text(value):h.update(json.dumps(value,sort_keys=True,separators=(',',':')).encode())
    text(d['nodes'])
    for skin in d['skins']:
        text((skin['joints'],skin['skeleton']))
        h.update(raw_accessor(d,model.binary,skin['inverseBindMatrices']))
    for animation in d['animations']:
        text(animation['name'])
        for channel in animation['channels']:
            sampler=animation['samplers'][channel['sampler']]
            text((channel['target'],sampler['interpolation']))
            for key in ('input','output'):h.update(raw_accessor(d,model.binary,sampler[key]))
    for p,name in enumerate(model.names):
        text(name)
        text(d['materials'][model.parts[p]['material']]['pbrMetallicRoughness'])
        for i in model.indices[p]:
            h.update(struct.pack('<3f',*model.vertices[p][i]))
            h.update(bytes(model.ids[p][i]))
            h.update(bytes(round(w*255) for w in model.weights[p][i]))
            if p:
                # Older Meganium containers differ only in sub-1e-14 normal
                # residuals across libm platforms; preserve the visible field.
                h.update(struct.pack('<3f',*(0. if abs(v)<1e-12 else v for v in model.normals[p][i])))
    return h.hexdigest()


class LeafSaurShadingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        directory=os.environ.get('LEAF_SAUR_SKIN_MODELS')
        cls.models={s:Model((Path(directory)/(s+'.glb') if directory else p).read_bytes()) for s,p in ASSETS.items()}

    def test_reviewed_triangles_artwork_supports_weights_and_keys_are_preserved(self):
        for species,m in self.models.items():
            with self.subTest(species=species):self.assertEqual(preserved_contract(m),CONTRACTS[species])

    def test_curved_body_normals_stay_outward_and_floor_caps_stay_flat(self):
        for species,m in self.models.items():
            with self.subTest(species=species):
                points=m.vertices[0];normals=m.normals[0];floor=0;smoothed=0
                for n in normals:self.assertAlmostEqual(math.sqrt(dot(n,n)),1.,places=6)
                for start in range(0,len(m.indices[0]),3):
                    ids=m.indices[0][start:start+3]
                    a,b,c=(points[i] for i in ids);face=unit(cross(sub(b,a),sub(c,a)))
                    for i in ids:
                        cosine=dot(face,normals[i])
                        self.assertGreater(cosine,math.cos(math.radians(35.))-1e-6)
                        smoothed+=cosine<.995
                        if all(points[j][1]==0. for j in ids):
                            self.assertGreater(cosine,.999999);floor+=1
                self.assertGreater(smoothed,1000)
                self.assertEqual(floor,720)
                # This is a vertex reduction, not a triangle/shape reduction.
                self.assertLess(len(points),2100)


if __name__=='__main__':unittest.main()
