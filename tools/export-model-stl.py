#!/usr/bin/env python3
"""Export canonical indexed model JSON to deterministic GitHub-viewable binary STL.

No dependencies. The input meshes are authoritative: preserve every indexed
triangle, including degenerate triangles, in source order. Rig exports use their
unanimated bind pose, matching johto_characters.rs. See MODEL-STL.md.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
import math
from pathlib import Path
import struct
import sys
from typing import Iterator


STL_HEADER = b"Geothite exact indexed mesh; bind pose; (x,y,z)->(x,-z,y); no color".ljust(80, b"\0")
RECORD = struct.Struct("<12fH")
GITHUB_LIMIT = 10_000_000
JOINT_NAMES = (
    "pelvis", "torso", "head", "eyes", "upper_arm_l", "forearm_l", "hand_l",
    "upper_arm_r", "forearm_r", "hand_r", "thigh_l", "shin_l", "shoe_l",
    "thigh_r", "shin_r", "shoe_r",
)
JOINT_PARENTS = (None, 0, 1, 2, 1, 4, 5, 1, 7, 8, 0, 10, 11, 0, 13, 14)


def f32(value: float) -> float:
    """Match Rust's f32 mesh storage and each bind-translation addition."""
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError("mesh values must be numbers")
    try:
        result = struct.unpack("<f", struct.pack("<f", value))[0]
    except (OverflowError, struct.error) as error:
        raise ValueError("mesh value is outside f32 range") from error
    if not math.isfinite(result):
        raise ValueError("mesh values must be finite f32 numbers")
    return result


def triple(values: list) -> tuple[float, float, float]:
    if not isinstance(values, list) or len(values) != 3:
        raise ValueError("expected a three-component vector")
    return tuple(f32(value) for value in values)


def rotate(point):
    # A proper +90 degree rotation about X: determinant +1, no reflection.
    return point[0], -point[2], point[1]


def facet_normal(vertices):
    a, b, c = vertices
    ab = tuple(b[i] - a[i] for i in range(3))
    ac = tuple(c[i] - a[i] for i in range(3))
    cross = (
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    )
    length = math.hypot(*cross)
    # Preserve degenerate source triangles rather than silently repairing art.
    return tuple(f32(value / length) for value in cross) if length else (0.0, 0.0, 0.0)


@dataclass(frozen=True)
class Primitive:
    positions: tuple
    indices: tuple
    translation: tuple | None = None

    def triangles(self) -> Iterator[tuple]:
        positions = self.positions
        if self.translation is not None:
            positions = tuple(
                tuple(f32(p[i] + self.translation[i]) for i in range(3))
                for p in positions
            )
        positions = tuple(rotate(p) for p in positions)
        for offset in range(0, len(self.indices), 3):
            yield tuple(positions[i] for i in self.indices[offset:offset + 3])


def validate_geometry(source: dict) -> tuple[tuple, tuple]:
    positions = source.get("positions")
    normals = source.get("normals")
    indices = source.get("indices")
    if not isinstance(positions, list) or not positions or len(positions) % 3:
        raise ValueError("positions must be a nonempty raw numeric array of XYZ triples")
    if not isinstance(normals, list) or len(normals) != len(positions):
        raise ValueError("normals must match positions")
    points = tuple(triple(positions[i:i + 3]) for i in range(0, len(positions), 3))
    for i in range(0, len(normals), 3):
        if math.hypot(*triple(normals[i:i + 3])) < 0.001:
            raise ValueError("source vertex normal must be nonzero")
    if not isinstance(indices, list) or not indices or len(indices) % 3:
        raise ValueError("indices must be a nonempty raw array of triangle indices")
    if any(type(i) is not int or i < 0 or i >= len(points) for i in indices):
        raise ValueError("triangle index is out of range")
    return points, tuple(indices)


def validate_color(source: dict):
    color = source.get("base_color")
    if not isinstance(color, list) or len(color) != 4 or any(not 0 <= f32(x) <= 1 for x in color):
        raise ValueError("base_color must contain four finite components in [0, 1]")


class ModelReader:
    def __init__(self, root: Path):
        self.root = root
        self.library = None

    def geometry_library(self):
        if self.library is None:
            library = json.loads((self.root / "johto_characters/shared.geometry.json").read_text())
            identity = library.get("id", "")
            if library.get("version") != 1 or len(identity) != 64 or any(c not in "0123456789abcdef" for c in identity):
                raise ValueError("unsupported character geometry library")
            geometries = library.get("geometries", [])
            if not geometries:
                raise ValueError("empty character geometry library")
            self.library = identity, tuple(validate_geometry(g) for g in geometries)
        return self.library

    def read(self, path: Path) -> tuple[Primitive, ...]:
        source = json.loads(path.read_text())
        if "joints" in source:
            return self.read_rig(source)
        if "primitives" not in source or not source["primitives"]:
            raise ValueError("expected a mesh with raw indexed primitives")
        dimensions = None
        if "dimensions_pixels" in source:
            dimensions = triple(source["dimensions_pixels"])
            if any(value <= 0 for value in dimensions):
                raise ValueError("dimensions_pixels must contain three positive finite f32 dimensions")
        output = []
        for primitive in source["primitives"]:
            validate_color(primitive)
            # Current static Rust loaders consume already-baked coordinates.
            # Reject future transform fields instead of silently ignoring them.
            if any(k in primitive for k in ("translation", "rotation", "scale", "matrix", "transform", "geometry")):
                raise ValueError("unsupported static primitive transform or reference")
            positions, indices = validate_geometry(primitive)
            if dimensions is not None:
                # The lighthouse/ship/traditional authoring helpers divide each
                # coordinate by dimensions_pixels. Decode that storage transform
                # in native pixel units before the STL axis rotation. Matching
                # runtime precision requires rounding the source AND the product.
                # Generic bounds/dimensions are metadata, never implicit scales.
                positions = tuple(
                    tuple(f32(point[axis] * dimensions[axis]) for axis in range(3))
                    for point in positions
                )
            output.append(Primitive(positions, indices))
        count = sum(len(p.indices) // 3 for p in output)
        if "triangle_count" in source and source["triangle_count"] != count:
            raise ValueError("authored triangle_count does not match indexed geometry")
        return tuple(output)

    def read_rig(self, source: dict) -> tuple[Primitive, ...]:
        version = source.get("version")
        joints = source["joints"]
        if version not in (1, 2) or len(joints) != len(JOINT_NAMES):
            raise ValueError("unsupported articulated character schema")
        library = None
        if version == 2:
            identity, library = self.geometry_library()
            if source.get("geometry_library") != identity:
                raise ValueError("character geometry library identity mismatch")
        elif source.get("geometry_library") is not None:
            raise ValueError("inline rig must not reference a geometry library")
        worlds = []
        output = []
        for index, joint in enumerate(joints):
            parent = joint.get("parent")
            if joint.get("name") != JOINT_NAMES[index] or parent != JOINT_PARENTS[index]:
                raise ValueError("invalid character joint or parent")
            if any(k in joint for k in ("rotation", "scale", "matrix", "transform")):
                raise ValueError("unsupported character joint transform")
            translation = triple(joint["translation"])
            parent_world = (0.0, 0.0, 0.0) if parent is None else worlds[parent]
            world = tuple(f32(translation[i] + parent_world[i]) for i in range(3))
            worlds.append(world)
            if not joint["primitives"]:
                raise ValueError("character joint must own real geometry")
            for primitive in joint["primitives"]:
                validate_color(primitive)
                if version == 1:
                    if primitive.get("geometry") is not None:
                        raise ValueError("inline rig contains a shared reference")
                    geometry = validate_geometry(primitive)
                else:
                    ref = primitive.get("geometry")
                    if any(primitive.get(k) is not None for k in ("positions", "normals", "indices")):
                        raise ValueError("shared rig contains inline geometry")
                    if type(ref) is not int or ref < 0 or ref >= len(library):
                        raise ValueError("invalid character geometry reference")
                    geometry = library[ref]
                output.append(Primitive(*geometry, translation=world))
        return tuple(output)


def encode_stl(primitives: tuple[Primitive, ...]) -> bytes:
    count = sum(len(p.indices) // 3 for p in primitives)
    result = bytearray(STL_HEADER + struct.pack("<I", count))
    for primitive in primitives:
        for vertices in primitive.triangles():
            values = facet_normal(vertices) + tuple(v for p in vertices for v in p)
            result.extend(RECORD.pack(*values, 0))
    return bytes(result)


def validate_stl(data: bytes, primitives: tuple[Primitive, ...]):
    """Read back every facet and compare all float32 vertices and normals."""
    count = sum(len(p.indices) // 3 for p in primitives)
    if len(data) < 84 or struct.unpack_from("<I", data, 80)[0] != count:
        raise ValueError("STL triangle count differs from source")
    if len(data) != 84 + count * RECORD.size:
        raise ValueError("STL length differs from triangle count")
    offset = 84
    for primitive in primitives:
        for vertices in primitive.triangles():
            normal_and_vertices = RECORD.unpack_from(data, offset)
            expected = facet_normal(vertices) + tuple(v for p in vertices for v in p)
            # Compare bits, including signed zero, not approximate decimal values.
            if struct.pack("<12f", *normal_and_vertices[:12]) != struct.pack("<12f", *expected):
                raise ValueError(f"STL facet {((offset - 84) // RECORD.size)} differs from source vertices or winding normal")
            if normal_and_vertices[12] != 0:
                raise ValueError("unexpected STL attribute/color bytes")
            offset += RECORD.size


def output_path(source: Path, source_root: Path, output_root: Path) -> Path:
    relative = source.relative_to(source_root)
    for suffix in (".mesh.json", ".rig.json"):
        if relative.name.endswith(suffix):
            return output_root / relative.with_name(relative.name[:-len(suffix)] + ".stl")
    raise ValueError(f"not a model document: {relative}")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    default_root = Path(__file__).resolve().parents[1] / "crates/crystal-voxel-view/models"
    parser.add_argument("--models-root", type=Path, default=default_root)
    parser.add_argument("--output-root", type=Path, help="default: adjacent to each canonical model")
    parser.add_argument("--model", action="append", help="one relative .mesh.json or .rig.json path; repeatable")
    parser.add_argument("--check", action="store_true", help="verify existing files without changing them")
    parser.add_argument("--list-output-paths", action="store_true", help="after successful verification, print only verified paths relative to output-root on stdout")
    args = parser.parse_args(argv)
    root = args.models_root.resolve()
    output_root = (args.output_root or root).resolve()
    if not root.is_dir():
        parser.error(f"models root does not exist: {root}")
    sources = sorted([root / p for p in args.model] if args.model else [*root.rglob("*.mesh.json"), *root.rglob("*.rig.json")])
    if not sources:
        parser.error("no canonical model documents found")
    # Full-tree runs have an exact one-to-one model/STL contract. Do not leave
    # stale derivatives that misleadingly look like current authored models.
    expected_outputs = {output_path(source, root, output_root) for source in sources}
    if not args.model:
        unexpected = sorted(set(output_root.rglob("*.stl")) - expected_outputs)
        if unexpected:
            print("Unexpected/orphan STL files: " + ", ".join(str(p.relative_to(output_root)) for p in unexpected), file=sys.stderr)
            return 1
    reader = ModelReader(root)
    total_bytes = total_triangles = 0
    largest = (0, "")
    for source in sources:
        try:
            if not source.resolve().is_relative_to(root):
                raise ValueError("model must be inside models root")
            primitives = reader.read(source)
            data = encode_stl(primitives)
            if len(data) > GITHUB_LIMIT:
                raise ValueError(f"STL is {len(data)} bytes, above GitHub's 10 MB limit; do not silently split geometry")
            destination = output_path(source, root, output_root)
            if args.check:
                actual = destination.read_bytes()
                validate_stl(actual, primitives)
                if actual != data:
                    raise ValueError("STL is not the deterministic canonical export")
            else:
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(data)
                validate_stl(destination.read_bytes(), primitives)
            total_bytes += len(data)
            total_triangles += sum(len(p.indices) // 3 for p in primitives)
            largest = max(largest, (len(data), str(destination.relative_to(output_root))))
        except (ValueError, OSError, KeyError, TypeError) as error:
            print(f"{source.relative_to(root)}: {error}", file=sys.stderr)
            return 1
    verb = "Verified" if args.check else "Exported and verified"
    summary_stream = sys.stderr if args.list_output_paths else sys.stdout
    print(f"{verb} {len(sources)} models; {total_triangles:,} triangles; {total_bytes:,} bytes", file=summary_stream)
    print(f"Largest: {largest[1]} ({largest[0]:,} bytes); all within GitHub's 10 MB viewer limit", file=summary_stream)
    if args.list_output_paths:
        for path in sorted(expected_outputs):
            print(path.relative_to(output_root).as_posix())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
