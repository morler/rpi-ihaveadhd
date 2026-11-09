import hashlib, io, json, pathlib, re, sys, tarfile

path = sys.argv[1] if len(sys.argv) > 1 else "dist/rpi-extension.json"
path = pathlib.Path(path)
m = json.loads(path.read_text())
assert m["schema_version"] == 1
assert re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", m["version"]), m["version"]
assert re.fullmatch(r"[0-9a-f]{40}", m["source_commit"])
assert re.fullmatch(r"https://github\.com/[^/]+/[^/]+", m["repository"]), m["repository"]
assert re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", m["sdk"]["version"]), m["sdk"]["version"]
assert m["sdk"]["plugin_abi"] >= 1
assert re.fullmatch(r"[A-Za-z0-9_-]+", m["name"]), m["name"]
assert m["tag"], "empty tag"
assert len(m["targets"]) >= 1
checked = 0
for triple, t in m["targets"].items():
    assert t["format"] in ("zip", "tar.gz"), t["format"]
    assert "/releases/download/" in t["url"] and t["url"].startswith("https://github.com/")
    assert t["size"] >= 1
    assert re.fullmatch(r"[0-9a-f]{64}", t["sha256"])
    assert re.fullmatch(r"[A-Za-z0-9_.-]+\.(dll|so|dylib)", t["library"]["path"]), t["library"]["path"]
    assert re.fullmatch(r"[0-9a-f]{64}", t["library"]["sha256"])

    # Verify the bytes the URL serves, when the archive was staged next to the manifest.
    archive = path.parent / t["url"].rsplit("/", 1)[-1]
    if not archive.exists():
        continue
    data = archive.read_bytes()
    assert len(data) == t["size"], f"{triple}: size {len(data)} != {t['size']}"
    assert hashlib.sha256(data).hexdigest() == t["sha256"], f"{triple}: archive sha256 mismatch"
    with tarfile.open(fileobj=io.BytesIO(data)) as tar:
        members = [x for x in tar.getmembers() if x.isfile()]
        assert len(members) == 1, f"{triple}: archive holds {len(members)} files"
        assert members[0].name == t["library"]["path"], f"{triple}: library path {members[0].name}"
        lib = tar.extractfile(members[0]).read()
    assert hashlib.sha256(lib).hexdigest() == t["library"]["sha256"], f"{triple}: library sha256 mismatch"
    checked += 1
print("schema OK:", ", ".join(sorted(m["targets"])), f"| verified bytes for {checked} archive(s)")
