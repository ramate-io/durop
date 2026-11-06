#!/usr/bin/env python3
"""Build one native Apple Silicon app; no release publishing or developer setup."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import plistlib
import re
import shutil
import subprocess
import struct
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
TARGET = "aarch64-apple-darwin"
MINIMUM = "14.0"
BUNDLE_ID = "io.ramate.durop"
FORBIDDEN = ("/nix/", "/opt/homebrew", "/usr/local", "/Cellar/")


def run(args, **kwargs):
    return subprocess.run([str(x) for x in args], check=True, **kwargs)


def capture(args, **kwargs):
    return subprocess.check_output([str(x) for x in args], text=True, **kwargs).strip()


def source_fingerprint():
    digest=hashlib.sha256()
    files=[ROOT/name for name in ("Cargo.toml","Cargo.lock","rust-toolchain.toml")]
    for source in (ROOT / "src", ROOT / "crates"):
        files.extend(path for path in source.rglob("*") if path.is_file())
    for path in sorted(files):
        name=str(path.relative_to(ROOT)).encode()
        data=path.read_bytes()
        digest.update(len(name).to_bytes(8,"big"));digest.update(name)
        digest.update(len(data).to_bytes(8,"big"));digest.update(data)
    return digest.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cargo", default=os.environ.get("DUROP_NATIVE_CARGO", "cargo"), help="native Cargo executable or rustup shim")
    parser.add_argument("--output", type=Path, default=ROOT / "dist")
    args = parser.parse_args()
    args.output=args.output.resolve()
    if platform.system() != "Darwin":
        parser.error("native macOS with Apple SDK tools is required")
    if os.environ.get("IN_NIX_SHELL"):
        parser.error("run packaging outside nix develop")
    cargo = shutil.which(args.cargo)
    if not cargo:
        parser.error("native Cargo not found; install the pinned rust-toolchain.toml channel")
    cargo = Path(cargo).absolute()
    if any(part in str(cargo.resolve()) for part in FORBIDDEN):
        parser.error("Cargo must be native, outside Nix/Homebrew")
    channel = re.search(r'^channel\s*=\s*"([^"]+)"', (ROOT / "rust-toolchain.toml").read_text(), re.M).group(1)
    package = (ROOT / "Cargo.toml").read_text().split("[package]", 1)[1].split("[", 1)[0]
    version = re.search(r'^version\s*=\s*"([^"]+)"', package, re.M).group(1)
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        parser.error("POC bundle requires a numeric major.minor.patch package version")
    sdk = capture(["/usr/bin/xcrun", "--sdk", "macosx", "--show-sdk-path"])
    env = {key: os.environ[key] for key in ("HOME", "USER", "LOGNAME", "TMPDIR", "LANG", "CARGO_HOME", "RUSTUP_HOME") if key in os.environ}
    env.update(PATH=f"{cargo.parent}:/usr/bin:/bin:/usr/sbin:/sbin", SDKROOT=sdk,
               MACOSX_DEPLOYMENT_TARGET=MINIMUM, CARGO_TARGET_DIR=str(ROOT / "target/native-package"))
    rustc = capture([cargo, "-V"], env=env, cwd=ROOT)
    if not rustc.startswith(f"cargo {channel} "):
        parser.error(f"Cargo channel must match {channel}; found {rustc}")
    compiler = capture(["rustc", "-vV"], env=env, cwd=ROOT)
    sysroot = capture(["rustc", "--print", "sysroot"], env=env, cwd=ROOT)
    if any(part in sysroot for part in FORBIDDEN):
        parser.error("native Rust sysroot required")
    if f"release: {channel}\n" not in compiler + "\n":
        parser.error("rustc version does not match rust-toolchain.toml")
    fingerprint=source_fingerprint()
    run([cargo, "build", "--package", "durop", "--bin", "durop", "--release", "--locked", "--target", TARGET], cwd=ROOT, env=env)
    if source_fingerprint()!=fingerprint:
        raise RuntimeError("Rust source inputs changed during the build; retry from a frozen checkout")
    binary = ROOT / "target/native-package" / TARGET / "release/durop"
    if not binary.is_file():
        parser.error(f"native release executable missing: {binary}")
    metadata = json.loads(capture([cargo, "metadata", "--locked", "--format-version", "1", "--filter-platform", TARGET], cwd=ROOT, env=env))
    args.output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".durop-package-", dir=args.output) as temporary:
        staging = Path(temporary)
        app = staging / "Durop.app"
        contents = app / "Contents"
        executable = contents / "MacOS/durop"
        resources = contents / "Resources"
        executable.parent.mkdir(parents=True)
        resources.mkdir()
        shutil.copy2(binary, executable)
        shutil.copytree(ROOT / "assets", resources / "assets")
        shutil.copytree(ROOT / "sample-packs/podcast-pack", resources / "sample-packs/podcast-pack")
        shutil.copytree(ROOT / "sample-packs/reactions", resources / "sample-packs/reactions")
        if (ROOT / "fixtures/media").is_dir():
            shutil.copytree(ROOT / "fixtures/media", resources / "sample-packs/media-proof")
        # ICNS ic08 stores the existing 256px PNG verbatim. Native iconutil
        # decoding is covered separately; no pixel editing or borrowed artwork.
        icon=(ROOT / "assets/overlay-proof.png").read_bytes()
        if len(icon)<24 or icon[:8]!=b"\x89PNG\r\n\x1a\n" or struct.unpack(">II",icon[16:24])!=(256,256):
            raise RuntimeError("ICNS source must remain the authored 256px PNG")
        (resources / "Durop.icns").write_bytes(struct.pack(">4sI4sI",b"icns",16+len(icon),b"ic08",8+len(icon))+icon)
        plist = {
            "CFBundleIdentifier": BUNDLE_ID, "CFBundleName": "Durop", "CFBundleDisplayName": "Durop",
            "CFBundleExecutable": "durop", "CFBundlePackageType": "APPL", "CFBundleIconFile": "Durop.icns",
            "CFBundleShortVersionString": version, "CFBundleVersion": version,
            "LSMinimumSystemVersion": MINIMUM, "NSHighResolutionCapable": True,
            "NSLocalNetworkUsageDescription": "Durop uses an explicitly selected Tailscale peer for authenticated remote media control.",
            "NSPrincipalClass": "NSApplication", "LSApplicationCategoryType": "public.app-category.entertainment",
        }
        with (contents / "Info.plist").open("wb") as stream:
            plistlib.dump(plist, stream)
        revision = capture(["/usr/bin/git", "rev-parse", "HEAD"], cwd=ROOT)
        info = {"version": version, "source_revision": revision,
                "source_dirty": bool(capture(["/usr/bin/git", "status", "--porcelain"], cwd=ROOT)),
                "native_build_executable_sha256": hashlib.sha256(executable.read_bytes()).hexdigest(),
                "source_fingerprint_sha256":fingerprint,
                "cargo_lock_sha256": hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest(),
                "rust_channel": channel,
                "sdk_version": capture(["/usr/bin/xcrun", "--sdk", "macosx", "--show-sdk-version"]),
                "minimum_macos": MINIMUM, "architecture": "arm64", "signature": "ad-hoc", "notarized": False,
                "git_refresh": "optional external Git; not bundled or required for local/cached playback"}
        (resources / "BUILD-INFO.json").write_text(json.dumps(info, indent=2) + "\n")
        run([sys.executable, ROOT / "packaging/notices.py", resources / "notices"], input=json.dumps(metadata), text=True)
        rust_legal=Path(sysroot) / "share/doc/rust"
        if not (rust_legal / "COPYRIGHT-library.html").is_file():
            raise RuntimeError("native Rust distribution is missing its standard-library copyright notice")
        shutil.copy2(rust_legal / "COPYRIGHT-library.html",resources / "notices/RUST-COPYRIGHT-library.html")
        if (rust_legal / "licenses").is_dir():
            shutil.copytree(rust_legal / "licenses",resources / "notices/rust-licenses")
        for document in ("README.md","CONTRIBUTING.md","LICENSE"):
            shutil.copy2(ROOT/document,resources/document)
        (resources/"docs").mkdir()
        for source in (ROOT/"docs").glob("*.md"):
            shutil.copy2(source,resources/"docs"/source.name)
        shutil.copy2(ROOT/"fixtures/media/README.md",resources/"notices/SAMPLE-ASSETS.md")
        run(["/usr/bin/codesign", "--force", "--sign", "-", "--timestamp=none", app])
        validation=capture([sys.executable, ROOT / "packaging/macos/validate.py", app])
        json.loads(validation)
        # An arbitrary working directory and clean environment expose accidental
        # Cargo/asset-root/developer-path requirements without launching a GUI.
        smoke_env = {"PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "HOME": str(staging / "smoke-home")}
        Path(smoke_env["HOME"]).mkdir()
        smoke=capture([executable, "--self-check"], cwd="/private/tmp", env=smoke_env, timeout=30)
        if f"resources={resources}" not in smoke or not re.search(r"clips=[1-9][0-9]*",smoke):
            raise RuntimeError("self-check did not confirm the bundled resource root and nonempty sample pack")
        if any(Path(smoke_env["HOME"]).iterdir()):
            raise RuntimeError("read-only self-check unexpectedly wrote to its empty HOME")
        print(smoke)
        run(["/usr/bin/codesign", "--verify", "--strict", "--deep", app])
        archive_name = f"Durop-{version}-macos-arm64.zip"
        archive = staging / archive_name
        run(["/usr/bin/ditto", "-c", "-k", "--keepParent", app, archive])
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        for item in (app, archive):
            destination = args.output / item.name
            if destination.exists():
                raise RuntimeError(f"output already exists; choose a new --output directory: {destination}")
            shutil.move(str(item), str(destination))
        (args.output / (archive_name + ".sha256")).write_text(f"{digest}  {archive_name}\n")
        (args.output / "BUILD-INFO.json").write_text(json.dumps(info, indent=2) + "\n")
        (args.output / "VALIDATION.json").write_text(validation + "\n")
        print(args.output / archive_name)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"packaging failed: {error}", file=sys.stderr)
        sys.exit(1)
