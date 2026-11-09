#!/usr/bin/env python3
"""Build rpi-extension.json from the staged archives in a directory."""
import hashlib
import json
import pathlib
import sys
import tarfile
import zipfile

DIST = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "dist")
NAME = "rpi-ihaveadhd"
VERSION = sys.argv[2]
REPO = sys.argv[3]
TAG = sys.argv[4]
COMMIT = sys.argv[5]
SDK_VERSION = sys.argv[6]
SDK_ABI = int(sys.argv[7])


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def read_library(archive):
    """Return (path, bytes) of the single cdylib member inside the archive."""
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as z:
            members = [m for m in z.namelist() if m.endswith((".so", ".dylib", ".dll"))]
            if len(members) != 1:
                raise SystemExit(f"{archive}: expected 1 library, got {members}")
            return members[0], z.read(members[0])
    with tarfile.open(archive) as t:
        members = [m for m in t.getmembers() if m.name.endswith((".so", ".dylib", ".dll"))]
        if len(members) != 1:
            raise SystemExit(f"{archive}: expected 1 library, got {[m.name for m in members]}")
        return members[0].name, t.extractfile(members[0]).read()


targets = {}
for archive in sorted(DIST.iterdir()):
    if archive.suffix not in (".gz", ".zip"):
        continue
    # <name>-<version>-<target>.<ext>
    target = archive.name[: -len(".tar.gz") if archive.name.endswith(".tar.gz") else -len(".zip")]
    target = target[len(f"{NAME}-{VERSION}-") :]
    library, payload = read_library(archive)
    targets[target] = {
        "url": f"{REPO}/releases/download/{TAG}/{archive.name}",
        "format": "zip" if archive.suffix == ".zip" else "tar.gz",
        "size": archive.stat().st_size,
        "sha256": sha256(archive.read_bytes()),
        "library": {"path": library, "sha256": sha256(payload)},
    }

if not targets:
    raise SystemExit(f"no archives staged in {DIST}")

manifest = {
    "schema_version": 1,
    "name": NAME,
    "version": VERSION,
    "repository": REPO,
    "tag": TAG,
    "source_commit": COMMIT,
    "sdk": {"version": SDK_VERSION, "plugin_abi": SDK_ABI},
    "targets": targets,
}
out = DIST / "rpi-extension.json"
out.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
print(out)
print("targets:", ", ".join(sorted(targets)))
