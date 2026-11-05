#!/usr/bin/env python3
"""Audit every Mach-O and its loader closure inside one standalone .app."""
import argparse
import json
import hashlib
from pathlib import Path
import plistlib
import posixpath
import re
import subprocess
import sys

SYSTEM = ("/System/Library/", "/usr/lib/")
FORBIDDEN = ("/nix/", "/opt/homebrew", "/usr/local", "/Cellar/", "/Users/", "/private/tmp/", "/tmp/")
MACH_MAGIC = {b"\xfe\xed\xfa\xce", b"\xce\xfa\xed\xfe", b"\xfe\xed\xfa\xcf", b"\xcf\xfa\xed\xfe", b"\xca\xfe\xba\xbe", b"\xbe\xba\xfe\xca", b"\xca\xfe\xba\xbf", b"\xbf\xba\xfe\xca"}


def version(value):
    if not re.fullmatch(r"\d+(\.\d+){0,2}", value):
        raise ValueError(f"invalid macOS version: {value}")
    result = tuple(map(int, value.split(".")))
    return result + (0,) * (3 - len(result))


def output(args):
    return subprocess.check_output([str(x) for x in args], text=True, stderr=subprocess.STDOUT)


def load_commands(text):
    rpaths, minimums, ids = [], [], []
    command = None
    for raw in text.splitlines():
        line = raw.strip()
        if line.startswith("cmd "):
            command = line[4:]
            if command == "LC_DYLD_ENVIRONMENT":
                raise ValueError("Mach-O embeds a dynamic-loader environment override")
        elif command == "LC_RPATH" and line.startswith("path "):
            rpaths.append(line[5:].rsplit(" (offset ", 1)[0])
        elif command == "LC_ID_DYLIB" and line.startswith("name "):
            ids.append(line[5:].rsplit(" (offset ", 1)[0])
        elif command == "LC_BUILD_VERSION" and line.startswith("minos "):
            minimums.append(line.split()[1])
        elif command == "LC_VERSION_MIN_MACOSX" and line.startswith("version "):
            minimums.append(line.split()[1])
    return rpaths, minimums, ids


def contained(root, target):
    resolved = target.resolve()
    try:
        resolved.relative_to(root)
    except ValueError:
        raise ValueError(f"loader path escapes application: {target.name}")
    return resolved


def is_system(path):
    return path.startswith(SYSTEM) and posixpath.normpath(path).startswith(SYSTEM)


def expand(path, loader, executable, root):
    for prefix, base in (("@loader_path", loader.parent), ("@executable_path", executable.parent)):
        if path == prefix or path.startswith(prefix + "/"):
            return contained(root, base / path[len(prefix):].lstrip("/"))
    if is_system(path):
        return Path(path)
    raise ValueError(f"unsupported or external loader path: {path}")


def audit_machos(app, minimum):
    root = app.resolve()
    executable = root / "Contents/MacOS/durop"
    binaries = {}
    for path in root.rglob("*"):
        if path.is_symlink():
            contained(root, path)
        if not path.is_file():
            continue
        with path.open("rb") as stream:
            magic = stream.read(4)
        if magic not in MACH_MAGIC:
            continue
        path = contained(root, path)
        architectures = output(["/usr/bin/lipo", "-archs", path]).strip().split()
        if architectures != ["arm64"]:
            raise ValueError(f"POC requires arm64-only Mach-O: {path.name}: {architectures}")
        commands = load_commands(output(["/usr/bin/otool", "-l", path]))
        rpaths, minimums, ids = commands
        if not minimums or any(version(value) > version(minimum) for value in minimums):
            raise ValueError(f"Mach-O minimum exceeds declared macOS {minimum}: {path.name}: {minimums}")
        if path == executable and any(version(value) != version(minimum) for value in minimums):
            raise ValueError("main executable deployment target differs from Info.plist")
        for value in rpaths + ids:
            if any(bad in value for bad in FORBIDDEN):
                raise ValueError(f"developer path embedded in loader metadata: {path.name}")
            if value in ids:
                if value.startswith("@rpath/"):
                    if ".." in Path(value).parts:
                        raise ValueError(f"bundled dylib install name contains traversal: {path.name}")
                else:
                    expand(value,path,executable,root)
            else:
                expand(value, path, executable, root)
        dependencies = []
        for line in output(["/usr/bin/otool", "-L", path]).splitlines()[1:]:
            value = line.strip().split(" (compatibility version ", 1)[0]
            if value and value not in ids:
                dependencies.append(value)
        binaries[path] = (rpaths, dependencies)
    if executable not in binaries:
        raise ValueError("main executable is missing or not Mach-O")
    main_rpaths = binaries[executable][0]
    dependency_count = 0
    for path, (rpaths, dependencies) in binaries.items():
        for dependency in dependencies:
            dependency_count += 1
            if any(bad in dependency for bad in FORBIDDEN):
                raise ValueError(f"external developer dependency: {path.name}: {dependency}")
            if is_system(dependency):
                continue  # System libraries may exist only in the dyld shared cache.
            if dependency.startswith("@rpath/"):
                candidates = []
                for rpath, loader in [(x, path) for x in rpaths] + [(x, executable) for x in main_rpaths]:
                    base = expand(rpath, loader, executable, root)
                    if is_system(str(base)):
                        continue
                    candidates.append(contained(root, base / dependency[len("@rpath/"):]))
                resolved = next((candidate for candidate in candidates if candidate in binaries), None)
                if resolved is None:
                    raise ValueError(f"unresolved bundled @rpath dependency: {path.name}: {dependency}")
            else:
                resolved = expand(dependency, path, executable, root)
                if resolved not in binaries:
                    raise ValueError(f"bundled dependency missing or not Mach-O: {dependency}")
    return {"architecture": "arm64", "minimum_macos": minimum,
            "mach_o_files": sorted(str(path.relative_to(root)) for path in binaries),
            "dependency_count": dependency_count, "external_non_system_dependencies": 0,
            "signed_executable_sha256":hashlib.sha256(executable.read_bytes()).hexdigest()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("app", type=Path)
    args = parser.parse_args()
    with (args.app / "Contents/Info.plist").open("rb") as stream:
        plist = plistlib.load(stream)
    if plist.get("CFBundleIdentifier") != "io.ramate.durop" or plist.get("CFBundleExecutable") != "durop":
        raise ValueError("unexpected application identity/executable")
    icon = args.app / "Contents/Resources" / plist["CFBundleIconFile"]
    if icon.read_bytes()[:4] != b"icns":
        raise ValueError("application icon missing or invalid")
    report = audit_machos(args.app, plist["LSMinimumSystemVersion"])
    output(["/usr/bin/codesign", "--verify", "--strict", "--deep", args.app])
    signing = output(["/usr/bin/codesign", "-d", "-vv", args.app])
    if "Signature=adhoc" not in signing:
        raise ValueError("POC package must accurately identify its expected ad-hoc signature")
    report.update(bundle_id=plist["CFBundleIdentifier"], signature="ad-hoc", notarized=False)
    print(json.dumps(report, indent=2, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(f"bundle validation failed: {error}", file=sys.stderr)
        sys.exit(1)
