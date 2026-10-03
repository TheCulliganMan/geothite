#!/usr/bin/env python3
"""Regression tests for strict preflight and byte-preserving legacy conversion."""
import base64,gzip,hashlib,io,json,tempfile,unittest
from pathlib import Path
from migrate_asset_storage import migrate
from johto_art_sources import REQUIRED_SOURCES
from model_asset_storage import read_model_bytes

def sha(b):return hashlib.sha256(b).hexdigest()

class MigrationTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.root=Path(self.temp.name)
        self.source=self.root/'art/johto/source';(self.source/'chunks').mkdir(parents=True)
        records=[]
        for name in sorted(REQUIRED_SOURCES):
            raw=b'BLENDER-v300'+name.encode();payload=gzip.compress(raw,mtime=0)
            part={'file':f'chunks/{name}.000.b64','bytes':len(payload),'sha256':sha(payload),'encoding':'base64'}
            (self.source/part['file']).write_bytes(base64.b64encode(payload))
            records.append({'file':name,'encoding':'gzip-blend','bytes':len(payload),'sha256':sha(payload),'uncompressed_bytes':len(raw),'uncompressed_sha256':sha(raw),'chunks':[part]})
        (self.source/'manifest.json').write_text(json.dumps({'format_version':3,'sources':records}))
        self.model=self.root/'crates/crystal-voxel-view/models/fixture.mesh.json';self.model.parent.mkdir(parents=True)
        self.raw=b'{"values":[-0.0,1.234567890123456],"label":"exact"}\n'
        self.gz=gzip.compress(self.raw,mtime=345)
        self.model.write_text(json.dumps({'storage':'geothite-model-gzip-v1','bytes':len(self.raw),'sha256':sha(self.raw),'data':base64.b64encode(self.gz).decode()}))
    def tearDown(self):self.temp.cleanup()
    def snapshot(self):return {p.relative_to(self.root):p.read_bytes()for p in self.root.rglob('*')if p.is_file()}
    def test_migration_is_lossless_preflight_read_only_and_repeat_idempotent(self):
        before=self.snapshot();report=migrate(self.root,check=True)
        self.assertEqual(self.snapshot(),before)
        self.assertIn('art/johto/source/chunks/characters.blend.000.b64',report['changed_paths'])
        migrated=migrate(self.root)
        self.assertEqual(report['model_identities'],migrated['model_identities'])
        self.assertEqual(report['source_identities'],migrated['source_identities'])
        self.assertEqual(read_model_bytes(self.model),self.raw)
        self.assertEqual(self.model.read_bytes(),self.raw)
        self.assertFalse(Path(str(self.model)+'.gz').exists())
        native=self.snapshot();migrate(self.root);self.assertEqual(self.snapshot(),native)
        self.assertFalse((self.source/'chunks').exists())
    def test_bad_final_model_prevents_all_source_writes(self):
        value=json.loads(self.model.read_bytes());value['sha256']='0'*64;self.model.write_text(json.dumps(value))
        before=self.snapshot()
        with self.assertRaises(ValueError):migrate(self.root)
        self.assertEqual(self.snapshot(),before)
    def test_local_native_edits_are_never_overwritten(self):
        (self.source/'characters.blend').write_bytes(b'local edits');before=self.snapshot()
        with self.assertRaises(ValueError):migrate(self.root)
        self.assertEqual(self.snapshot(),before)
    def test_orphan_chunks_and_differing_binary_rejected(self):
        orphan=self.source/'chunks/orphan.b64';orphan.write_bytes(b'bad')
        with self.assertRaises(ValueError):migrate(self.root)
        orphan.unlink();Path(str(self.model)+'.gz').write_bytes(b'local binary edits')
        with self.assertRaises(ValueError):migrate(self.root)
    def test_orphan_binary_prevents_all_writes(self):
        (self.model.parent/'orphan.json.gz').write_bytes(b'local bytes')
        before=self.snapshot()
        with self.assertRaises(ValueError):migrate(self.root)
        self.assertEqual(before,self.snapshot())
    def test_invalid_decoded_models_cannot_partially_write_sources(self):
        for raw in [b'{"storage":"unknown"}', '{"unicode":true}'.encode('utf-16')]:
            self.model.write_text(json.dumps({'storage':'geothite-model-gzip-v1','bytes':len(raw),'sha256':sha(raw),'data':base64.b64encode(gzip.compress(raw)).decode()}))
            before=self.snapshot()
            for check in [True,False]:
                with self.subTest(raw=raw,check=check),self.assertRaises((ValueError,UnicodeDecodeError)):migrate(self.root,check)
                self.assertEqual(before,self.snapshot())
    def test_missing_required_source_cannot_partially_write_sources(self):
        manifest=self.source/'manifest.json';value=json.loads(manifest.read_bytes())
        missing=value['sources'].pop()
        for chunk in missing['chunks']:(self.source/chunk['file']).unlink()
        manifest.write_text(json.dumps(value));before=self.snapshot()
        for check in [True,False]:
            with self.assertRaises(ValueError):migrate(self.root,check)
            self.assertEqual(before,self.snapshot())
    def test_source_chunk_path_escape_is_rejected_before_read(self):
        p=self.source/'manifest.json';value=json.loads(p.read_bytes())
        value['sources'][0]['chunks'][0]['file']='../outside.b64';p.write_text(json.dumps(value))
        before=self.snapshot()
        with self.assertRaises(ValueError):migrate(self.root)
        self.assertEqual(self.snapshot(),before)

if __name__=='__main__':unittest.main()
