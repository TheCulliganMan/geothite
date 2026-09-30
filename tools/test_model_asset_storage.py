#!/usr/bin/env python3
"""Storage validation independent of Blender, network and game packs."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from model_asset_storage import pack_model, compress_model, read_model_bytes, read_model_json, validate_storage, wrapper_path, read_stored_bytes, MAX_MODEL_BYTES

class ModelAssetStorageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.path = self.root / 'fixture.mesh.json'
        self.raw = json.dumps({'name':'é🧩', 'values':[-0.0,1.234567890123456,17]*20},ensure_ascii=False,separators=(',',':')).encode()+b'\n'
        self.path.write_bytes(self.raw)
        pack_model(self.path,32)
    def tearDown(self):
        self.temp.cleanup()
    def rewrite(self, transform):
        data=json.loads(self.path.read_bytes());transform(data)
        self.path.write_text(json.dumps(data))
    def test_exact_bytes_unicode_signed_zero_and_repeat_pack(self):
        self.assertEqual(read_model_bytes(self.path),self.raw)
        self.assertEqual(read_model_json(self.path),json.loads(self.raw))
        before={p.relative_to(self.root):p.read_bytes() for p in self.root.rglob('*') if p.is_file()}
        pack_model(self.path,32)
        self.assertEqual(before,{p.relative_to(self.root):p.read_bytes() for p in self.root.rglob('*') if p.is_file()})
        self.assertEqual(validate_storage(self.root),[self.path])
    def test_missing_and_corrupt_chunk_rejected(self):
        first=self.root/json.loads(self.path.read_bytes())['chunks'][0]['path']
        first.write_bytes(b'corruption')
        with self.assertRaises(ValueError):read_model_bytes(self.path)
        first.unlink()
        with self.assertRaises(FileNotFoundError):read_model_bytes(self.path)
    def test_reordering_duplication_traversal_and_absolute_paths_rejected(self):
        original=self.path.read_bytes()
        changes=[lambda d:d['chunks'].reverse(),lambda d:d['chunks'].__setitem__(1,copy.deepcopy(d['chunks'][0])),lambda d:d['chunks'][0].__setitem__('path','../outside.txt'),lambda d:d['chunks'][0].__setitem__('path','/tmp/outside.txt')]
        for change in changes:
            self.path.write_bytes(original);self.rewrite(change)
            with self.assertRaises(ValueError):read_model_bytes(self.path)
    def test_corrupt_digest_size_and_rust_wrapper_rejected(self):
        original=self.path.read_bytes()
        for change in [lambda d:d.__setitem__('sha256','0'*64),lambda d:d.__setitem__('bytes',d['bytes']+1),lambda d:d['chunks'][0].__setitem__('bytes',True)]:
            self.path.write_bytes(original);self.rewrite(change)
            with self.assertRaises(ValueError):read_model_bytes(self.path)
        self.path.write_bytes(original)
        wrapper_path(self.path).write_text('"wrong payload"\n')
        with self.assertRaises(ValueError):read_model_bytes(self.path)
    def test_orphan_chunk_and_foreign_asset_dependency_rejected(self):
        orphan=Path(str(self.path)+'.chunks')/'999.txt';orphan.write_text('unreferenced')
        with self.assertRaises(ValueError):validate_storage(self.root)
        orphan.unlink()
        other=self.root/'different.mesh.json';other.write_bytes(self.path.read_bytes())
        with self.assertRaises(ValueError):read_model_bytes(other)
    def test_plain_json_stays_a_valid_asset(self):
        p=self.root/'small.json';p.write_bytes(b'{"primitives":[]}\n')
        self.assertFalse(pack_model(p))
        self.assertEqual(read_model_bytes(p),b'{"primitives":[]}\n')

    def test_gzip_storage_is_exact_deterministic_and_has_no_clock_metadata(self):
        compress_model(self.path,128)
        self.assertEqual(read_model_bytes(self.path),self.raw)
        before={p.relative_to(self.root):p.read_bytes() for p in self.root.rglob('*') if p.is_file()}
        compress_model(self.path,128)
        self.assertEqual(before,{p.relative_to(self.root):p.read_bytes() for p in self.root.rglob('*') if p.is_file()})
        import base64
        stored=json.loads(read_stored_bytes(self.path))
        zipped=base64.b64decode(stored['data'])
        self.assertEqual(zipped[4:8],b'\0'*4)
        self.assertEqual(zipped[9],255)
    def test_inline_compression_removes_superseded_literal_chunks(self):
        compress_model(self.path)
        self.assertFalse(wrapper_path(self.path).exists())
        self.assertFalse(Path(str(self.path)+'.chunks').exists())
        self.assertEqual(read_model_bytes(self.path),self.raw)
        self.assertEqual(validate_storage(self.root),[])
    def test_gzip_rejects_wrong_hash_size_encoding_and_trailing_data(self):
        compress_model(self.path)
        original=self.path.read_bytes()
        changes=[lambda d:d.__setitem__('sha256','0'*64),lambda d:d.__setitem__('bytes',1),lambda d:d.__setitem__('bytes',MAX_MODEL_BYTES+1),lambda d:d.__setitem__('data','invalid base64')]
        for change in changes:
            self.path.write_bytes(original);self.rewrite(change)
            with self.assertRaises((ValueError, OSError)):read_model_bytes(self.path)
        import base64
        self.path.write_bytes(original)
        self.rewrite(lambda d:d.__setitem__('data',base64.b64encode(base64.b64decode(d['data'])+b'trailing').decode()))
        with self.assertRaises((ValueError, OSError)):read_model_bytes(self.path)

if __name__ == '__main__':unittest.main()
