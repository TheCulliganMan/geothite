#!/usr/bin/env python3
"""Check native Blender source identities without opening Blender."""
import gzip,hashlib,json,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import johto_art_sources as sources
from johto_art_sources import read_source

class NativeSourceTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.root=Path(self.temp.name)
        self.raw=b'BLENDER-v300'+b'fixture'*12;self.payload=gzip.compress(self.raw,mtime=0)
        self.entry={'file':'fixture.blend','encoding':'gzip-blend','bytes':len(self.payload),'sha256':hashlib.sha256(self.payload).hexdigest(),'uncompressed_bytes':len(self.raw),'uncompressed_sha256':hashlib.sha256(self.raw).hexdigest()}
        (self.root/'fixture.blend').write_bytes(self.payload)
    def tearDown(self):self.temp.cleanup()
    def test_normal_file_is_returned_unchanged(self):self.assertEqual(read_source(self.root,self.entry),self.payload)
    def test_unsafe_name_and_chunk_manifest_rejected(self):
        for change in [{'file':'../fixture.blend'},{'file':'/tmp/fixture.blend'},{'file':'fixture.txt'},{'chunks':[]}]:
            with self.assertRaises(ValueError):read_source(self.root,{**self.entry,**change})
    def test_compressed_and_decoded_identity_checked(self):
        for key in ['bytes','sha256','uncompressed_bytes','uncompressed_sha256']:
            value=0 if key.endswith('bytes') else '0'*64
            with self.subTest(key=key),self.assertRaises(ValueError):read_source(self.root,{**self.entry,key:value})
    def test_wrong_header_and_missing_source_rejected(self):
        raw=b'NOTBLEND';payload=gzip.compress(raw)
        (self.root/'fixture.blend').write_bytes(payload)
        changed={**self.entry,'bytes':len(payload),'sha256':hashlib.sha256(payload).hexdigest(),'uncompressed_bytes':len(raw),'uncompressed_sha256':hashlib.sha256(raw).hexdigest()}
        with self.assertRaises(ValueError):read_source(self.root,changed)
        (self.root/'fixture.blend').unlink()
        with self.assertRaises(FileNotFoundError):read_source(self.root,self.entry)

    def catalog(self):
        other={**self.entry,'file':'other.blend'}
        (self.root/'other.blend').write_bytes(self.payload)
        manifest={'format_version':4,'sources':[self.entry,other]}
        (self.root/'manifest.json').write_text(json.dumps(manifest))
        return patch.multiple(sources,SOURCE_DIR=self.root,REQUIRED_SOURCES={'fixture.blend','other.blend'})

    def test_in_place_blender_edit_refreshes_only_target_identity(self):
        with self.catalog():
            edited=self.raw+b' edited mesh data'
            target=self.root/'fixture.blend'
            target.write_bytes(edited)
            with self.assertRaises(ValueError):list(sources.load_sources())
            item=sources.store_source(target)
            self.assertEqual(gzip.decompress(read_source(self.root,item)),edited)
            self.assertEqual((self.root/'other.blend').read_bytes(),self.payload)
            self.assertEqual(len(list(sources.load_sources())),2)

    def test_other_source_corruption_stops_store_before_any_write(self):
        with self.catalog():
            target=self.root/'fixture.blend'
            target.write_bytes(self.raw+b' intentional edit')
            (self.root/'other.blend').write_bytes(b'corrupt unrelated source')
            before={p.name:p.read_bytes()for p in self.root.iterdir()}
            with self.assertRaises(ValueError):sources.store_source(target)
            self.assertEqual(before,{p.name:p.read_bytes()for p in self.root.iterdir()})

if __name__=='__main__':unittest.main()
