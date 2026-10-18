#!/usr/bin/env python3
"""Public Pidgeotto readers/builders consume one canonical standard skin."""
import copy,hashlib,importlib.util,json,struct,sys,tempfile,unittest
from pathlib import Path
import animated_glb as glb
import pidgeotto_glb as public
import pidgeotto_skin_glb as skin
from skin_glb_test_support import decode,encode
ROOT=Path(__file__).resolve().parents[1]
MODELS=ROOT/'crates/crystal-voxel-view/models'
class PidgeottoGlbTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.path=MODELS/'battle_species/pidgeotto.glb';cls.blob=cls.path.read_bytes();cls.model=public.read_pidgeotto(cls.path);cls.original=skin.original_model(cls.model);cls.doc,cls.binary=decode(cls.blob)
 def test_original_geometry_fingerprint_survives_the_skin_migration(self):
  h=hashlib.sha256()
  for p in self.original['primitives']:
   n=p['part'].encode();h.update(struct.pack('<I',len(n)));h.update(n)
   for k in (*glb.GEOMETRY_KEYS,'base_color'):h.update(struct.pack('<I',len(p[k])));h.update(glb.packed(p[k],'I' if k=='indices' else 'f'))
  self.assertEqual(h.hexdigest(),'171a58c257ea427a62984594386d699d36afb82fe5b2e1377405b545526ad3cf')
 def test_canonical_export_and_contour_rebuild_are_idempotent(self):
  from pidgeotto_contours import prepare
  self.assertEqual(public.export_pidgeotto(self.model),self.blob);self.assertEqual(public.export_pidgeotto(self.original),self.blob);self.assertEqual(public.export_pidgeotto(prepare(self.original)),self.blob);self.assertEqual(public.load_pidgeotto(self.path.parent),self.model)
 def test_decimal_builder_input_refines_like_serialized_f32_geometry(self):
  from pidgeotto_contours import prepare,refine
  from sculpt_geometry import loft
  p=loft('Bird body',(.6,.4,.2),[(.1+j*.12,w,w*.8,.033333) for j,w in enumerate((.035,.10,.15,.13,.07,.02))],12)
  for key in ('positions','normals'):p[key]=[round(v,6) for v in p[key]]
  raw=dict(name='pidgeotto',version=1,coordinate_system=glb.COORDINATES,primitives=[p]);serialized=copy.deepcopy(raw)
  for key in ('positions','normals'):serialized['primitives'][0][key]=list(struct.unpack('<'+str(len(p[key]))+'f',struct.pack('<'+str(len(p[key]))+'f',*p[key])))
  self.assertNotEqual(skin.geometry_digest(dict(primitives=[refine(p)])),skin.geometry_digest(dict(primitives=[refine(serialized['primitives'][0])])))
  self.assertEqual(skin.geometry_digest(prepare(raw)),skin.geometry_digest(prepare(serialized)))
 def test_standard_skin_and_all_three_source_cues_are_present(self):
  d=self.doc;self.assertEqual(len(d['skins']),1);self.assertEqual(len(d['skins'][0]['joints']),14);self.assertEqual(len(d['meshes']),1);self.assertEqual(len(d['meshes'][0]['primitives']),34);self.assertEqual([c['name'] for c in d['animations']],['pidgeotto.idle','pidgeotto.attack','pidgeotto.hit'])
  self.assertFalse(self.path.with_suffix('.mesh.json').exists())
 def test_canonical_selection_rejects_duplicate_json(self):
  with tempfile.TemporaryDirectory() as t:
   p=Path(t);(p/public.FILE).write_bytes(self.blob);self.assertEqual(public.species_paths(p),{'pidgeotto':p/public.FILE});self.assertEqual(public.read_pidgeotto(p/public.FILE),self.model);(p/'pidgeotto.mesh.json').write_text('{}')
   with self.assertRaisesRegex(ValueError,'duplicate'):public.species_paths(p)
 def test_malformed_schema_and_binary_are_rejected(self):
  mutations=[lambda d:d.update(extensionsRequired=['KHR_draco_mesh_compression']),lambda d:d['nodes'][0].update(scale=[2,1,1]),lambda d:d['nodes'][1].update(children=[0]),lambda d:d['skins'][0]['joints'].pop(),lambda d:d['materials'][0].update(doubleSided=True),lambda d:d['materials'][0]['pbrMetallicRoughness'].update(metallicFactor=.5),lambda d:d['animations'][0]['channels'][0]['target'].update(path='translation'),lambda d:d['animations'][0]['samplers'][0].update(interpolation='STEP'),lambda d:d['animations'].clear(),lambda d:d['accessors'][0].update(count=999999999),lambda d:d['bufferViews'][0].update(byteOffset=999999999),lambda d:d['buffers'][0].update(uri='other.bin'),lambda d:d['meshes'][0]['primitives'][0]['attributes'].update(TEXCOORD_0=0)]
  for edit in mutations:
   d=copy.deepcopy(self.doc);edit(d)
   with self.assertRaises(ValueError):public.decode_pidgeotto(encode(d,self.binary))
  for blob in (self.blob[:-1],self.blob+b'\0',b'bad',b'\0'*(public.MAX_BYTES+1)):
   with self.assertRaises(ValueError):public.decode_pidgeotto(blob)
 def test_position_accessors_are_bounded_before_allocating_binary_formats(self):
  position=self.doc['meshes'][0]['primitives'][0]['attributes']['POSITION']
  for count in (0,20001,2147483647):
   d=copy.deepcopy(self.doc);d['accessors'][position]['count']=count
   with self.assertRaises(ValueError):public.decode_pidgeotto(encode(d,self.binary))
  d=copy.deepcopy(self.doc);d['accessors'][position]['bufferView']=2147483647
  with self.assertRaises(ValueError):public.decode_pidgeotto(encode(d,self.binary))
 def test_neutral_stl_export_matches_transient_source_facets(self):
  spec=importlib.util.spec_from_file_location('pidgeotto_stl',ROOT/'tools/export-model-stl.py');stl=importlib.util.module_from_spec(spec);sys.modules[spec.name]=stl;spec.loader.exec_module(stl)
  canonical=stl.ModelReader(MODELS).read(self.path)
  with tempfile.TemporaryDirectory() as t:
   p=Path(t);source=p/'pidgeotto.mesh.json';source.write_text(json.dumps(self.model));raw=stl.ModelReader(p).read(source);self.assertEqual(stl.encode_stl(canonical),stl.encode_stl(raw))
if __name__=='__main__':unittest.main()
