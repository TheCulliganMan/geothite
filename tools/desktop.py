#!/usr/bin/env python3
"""Native desktop packaging. All products stay in ignored local directories."""
import argparse
import hashlib
import math
import os
from pathlib import Path
import platform
import plistlib
import re
import shutil
import struct
import subprocess
import sys
import tarfile
import tempfile
import time
import zlib

ROOT = Path(__file__).resolve().parent.parent
DIST = ROOT / ".local/desktop-dist"
APP_ID = "com.geothite.game"


def run(*args, **kwargs):
    print("+", " ".join(map(str, args)), flush=True)
    return subprocess.run(list(map(str, args)), check=True, **kwargs)


def icon(path, size=1024):
    """Rasterize the original faceted crystal without packaging dependencies."""
    facets = [
        ([(.50, .08), (.81, .35), (.71, .71), (.50, .92), (.19, .64), (.21, .32)], (194, 248, 255, 255)),
        ([(.50, .11), (.79, .36), (.42, .47)], (118, 229, 241, 255)),
        ([(.50, .11), (.42, .47), (.23, .33)], (73, 185, 221, 255)),
        ([(.23, .33), (.42, .47), (.50, .89), (.21, .63)], (39, 135, 182, 255)),
        ([(.42, .47), (.79, .36), (.69, .70), (.50, .89)], (67, 206, 223, 255)),
    ]
    pixels = bytearray()
    radius = size * .20
    for y in range(size):
        row = bytearray(size * 4)
        dy = max(radius - y - .5, y + .5 - (size - radius), 0)
        inset = math.ceil(radius - math.sqrt(max(radius * radius - dy * dy, 0)))
        shade = int(36 - 13 * y / size)
        row[inset * 4:(size - inset) * 4] = bytes((shade, shade + 9, shade + 25, 255)) * (size - inset * 2)
        for points, color in facets:
            intersections = []
            for (ax, ay), (bx, by) in zip(points, points[1:] + points[:1]):
                ay, by = ay * size, by * size
                if min(ay, by) <= y + .5 < max(ay, by):
                    intersections.append(size * (ax + (bx - ax) * (y + .5 - ay) / (by - ay)))
            if intersections:
                left, right = math.ceil(min(intersections)), math.ceil(max(intersections))
                row[left * 4:right * 4] = bytes(color) * (right - left)
        pixels.extend(b"\0" + row)

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

    path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0))
                     + chunk(b"IDAT", zlib.compress(pixels)) + chunk(b"IEND", b""))


def copy_artwork(source, destination):
    required = [source / "people/kenney/human-male.glb", source / "people/kenney/human-female.glb"]
    if not all(path.is_file() for path in required):
        raise RuntimeError("Missing character artwork. Run git lfs pull first.")
    for path in source.rglob("*"):
        if not path.is_file() or path.suffix.lower() not in {".glb", ".png", ".jpg", ".jpeg", ".json", ".txt", ".md"}:
            continue
        if path.is_symlink():
            raise RuntimeError(f"Artwork must not contain symlinks: {path}")
        with path.open("rb") as stream:
            if stream.read(80).startswith(b"version https://git-lfs.github.com/spec/v1"):
                raise RuntimeError(f"Git LFS pointer instead of artwork: {path}; run git lfs pull")
        output = destination / path.relative_to(source)
        output.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, output)


def plist(version):
    return {
        "CFBundleName": "Geothite", "CFBundleDisplayName": "Geothite",
        "CFBundleIdentifier": APP_ID, "CFBundleExecutable": "Geothite",
        "CFBundlePackageType": "APPL", "CFBundleIconFile": "Geothite.icns",
        "CFBundleInfoDictionaryVersion": "6.0", "CFBundleDevelopmentRegion": "en",
        "CFBundleShortVersionString": version, "CFBundleVersion": version,
        "LSApplicationCategoryType": "public.app-category.role-playing-games",
        "LSMinimumSystemVersion": "12.0", "NSHighResolutionCapable": True,
        "NSSupportsAutomaticGraphicsSwitching": True,
        "NSHumanReadableCopyright": "Geothite contributors. ISC license.",
    }


def build(target, profile, jobs):
    optimization = []
    if profile != "release":
        for crate in ["crystal-core", "crystal-assets", "crystal-runtime", "crystal-bevy", "crystal-voxel-view", "crystal-net"]:
            optimization += ["--config", f"profile.{profile}.package.{crate}.opt-level=1",
                             "--config", f"profile.{profile}.package.{crate}.incremental=true"]
    run("cargo", "build", "--locked", "--profile", profile, "--jobs", jobs,
        *optimization,
        "-p", "crystal-bevy", "-p", "crystal-assets", "-p", "crystal-voxel-view",
        "--features", "crystal-bevy/voxel-view,crystal-bevy/desktop-app",
        "--bin", "crystal-bevy", "--bin", "verify_crystalpack",
        "--bin", "bundle_open_models", "--bin", "bundle_open_people",
        cwd=ROOT, env={**os.environ, "CARGO_TARGET_DIR": str(target)})


def validate(binary_dir, pack, art, scratch):
    for name in ["crystal-bevy", "verify_crystalpack", "bundle_open_models", "bundle_open_people"]:
        if not (binary_dir / name).is_file():
            raise RuntimeError(f"Missing {binary_dir / name}. Run just desktop-build first.")
    run(binary_dir / "verify_crystalpack", pack, "--desktop")
    run(binary_dir / "bundle_open_models", art, scratch / "scenery.json")
    run(binary_dir / "bundle_open_people", art, scratch / "people.json")


def sign(app, identity):
    arguments = ["codesign", "--force", "--sign", identity]
    if identity != "-":
        arguments += ["--options", "runtime", "--timestamp"]
    run(*arguments, app)
    run("codesign", "--verify", "--deep", "--strict", app)


def stage_macos(scratch, binary_dir, pack, art, version, identity):
    app = scratch / "Geothite.app"
    resources = app / "Contents/Resources"
    executable = app / "Contents/MacOS/Geothite"
    executable.parent.mkdir(parents=True)
    resources.mkdir()
    shutil.copy2(binary_dir / "crystal-bevy", executable)
    executable.chmod(0o755)
    shutil.copy2(pack, resources / "core-modular.crystalpack")
    copy_artwork(art, resources / "assets/open-models")
    shutil.copy2(ROOT / "LICENSE", resources / "LICENSE")
    metadata = plist(version)
    load_commands = subprocess.check_output(["otool", "-l", executable], text=True)
    minimum = re.search(r"\bminos\s+(\d+(?:\.\d+)+)", load_commands)
    if minimum is None:
        minimum = re.search(r"LC_VERSION_MIN_MACOSX\s+cmdsize\s+\d+\s+version\s+(\d+(?:\.\d+)+)", load_commands)
    if minimum is None:
        raise RuntimeError("Cannot determine the executable's minimum macOS version.")
    metadata["LSMinimumSystemVersion"] = minimum[1]
    (app / "Contents/Info.plist").write_bytes(plistlib.dumps(metadata))
    (app / "Contents/PkgInfo").write_bytes(b"APPL????")
    png = scratch / "geothite.png"
    icon(png)
    iconset = scratch / "Geothite.iconset"
    iconset.mkdir()
    for size in [16, 32, 128, 256, 512]:
        for scale in [1, 2]:
            suffix = "@2x" if scale == 2 else ""
            run("sips", "-z", size * scale, size * scale, png, "--out", iconset / f"icon_{size}x{size}{suffix}.png", stdout=subprocess.DEVNULL)
    run("iconutil", "-c", "icns", iconset, "-o", resources / "Geothite.icns")
    # Reject Homebrew/build-machine dylibs rather than shipping a broken app.
    dependencies = subprocess.check_output(["otool", "-L", executable], text=True)
    for line in dependencies.splitlines()[1:]:
        library = line.strip().split(" (", 1)[0]
        if not library.startswith(("/System/Library/", "/usr/lib/")):
            raise RuntimeError(f"Non-system dynamic library needs bundling: {library}")
    sign(app, identity)
    return app, executable


def stage_linux(scratch, binary_dir, pack, art):
    directory = scratch / "Geothite"
    directory.mkdir()
    executable = directory / "Geothite"
    shutil.copy2(binary_dir / "crystal-bevy", executable)
    executable.chmod(0o755)
    shutil.copy2(pack, directory / "core-modular.crystalpack")
    copy_artwork(art, directory / "assets/open-models")
    shutil.copy2(ROOT / "LICENSE", directory / "LICENSE")
    icon(directory / "geothite.png", 256)
    (directory / "geothite.desktop").write_text(
        "[Desktop Entry]\nType=Application\nName=Geothite\nExec=Geothite\nIcon=geothite\nTerminal=false\nCategories=Game;RolePlaying;\n")
    (directory / "README.txt").write_text(
        "Extract this directory and run ./Geothite. Requires a compatible Linux graphics session, "
        "Vulkan/OpenGL drivers, ALSA, udev and X11 libraries. Saves use $XDG_DATA_HOME/geothite/saves "
        "or ~/.local/share/geothite/saves. This archive is built for the host architecture and distro ABI.\n")
    return directory, executable


def replace_directory(source, destination):
    """Stage beside the destination, keeping the old app on copy failure."""
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".geothite-", dir=destination.parent) as temporary:
        pending = Path(temporary) / destination.name
        shutil.copytree(source, pending)
        previous = Path(temporary) / "previous"
        if destination.exists():
            destination.rename(previous)
        try:
            pending.rename(destination)
        except OSError:
            if previous.exists():
                previous.rename(destination)
            raise


def installed_path(host):
    if host == "macos":
        return Path.home() / "Applications/Geothite.app"
    return Path.home() / ".local/opt/geothite"


def launch(host):
    installed = installed_path(host)
    if not installed.exists():
        raise RuntimeError("No installed desktop copy. Run just desktop first.")
    if host == "macos":
        log = Path.home() / "Library/Logs/Geothite/game.log"
        log.parent.mkdir(parents=True, exist_ok=True)
        run("open", "--stdout", log, "--stderr", log, installed)
        print(f"Launch log: {log}")
    else:
        log = Path(os.environ.get("XDG_STATE_HOME", Path.home() / ".local/state")) / "geothite"
        log.mkdir(parents=True, exist_ok=True)
        with (log / "game.log").open("ab") as output:
            subprocess.Popen([str(installed / "Geothite")], start_new_session=True,
                             stdin=subprocess.DEVNULL, stdout=output, stderr=subprocess.STDOUT)


def package_macos(app, output, version, identity, notary_profile, scratch):
    image_root = scratch / "image"
    image_root.mkdir()
    shutil.copytree(app, image_root / app.name)
    (image_root / "Applications").symlink_to("/Applications")
    name = f"Geothite-{version}-macos-{platform.machine()}.dmg"
    pending = scratch / name
    run("hdiutil", "create", "-quiet", "-volname", "Geothite", "-srcfolder", image_root,
        "-format", "UDZO", "-ov", pending)
    if notary_profile:
        if identity == "-":
            raise RuntimeError("Notarization requires GEOTHITE_SIGN_IDENTITY (Developer ID Application).")
        run("codesign", "--sign", identity, "--timestamp", pending)
        run("xcrun", "notarytool", "submit", pending, "--keychain-profile", notary_profile, "--wait")
        run("xcrun", "stapler", "staple", pending)
        run("xcrun", "stapler", "validate", pending)
    result = output / name
    pending.replace(result)
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["build", "install", "package", "run", "check"])
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/core-modular.crystalpack")
    parser.add_argument("--target-dir", type=Path, default=Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target/desktop-build")))
    parser.add_argument("--profile", choices=["dev", "wasm-dev", "release"], default=os.environ.get("GEOTHITE_DESKTOP_PROFILE", "release"))
    parser.add_argument("--jobs", default=os.environ.get("CARGO_BUILD_JOBS", "2"))
    parser.add_argument("--skip-build", action="store_true", help="Repackage the last successful build without invoking Cargo")
    parser.add_argument("--notarize", action="store_true", help="Use configured Developer ID and notarytool keychain profile")
    args = parser.parse_args(argv)
    host = {"Darwin": "macos", "Linux": "linux"}.get(platform.system())
    if host is None:
        parser.error("On Windows use tools/build-desktop.ps1 (or just desktop).")
    if args.action == "run":
        launch(host)
        return
    pack = args.pack.resolve()
    if not pack.is_file() or pack.suffix != ".crystalpack":
        parser.error(f"Missing local content pack {pack}. Run just desktop-content first.")
    identity = os.environ.get("GEOTHITE_SIGN_IDENTITY", "-")
    notary_profile = os.environ.get("GEOTHITE_NOTARY_PROFILE") if args.notarize else None
    if args.notarize and (host != "macos" or not notary_profile or identity == "-"):
        parser.error("Set GEOTHITE_SIGN_IDENTITY and GEOTHITE_NOTARY_PROFILE on macOS before notarizing.")
    target = args.target_dir.resolve()
    binary_dir = target / ("debug" if args.profile == "dev" else args.profile)
    started = time.monotonic()
    if not args.skip_build and args.action != "check":
        build(target, args.profile, args.jobs)
    DIST.mkdir(parents=True, exist_ok=True)
    art = Path(os.environ.get("CRYSTAL_OPEN_MODEL_ROOT", ROOT / "assets/open-models")).resolve()
    with tempfile.TemporaryDirectory(prefix=".stage-", dir=DIST) as temporary:
        scratch = Path(temporary)
        validate(binary_dir, pack, art, scratch)
        if args.action in {"build", "check"}:
            print(f"Desktop build and content verified ({time.monotonic() - started:.1f}s).")
            return
        version = re.search(r'^version = "([^"]+)"', (ROOT / "crates/crystal-bevy/Cargo.toml").read_text(), re.M)[1]
        if host == "macos":
            staged, executable = stage_macos(scratch, binary_dir, pack, art, version, identity)
        else:
            staged, executable = stage_linux(scratch, binary_dir, pack, art)
        # Run from outside the checkout with overrides removed. This must load
        # the packaged pack/art and select a save outside the app without a GUI.
        clean_env = {key: value for key, value in os.environ.items() if key != "CRYSTAL_OPEN_MODEL_ROOT"}
        run(executable, "--check-installation", cwd=scratch, env=clean_env)
        destination = DIST / staged.name
        replace_directory(staged, destination)
        if args.action == "install":
            installed = installed_path(host)
            replace_directory(staged, installed)
            if host == "linux":
                entry = Path.home() / ".local/share/applications/geothite.desktop"
                entry.parent.mkdir(parents=True, exist_ok=True)
                executable_path = str(installed / "Geothite").replace("\\", "\\\\").replace('"', '\\"').replace("%", "%%")
                entry.write_text((staged / "geothite.desktop").read_text().replace("Exec=Geothite", f'Exec="{executable_path}"').replace("Icon=geothite", f"Icon={installed / 'geothite.png'}"))
            print(f"Installed {installed}")
        else:
            if host == "macos":
                archive = package_macos(staged, DIST, version, identity, notary_profile, scratch)
            else:
                archive = DIST / f"Geothite-{version}-linux-{platform.machine()}.tar.gz"
                pending = scratch / archive.name
                with tarfile.open(pending, "w:gz", compresslevel=1) as stream:
                    stream.add(staged, arcname="Geothite")
                pending.replace(archive)
            checksum = hashlib.sha256()
            with archive.open("rb") as stream:
                for block in iter(lambda: stream.read(1024 * 1024), b""):
                    checksum.update(block)
            digest = checksum.hexdigest()
            archive.with_suffix(archive.suffix + ".sha256").write_text(f"{digest}  {archive.name}\n")
            print(f"Created {archive}")
            if host == "macos" and not notary_profile:
                print("Local package; not notarized. Use just desktop-release with Developer ID credentials for public distribution.")
    print(f"Finished in {time.monotonic() - started:.1f}s.")


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, subprocess.CalledProcessError, OSError) as error:
        sys.exit(f"Desktop packaging failed: {error}")
