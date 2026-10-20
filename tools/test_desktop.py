"""Filesystem contracts that protect installed apps and exclude source artifacts."""
import importlib.util
from pathlib import Path
import plistlib
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("desktop", Path(__file__).with_name("desktop.py"))
desktop = importlib.util.module_from_spec(spec)
spec.loader.exec_module(desktop)


class DesktopPackagingTests(unittest.TestCase):
    def test_artwork_copy_excludes_sources_and_rejects_missing_lfs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            art = root / "input"
            rigs = art / "people/kenney"
            rigs.mkdir(parents=True)
            for name in ["human-male", "human-female"]:
                (rigs / f"{name}.glb").write_bytes(b"glTF runtime fixture")
            (art / "private.blend").write_bytes(b"source fixture")
            (art / "scratch.crystalpack").write_bytes(b"pack fixture")
            desktop.copy_artwork(art, root / "output")
            self.assertTrue((root / "output/people/kenney/human-male.glb").is_file())
            self.assertFalse((root / "output/private.blend").exists())
            self.assertFalse((root / "output/scratch.crystalpack").exists())
            (rigs / "human-male.glb").write_text("version https://git-lfs.github.com/spec/v1\n")
            with self.assertRaisesRegex(RuntimeError, "Git LFS pointer"):
                desktop.copy_artwork(art, root / "invalid")

    def test_failed_update_preserves_previous_install(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source, destination = root / "source", root / "Geothite.app"
            source.mkdir()
            destination.mkdir()
            (destination / "old").write_text("last working app")
            with patch.object(desktop.shutil, "copytree", side_effect=OSError("disk full")):
                with self.assertRaises(OSError):
                    desktop.replace_directory(source, destination)
            self.assertEqual((destination / "old").read_text(), "last working app")
            (source / "new").write_text("next app")
            desktop.replace_directory(source, destination)
            self.assertEqual((destination / "new").read_text(), "next app")
            self.assertFalse((destination / "old").exists())
            self.assertEqual(sorted(path.name for path in root.iterdir()), ["Geothite.app", "source"])

    def test_mac_metadata_identifies_a_gui_app(self):
        metadata = plistlib.loads(plistlib.dumps(desktop.plist("0.1.0")))
        self.assertEqual(metadata["CFBundlePackageType"], "APPL")
        self.assertEqual(metadata["CFBundleExecutable"], "Geothite")
        self.assertNotIn("LSUIElement", metadata)
        self.assertNotIn("LSBackgroundOnly", metadata)


if __name__ == "__main__":
    unittest.main()
