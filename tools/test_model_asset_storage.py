#!/usr/bin/env python3
"""Raw canonical model validation independent of Blender and game packs."""
import ast
import json
from pathlib import Path
import tempfile
import unittest
from model_asset_storage import (MAX_MODEL_BYTES, read_model_bytes, read_model_json,
    validate_storage, sha, store_model_bytes, stored_model_size, validate_model)

class ModelAssetStorageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.path = self.root/'fixture.mesh.json'
        self.raw = json.dumps({'name':'é🧩','values':[-0.0,1.234567890123456,17]*20},ensure_ascii=False,separators=(',',':')).encode()+b'\n'
        self.path.write_bytes(self.raw)
    def tearDown(self):
        self.temp.cleanup()
    def test_exact_authored_bytes_unicode_signed_zero_and_precision(self):
        identity=store_model_bytes(self.path,self.raw)
        self.assertEqual(self.path.read_bytes(),self.raw)
        self.assertEqual(identity,{'bytes':len(self.raw),'sha256':sha(self.raw)})
        self.assertEqual(read_model_json(self.path),json.loads(self.raw))
        self.assertEqual(validate_storage(self.root),[self.path])
        self.assertEqual(validate_model(self.path),(len(self.raw),len(self.raw)))
        self.assertEqual(stored_model_size(self.path),len(self.raw))
        self.assertEqual(list(self.root.iterdir()),[self.path])
    def test_invalid_inputs_are_rejected_before_writing(self):
        for raw in [b'',b'\xff',b'[]',b'{"storage":"geothite-model-gzip-v1"}',b'{',b' '*(MAX_MODEL_BYTES+1)]:
            with self.subTest(raw=raw[:30]),self.assertRaises((ValueError,UnicodeDecodeError)):
                store_model_bytes(self.path,raw)
            self.assertEqual(self.path.read_bytes(),self.raw)
    def test_encoded_storage_is_never_silently_accepted(self):
        for suffix in ['.gz','.include.rs','.chunks','.b64']:
            stale=Path(str(self.path)+suffix);stale.write_bytes(b'old wrapper')
            with self.subTest(suffix=suffix),self.assertRaises(ValueError):validate_storage(self.root)
            stale.unlink()
        self.path.write_text('{"storage":"unknown"}')
        with self.assertRaises(ValueError):read_model_bytes(self.path)
    def test_symlinks_cannot_overwrite_another_asset(self):
        target=self.root/'other.json';target.write_bytes(self.raw)
        self.path.unlink();self.path.symlink_to(target)
        with self.assertRaises(ValueError):store_model_bytes(self.path,b'{"other":1}')
        with self.assertRaises(ValueError):validate_storage(self.root)
        self.assertEqual(target.read_bytes(),self.raw)
    def test_foliage_authoring_reads_canonical_geometry(self):
        generator=Path(__file__).with_name('build-johto-foliage-lod.py')
        tree=ast.parse(generator.read_text())
        reader=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='read_foliage_sources')
        scope={'read_model_json':read_model_json}
        exec(compile(ast.Module(body=[reader],type_ignores=[]),str(generator),'exec'),scope)
        expected={}
        for name,directory in [('tree','new_bark'),('grass','johto')]:
            path=self.root/f'crates/crystal-voxel-view/models/{directory}/{name}.mesh.json'
            path.parent.mkdir(parents=True,exist_ok=True)
            geometry={'name':name,'bounds':{'min':[0,0,0],'max':[1,2,3]},'primitives':[{'indices':[0,1,2]}]}
            store_model_bytes(path,json.dumps(geometry).encode())
            self.assertNotIn('storage',json.loads(path.read_bytes()))
            expected[name]=geometry
        self.assertEqual(scope['read_foliage_sources'](self.root),expected)

if __name__=='__main__':unittest.main()
