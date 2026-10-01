#!/usr/bin/env python3
"""Regression checks for the GitHub STL export (standard library only)."""

from contextlib import redirect_stderr, redirect_stdout
import importlib.util
import io
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location("model_stl", Path(__file__).with_name("export-model-stl.py"))
stl = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = stl
SPEC.loader.exec_module(stl)


def geometry():
    return {
        "positions": [0, 0, 0, 1, 0, 0, 0, 1, 0],
        "normals": [0, 0, 1] * 3,
        "indices": [0, 1, 2],
    }


class ExportTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def write(self, name, data):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(data))
        return path

    def static(self):
        primitive = geometry() | {"base_color": [0.5, 0.3, 0.8, 1]}
        return {"primitives": [primitive]}

    def rig(self, version=2):
        identity = "a" * 64
        self.write("johto_characters/shared.geometry.json", {
            "version": 1, "id": identity, "geometries": [geometry()]
        })
        joints = []
        for name, parent in zip(stl.JOINT_NAMES, stl.JOINT_PARENTS):
            primitive = {"geometry": 0} if version == 2 else geometry()
            primitive["base_color"] = [0.2, 0.4, 0.6, 1]
            joints.append({"name": name, "parent": parent,
                           "translation": [0.25, 0.5, 0.125], "primitives": [primitive]})
        result = {"version": version, "joints": joints}
        if version == 2:
            result["geometry_library"] = identity
        return result

    def test_proper_axis_rotation_preserves_triangle_order_and_winding(self):
        source = self.write("triangle.mesh.json", self.static())
        primitives = stl.ModelReader(self.root).read(source)
        data = stl.encode_stl(primitives)
        self.assertEqual(len(data), 134)
        self.assertEqual(struct.unpack_from("<I", data, 80)[0], 1)
        record = struct.unpack_from("<12fH", data, 84)
        self.assertEqual(record[:3], (0, -1, 0))
        self.assertEqual(record[3:12], (0, 0, 0, 1, 0, 0, 0, 0, 1))
        self.assertEqual(record[12], 0)
        stl.validate_stl(data, primitives)

    def test_shared_rig_resolves_every_ancestor_in_bind_pose(self):
        source = self.write("johto_characters/sample.rig.json", self.rig())
        primitives = stl.ModelReader(self.root).read(source)
        self.assertEqual(len(primitives), 16)
        # Eyes are three descendants below pelvis, so four translations apply.
        self.assertEqual(primitives[3].translation, (1, 2, 0.5))
        self.assertEqual(next(primitives[3].triangles())[0], (1, -0.5, 2))
        data = stl.encode_stl(primitives)
        self.assertEqual(struct.unpack_from("<I", data, 80)[0], 16)
        stl.validate_stl(data, primitives)

    def test_shared_and_expanded_inline_rigs_are_identical(self):
        reader = stl.ModelReader(self.root)
        shared = reader.read(self.write("shared.rig.json", self.rig(2)))
        inline = reader.read(self.write("inline.rig.json", self.rig(1)))
        self.assertEqual(stl.encode_stl(shared), stl.encode_stl(inline))

    def test_translation_is_rounded_after_each_runtime_f32_operation(self):
        rig = self.rig()
        rig["joints"][0]["translation"][0] = 16_777_216
        rig["joints"][1]["translation"][0] = 1
        rig["joints"][2]["translation"][0] = -16_777_216
        primitives = stl.ModelReader(self.root).read(self.write("rounded.rig.json", rig))
        self.assertEqual(primitives[2].translation[0], 0)

    def test_bad_library_identity_and_reference_are_rejected(self):
        for field, value, expected in (
            ("identity", "b" * 64, "identity mismatch"),
            ("reference", 1, "invalid character geometry reference"),
            ("reference", -1, "invalid character geometry reference"),
        ):
            with self.subTest(field=field, value=value):
                rig = self.rig()
                if field == "identity":
                    rig["geometry_library"] = value
                else:
                    rig["joints"][0]["primitives"][0]["geometry"] = value
                with self.assertRaisesRegex(ValueError, expected):
                    stl.ModelReader(self.root).read(self.write("bad.rig.json", rig))

    def test_invalid_parent_and_unsupported_transforms_are_rejected(self):
        rig = self.rig()
        rig["joints"][1]["parent"] = 1
        with self.assertRaisesRegex(ValueError, "invalid character joint or parent"):
            stl.ModelReader(self.root).read(self.write("cycle.rig.json", rig))
        rig = self.rig()
        rig["joints"][0]["rotation"] = [0, 0, 0, 1]
        with self.assertRaisesRegex(ValueError, "unsupported character joint transform"):
            stl.ModelReader(self.root).read(self.write("transform.rig.json", rig))

    def test_invalid_geometry_is_rejected(self):
        for field, value in (("positions", "base64"), ("indices", [0, 1, 3]),
                             ("indices", [0, 1]), ("normals", [0] * 9),
                             ("positions", [float("nan")] * 9)):
            with self.subTest(field=field, value=value):
                source = self.static()
                source["primitives"][0][field] = value
                with self.assertRaises(ValueError):
                    stl.ModelReader(self.root).read(self.write("bad.mesh.json", source))

    def test_normalized_dimensions_restore_authored_proportions(self):
        source = self.static()
        source["dimensions_pixels"] = [192, 16, 4]
        source["primitives"][0]["positions"] = [0, 0, 0, 1, 0, 0, 0, 1, 1]
        primitives = stl.ModelReader(self.root).read(self.write("backdrop.mesh.json", source))
        self.assertEqual(next(primitives[0].triangles()), ((0, 0, 0), (192, 0, 0), (0, -4, 16)))
        data = stl.encode_stl(primitives)
        stl.validate_stl(data, primitives)
        # Nonuniform decoding must recompute the face normal from the actual
        # scaled triangle, not rotate the normalized buffer's vertex normals.
        normal = struct.unpack_from("<3f", data, 84)
        self.assertAlmostEqual(normal[1], -16 / (16**2 + 4**2)**0.5, places=6)
        self.assertAlmostEqual(normal[2], -4 / (16**2 + 4**2)**0.5, places=6)

    def test_dimensions_multiplication_uses_runtime_f32_steps(self):
        source = self.static()
        source["dimensions_pixels"] = [9, 16, 4]
        source["primitives"][0]["positions"][0] = 0.2
        primitives = stl.ModelReader(self.root).read(self.write("scaled.mesh.json", source))
        actual = next(primitives[0].triangles())[0][0]
        self.assertEqual(actual, stl.f32(stl.f32(0.2) * stl.f32(9)))
        self.assertNotEqual(actual, stl.f32(0.2 * 9))

    def test_invalid_explicit_dimensions_are_rejected(self):
        for dimensions in (None, "192,16,4", [192, 16], [192, 16, 4, 1],
                           [192, 0, 4], [192, -16, 4], [192, True, 4],
                           [192, float("nan"), 4], [192, float("inf"), 4],
                           [192, 1e-50, 4], [192, 1e50, 4]):
            with self.subTest(dimensions=dimensions):
                source = self.static()
                source["dimensions_pixels"] = dimensions
                with self.assertRaises(ValueError):
                    stl.ModelReader(self.root).read(self.write("bad-dimensions.mesh.json", source))

    def test_generic_dimensions_and_bounds_do_not_rescale_geometry(self):
        source = self.static()
        original = stl.ModelReader(self.root).read(self.write("plain.mesh.json", source))
        source["dimensions"] = [192, 16, 4]
        source["bounds"] = {"min": [0, 0, 0], "max": [192, 16, 4]}
        metadata = stl.ModelReader(self.root).read(self.write("metadata.mesh.json", source))
        self.assertEqual(stl.encode_stl(original), stl.encode_stl(metadata))

    def test_all_facets_and_degenerate_triangles_are_preserved(self):
        source = self.static()
        source["primitives"][0]["indices"] += [2, 1, 0, 0, 0, 0]
        primitives = stl.ModelReader(self.root).read(self.write("all.mesh.json", source))
        data = stl.encode_stl(primitives)
        self.assertEqual(struct.unpack_from("<I", data, 80)[0], 3)
        self.assertEqual(struct.unpack_from("<3f", data, 134), (0, 1, 0))
        self.assertEqual(struct.unpack_from("<3f", data, 184), (0, 0, 0))
        stl.validate_stl(data, primitives)

    def test_validator_detects_vertex_normal_count_and_length_corruption(self):
        primitives = stl.ModelReader(self.root).read(self.write("mesh.mesh.json", self.static()))
        good = stl.encode_stl(primitives)
        for offset in (80, 84, 96, 132):
            with self.subTest(offset=offset):
                data = bytearray(good)
                data[offset] ^= 1
                with self.assertRaises(ValueError):
                    stl.validate_stl(bytes(data), primitives)
        with self.assertRaises(ValueError):
            stl.validate_stl(good + b"\0", primitives)

    def test_export_is_deterministic_and_read_only_check_detects_changes(self):
        self.write("nested/model.mesh.json", self.static())
        output = self.root / "output"
        args = ["--models-root", str(self.root), "--output-root", str(output)]
        self.assertEqual(stl.main(args), 0)
        destination = output / "nested/model.stl"
        before = destination.read_bytes()
        self.assertEqual(stl.main(args), 0)
        self.assertEqual(before, destination.read_bytes())
        self.assertEqual(stl.main(args + ["--check"]), 0)
        destination.write_bytes(before[:-1])
        self.assertEqual(stl.main(args + ["--check"]), 1)
        self.assertEqual(destination.read_bytes(), before[:-1])

    def test_output_paths_are_adjacent_to_each_source_stem(self):
        for suffix in ("mesh.json", "rig.json"):
            self.assertEqual(stl.output_path(self.root / f"family/name.{suffix}", self.root, self.root),
                             self.root / "family/name.stl")

    def test_full_tree_rejects_unexpected_stl_and_missing_expected_stl(self):
        self.write("model.mesh.json", self.static())
        output = self.root / "output"
        args = ["--models-root", str(self.root), "--output-root", str(output)]
        with redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            self.assertEqual(stl.main(args), 0)
            (output / "orphan.stl").write_bytes(b"old export")
            self.assertEqual(stl.main(args + ["--check"]), 1)
            (output / "orphan.stl").unlink()
            (output / "model.stl").unlink()
            self.assertEqual(stl.main(args + ["--check"]), 1)

    def test_path_list_is_complete_and_only_printed_after_success(self):
        self.write("nested/model.mesh.json", self.static())
        output = self.root / "output"
        args = ["--models-root", str(self.root), "--output-root", str(output), "--list-output-paths"]
        with redirect_stdout(io.StringIO()) as paths, redirect_stderr(io.StringIO()):
            self.assertEqual(stl.main(args), 0)
        self.assertEqual(paths.getvalue(), "nested/model.stl\n")
        (output / "nested/model.stl").write_bytes(b"broken")
        with redirect_stdout(io.StringIO()) as paths, redirect_stderr(io.StringIO()):
            self.assertEqual(stl.main(args + ["--check"]), 1)
        self.assertEqual(paths.getvalue(), "")


if __name__ == "__main__":
    unittest.main()
