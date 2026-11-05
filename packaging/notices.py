#!/usr/bin/env python3
"""Collect locked native dependency licenses and public package attribution."""
import json
from pathlib import Path
import re
import shutil
import sys

ROOT = Path(__file__).resolve().parents[1]
LEGAL = ("LICENSE", "COPYING", "COPYRIGHT", "NOTICE")


def collect(metadata, destination):
    destination.mkdir(parents=True, exist_ok=True)
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    reachable, pending = set(), [metadata["resolve"]["root"]]
    while pending:
        identifier = pending.pop()
        if identifier in reachable:
            continue
        reachable.add(identifier)
        pending.extend(dependency["pkg"] for dependency in nodes[identifier]["deps"])
    packages = sorted((item for item in metadata["packages"] if item["id"] in reachable), key=lambda item: (item["name"], item["version"]))
    records, sections = [], []
    for package in packages:
        base = Path(package["manifest_path"]).parent
        name = package["name"] + " " + package["version"]
        expression = package.get("license")
        legal_files = []
        for path in base.rglob("*"):
            if path.is_file() and path.name.upper().startswith(LEGAL) and not path.is_symlink():
                # License/notice texts only; no source bodies or developer paths.
                if path.stat().st_size <= 4 * 1024 * 1024:
                    legal_files.append(path)
        declared = package.get("license_file")
        if declared:
            path = (base / declared).resolve()
            path.relative_to(base.resolve())
            if path.is_file() and path not in legal_files:
                legal_files.append(path)
        header = f"\n{'=' * 72}\n{name}\nDeclared license: {expression or 'see license file'}\n"
        authors = package.get("authors", [])
        if authors:
            header += "Package authors: " + "; ".join(authors) + "\n"
        if package.get("repository"):
            header += "Upstream: " + package["repository"] + "\n"
        texts = []
        if legal_files:
            for path in sorted(legal_files):
                relative = path.relative_to(base)
                texts.append(f"\n--- {relative} ---\n" + path.read_text(errors="replace"))
        else:
            # Some published workspace crates omit their shared license files.
            # Preserve their declared expression and authors, and include the
            # applicable standard terms explicitly labelled as standard text.
            identifiers = re.findall(r"[A-Za-z0-9][A-Za-z0-9.+-]*", expression or "")
            selected = next((item for item in ("MIT", "Apache-2.0", "MIT-0", "CC0-1.0") if item in identifiers), None)
            if selected is None:
                raise ValueError(f"no packaged license text or supported standard fallback: {name}: {expression}")
            fallback = ROOT / "packaging/licenses" / (selected + ".txt")
            texts.append(f"\nUpstream crate archive omits its shared license file.\nStandard {selected} license terms (SPDX license-list-data v3.26.0):\n" + fallback.read_text())
        records.append({"name": package["name"], "version": package["version"], "license": expression,
                        "authors": authors, "repository": package.get("repository"),
                        "packaged_notice_files": [str(path.relative_to(base)) for path in sorted(legal_files)],
                        "uses_standard_license_fallback": not bool(legal_files)})
        sections.append(header + "\n".join(texts))
    (destination / "DEPENDENCIES.json").write_text(json.dumps(records, indent=2) + "\n")
    (destination / "THIRD-PARTY-NOTICES.txt").write_text(
        "Locked native-target dependency notices. Alternative licenses are preserved as declared.\n"
        "Source notice files are reproduced when present. Standard-license fallbacks are labelled;\n"
        "package author attributions are recorded separately and do not replace upstream notices.\n"
        + "\n".join(sections))
    shutil.copy2(ROOT / "LICENSE", destination / "DUROP-LICENSE")
    shutil.copy2(ROOT / "packaging/licenses/objc2-0.6.4-LICENSE.md", destination / "OBJC2-SDK-LICENSE-NOTE.md")
    shutil.copy2(ROOT / "packaging/licenses/SOURCES.json", destination / "LICENSE-TEXT-SOURCES.json")
    return len(records)


if __name__ == "__main__":
    try:
        print(f"Collected notices for {collect(json.load(sys.stdin), Path(sys.argv[1]))} locked native-target packages")
    except (OSError, ValueError, KeyError, IndexError) as error:
        print(f"notice collection failed: {error}", file=sys.stderr)
        sys.exit(1)
