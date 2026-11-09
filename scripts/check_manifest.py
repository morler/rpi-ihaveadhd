import json, re, pathlib, sys

path = sys.argv[1] if len(sys.argv) > 1 else "dist/rpi-extension.json"
m = json.loads(pathlib.Path(path).read_text())
assert m["schema_version"] == 1
assert re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", m["version"]), m["version"]
assert re.fullmatch(r"[0-9a-f]{40}", m["source_commit"])
assert re.fullmatch(r"https://github\.com/[^/]+/[^/]+", m["repository"]), m["repository"]
assert re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", m["sdk"]["version"]), m["sdk"]["version"]
assert m["sdk"]["plugin_abi"] >= 1
assert re.fullmatch(r"[A-Za-z0-9_-]+", m["name"]), m["name"]
assert m["tag"], "empty tag"
assert len(m["targets"]) >= 1
for triple, t in m["targets"].items():
    assert t["format"] in ("zip", "tar.gz"), t["format"]
    assert "/releases/download/" in t["url"] and t["url"].startswith("https://github.com/")
    assert t["size"] >= 1
    assert re.fullmatch(r"[0-9a-f]{64}", t["sha256"])
    assert re.fullmatch(r"[A-Za-z0-9_.-]+\.(dll|so|dylib)", t["library"]["path"]), t["library"]["path"]
    assert re.fullmatch(r"[0-9a-f]{64}", t["library"]["sha256"])
print("schema OK:", ", ".join(sorted(m["targets"])))
