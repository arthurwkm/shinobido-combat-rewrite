"""Kagetora model: PS2-native RenderWare geometry (VIF packets) -> vertex lists.

    python tools/model.py            self-check KAGETORA_BODY against the RW headers
"""

import math
import struct

from rws import MODEL, chunks, frame_ids, skeleton

# VIF UNPACK cmd -> (name, bytes per element)
UNPACK = {0x68: ("pos", 12), 0x64: ("uv", 8), 0x6E: ("v4b", 4), 0x6A: ("v3b", 3)}


def geometries(d):
    """[(geometry struct dict, mesh list [(numIndices, material)], native VIF bytes, skin ext bytes)]"""
    clump = next(p for t, _, _, p in chunks(d) if t == 0x10)
    gl = next((s, p) for t, s, _, p in chunks(d, clump + 12) if t == 0x1A)
    out = []
    for t, s, _, p in chunks(d, gl[1] + 12, gl[1] + 12 + gl[0]):
        if t != 0x0F:
            continue
        g = {}
        ext = {}
        for t2, s2, _, p2 in chunks(d, p + 12, p + 12 + s):
            if t2 == 1:
                flags, g["tris"], g["verts"], g["morphs"] = struct.unpack_from("<4I", d, p2 + 12)
            if t2 == 3:
                ext = {t3: (p3 + 12, s3) for t3, s3, _, p3 in chunks(d, p2 + 12, p2 + 12 + s2)}
        q, n = ext[0x50E]
        _, nm, total = struct.unpack_from("<3I", d, q)
        meshes = [struct.unpack_from("<2I", d, q + 12 + 8 * i) for i in range(nm)]
        assert sum(m[0] for m in meshes) == total
        q, n = ext[0x510]
        assert struct.unpack_from("<I", d, q + 12)[0] == 4  # PS2 platform id
        out.append((g, meshes, d[q + 16 : q + 12 + struct.unpack_from("<I", d, q + 4)[0]], d[ext[0x116][0] : sum(ext[0x116])]))
    return out


def height(d):
    """Bind-pose vertical bounds: native vertices through their atomic frame and its ancestors."""
    clump = next(p for t, _, _, p in chunks(d) if t == 0x10)
    fl = next(p for t, _, _, p in chunks(d, clump + 12) if t == 0x0E)
    count = struct.unpack_from("<I", d, fl + 24)[0]
    frames = [struct.unpack_from("<12f2i", d, fl + 28 + 56 * i) for i in range(count)]
    geometry = geometries(d)
    ys = []
    for t, size, _, p in chunks(d, clump + 12, clump + 12 + struct.unpack_from("<I", d, clump + 4)[0]):
        if t != 0x14:
            continue
        q = next(q for tt, _, _, q in chunks(d, p + 12, p + 12 + size) if tt == 1)
        frame, gi = struct.unpack_from("<2I", d, q + 12)
        for _, pk in packets(geometry[gi][2]):
            for v in pk["pos"]:
                f = frame
                while f >= 0:
                    m = frames[f]
                    v = [m[9 + k] + sum(v[j] * m[3 * j + k] for j in range(3)) for k in range(3)]
                    f = m[12]
                ys.append(v[1])
    return min(ys), max(ys)


def materials(d):
    """Per geometry, the texture name of each material (None when untextured): material list 0x8 -> 0x7 -> texture 0x6 -> name 0x2."""
    clump = next(p for t, _, _, p in chunks(d) if t == 0x10)
    gl = next((s, p) for t, s, _, p in chunks(d, clump + 12) if t == 0x1A)
    out = []
    for t, s, _, p in chunks(d, gl[1] + 12, gl[1] + 12 + gl[0]):
        if t != 0x0F:
            continue
        names = []
        for t2, s2, _, p2 in chunks(d, p + 12, p + 12 + s):
            if t2 == 0x08:
                for t3, s3, _, p3 in chunks(d, p2 + 12, p2 + 12 + s2):
                    if t3 == 0x07:
                        tex = next((q for t4, _, _, q in chunks(d, p3 + 12, p3 + 12 + s3) if t4 == 0x06), None)
                        name = next((q for t5, _, _, q in chunks(d, tex + 12, tex + 12 + struct.unpack_from("<I", d, tex + 4)[0]) if t5 == 0x02), None) if tex else None
                        names.append(d[name + 12 : name + 28].split(b"\0")[0].decode() if name else None)
        out.append(names)
    return out


def packets(b):
    """Vertex packets of one native block: each is (count, {pos, uv, v4b, v3b} lists of tuples)."""
    out, i = [], 0
    while True:
        i = b.find(b"\x05\x01\x00\x01\x04\x80", i)
        if i < 0:
            return out
        n = b[i + 6]
        pk, j = {}, i + 4
        while True:  # four UNPACKs follow the dummy one, each preceded by STMOD+STCYCL
            j = b.find(b"\x05\x01\x00\x01", j + 4)
            cmd = b[j + 7]
            if j < 0 or cmd not in UNPACK or b[j + 6] != n:
                break
            name, sz = UNPACK[cmd]
            fmt = {"pos": "<3f", "uv": "<2f", "v4b": "<4B", "v3b": "<3b"}[name]
            pk[name] = [struct.unpack_from(fmt, b, j + 8 + sz * k) for k in range(n)]
            if len(pk) == 4:
                break
        out.append((n, pk))
        i += 8


def skin_info(d, skin):
    """(bone count, used bones, inverse bind matrices in HAnim list order, node id of each matrix)."""
    _, _, nodes = skeleton(d)
    nb, nu = skin[16], skin[17]
    used, o = list(skin[20 : 20 + nu]), 20 + nu
    ibm = [struct.unpack_from("<16f", skin, o + 64 * i) for i in range(nb)]
    return nb, used, ibm, [n[0] for n in nodes]


def bone_pos(m):
    """Bind-pose joint position (model units): origin of bone space, -t * R^T (row-vector matrices)."""
    return tuple(-sum(m[12 + k] * m[4 * j + k] for k in range(3)) for j in range(3))


def seg_dist(p, a, b):
    ab = [y - x for x, y in zip(a, b)]
    t = sum((q - x) * w for q, x, w in zip(p, a, ab)) / (sum(w * w for w in ab) or 1)
    t = max(0.0, min(1.0, t))
    return math.dist(p, [x + t * w for x, w in zip(a, ab)])


def build(d):
    """Per geometry: strips -> triangles and a rigid skin. Returns (skeleton, [primitive dicts])."""
    par, off, _ = skeleton(d)
    ids = frame_ids(d)
    prims = []
    mats = materials(d)
    for gi, (g, meshes, native, skin) in enumerate(geometries(d)):
        nb, used, ibm, nid = skin_info(d, skin)
        pos = [bone_pos(m) for m in ibm]
        parent = {ids[f]: ids[par[f]] for f in range(1, len(ids)) if par[f] > 0}
        row = {i: k for k, i in enumerate(nid)}
        # ESTIMATE: skin weights were not found in the file (see formats.md); rigid, nearest used bone segment
        segs = [(nid[u], pos[u], pos[row[parent[nid[u]]]] if nid[u] in parent else pos[u]) for u in used]
        pk = packets(native)
        k = 0
        for n_idx, mat in meshes:
            tot, cnt, vs = 0, 0, []
            while tot < n_idx:
                n, p = pk[k]
                k += 1
                skip = 2 if cnt else 0
                tot += n - skip
                cnt += 1
                vs += list(zip(p["pos"], p["v3b"], p["uv"]))[skip:]
            tris = []
            for i in range(2, len(vs)):
                t = [vs[i - 2], vs[i - 1], vs[i]]
                if len({v[0] for v in t}) == 3:
                    tris.append(t if i % 2 == 0 else [t[1], t[0], t[2]])
            joints = [min(segs, key=lambda s: seg_dist(v[0], s[1], s[2]))[0] for v in vs]
            prims.append({"material": mat, "texture": mats[gi][mat], "verts": vs, "joints": joints, "tris": tris})
        assert k == len(pk)
    return prims


def write_gltf(d, path, part):
    """glTF 2.0 with the skin (every node of the HAnim list is a joint), one primitive per RW mesh (material index in `extras`), materials by texture name (PNGs from tools/texture.py)."""
    import json

    prims = build(d)
    par, off, nodes = skeleton(d)
    ids = frame_ids(d)
    ibm = skin_info(d, geometries(d)[0][3])[2]
    nb = len(ibm)
    by_id = {nid: m for (nid, *_), m in zip(nodes, ibm)}
    blob, views, acc = bytearray(), [], []

    def add(data, ctype, count, typ, target=None, mm=None):
        while len(blob) % 4:
            blob.append(0)
        views.append({"buffer": 0, "byteOffset": len(blob), "byteLength": len(data), **({"target": target} if target else {})})
        blob.extend(data)
        a = {"bufferView": len(views) - 1, "componentType": ctype, "count": count, "type": typ}
        if mm:
            a["min"], a["max"] = mm
        acc.append(a)
        return len(acc) - 1

    jn = [{"name": f"bone{i}"} for i in range(nb)]
    for f in range(1, len(ids)):
        jn[ids[f]]["translation"] = list(off[f])  # rest rotation assumed identity (bone lengths match, see check)
        if par[f] > 0:
            jn[ids[par[f]]].setdefault("children", []).append(ids[f])
    root = ids[1]
    ibm_acc = add(b"".join(struct.pack("<16f", *[*by_id[i][:3], 0, *by_id[i][4:7], 0, *by_id[i][8:11], 0, *by_id[i][12:15], 1]) for i in range(nb)), 5126, nb, "MAT4")
    meshes, tex_ids = [], {}
    for p in prims:
        vs = [v for t in p["tris"] for v in t]
        n = len(vs)
        posd = b"".join(struct.pack("<3f", *v[0]) for v in vs)
        lo = [min(v[0][k] for v in vs) for k in range(3)]
        hi = [max(v[0][k] for v in vs) for k in range(3)]
        jmap = {v[0]: j for v, j in zip(p["verts"], p["joints"])}
        attrs = {
            "POSITION": add(posd, 5126, n, "VEC3", 34962, (lo, hi)),
            "NORMAL": add(b"".join(struct.pack("<3f", *(c / 127 for c in v[1])) for v in vs), 5126, n, "VEC3", 34962),
            "TEXCOORD_0": add(b"".join(struct.pack("<2f", *v[2]) for v in vs), 5126, n, "VEC2", 34962),
            "JOINTS_0": add(b"".join(struct.pack("<4B", jmap[v[0]], 0, 0, 0) for v in vs), 5121, n, "VEC4", 34962),
            "WEIGHTS_0": add(struct.pack("<4f", 1, 0, 0, 0) * n, 5126, n, "VEC4", 34962),
        }
        if p["texture"] not in tex_ids:
            tex_ids[p["texture"]] = len(tex_ids)
        meshes.append({"primitives": [{"attributes": attrs, "mode": 4, "material": tex_ids[p["texture"]], "extras": {"material": p["material"]}}]})
    nodes_out = jn + [{"mesh": i, "skin": 0} for i in range(len(meshes))]
    g = {
        "asset": {"version": "2.0", "generator": "tools/model.py"},
        "scene": 0,
        "scenes": [{"nodes": [root] + list(range(nb, len(nodes_out)))}],
        "nodes": nodes_out,
        "skins": [{"joints": list(range(nb)), "inverseBindMatrices": ibm_acc, "skeleton": root}],
        "meshes": meshes,
        "materials": [{"name": str(n), "pbrMetallicRoughness": {"baseColorTexture": {"index": i}, "metallicFactor": 0}, "alphaMode": "MASK"} for n, i in tex_ids.items()],
        "textures": [{"source": i} for i in tex_ids.values()],
        "images": [{"uri": f"../textures/{part}/{n}.png"} for n in tex_ids],
        "accessors": acc,
        "bufferViews": views,
        "buffers": [{"uri": path.name.replace(".gltf", ".bin"), "byteLength": len(blob)}],
    }
    path.write_text(json.dumps(g))
    path.with_suffix(".bin").write_bytes(blob)


def check(d):
    for g, meshes, native, skin in geometries(d):
        pk = packets(native)
        assert sum(m[0] for m in meshes) == sum(n for n, _ in pk) - 2 * (len(pk) - len(meshes))  # strips overlap by 2
        assert all(len(v) == n for n, p in pk for v in p.values()) and all(len(p) == 4 for _, p in pk)
        nb, used, ibm, nid = skin_info(d, skin)
        assert nb == len(nid) and max(used) < nb and sorted(nid) == list(range(nb))
        par, off, _ = skeleton(d)
        ids = frame_ids(d)
        pos = {nid[k]: bone_pos(m) for k, m in enumerate(ibm)}
        for f in range(1, len(ids)):  # matrix k belongs to HAnim list node k: bone lengths equal frame offsets
            if par[f] > 0:
                assert abs(math.dist(pos[ids[f]], pos[ids[par[f]]]) - math.dist(off[f], (0, 0, 0))) < 0.01
    prims = build(d)
    k, got = 0, []
    for g, meshes, *_ in geometries(d):
        got.append(sum(len(p["tris"]) for p in prims[k : k + len(meshes)]))
        k += len(meshes)
    assert got == [g["tris"] for g, *_ in geometries(d)]
    nrm = [math.hypot(*v[1]) for p in prims for v in p["verts"]]
    assert all(abs(x - 127) < 3 for x in nrm)  # normals are signed bytes of length ~127


if __name__ == "__main__":
    out = MODEL.parent.parent / "model"
    out.mkdir(exist_ok=True)
    for part in ("BODY", "HEAD_H", "HEAD_L", "WEAPON"):
        d = (MODEL / f"KAGETORA_{part}.rws").read_bytes()
        check(d)
        write_gltf(d, out / f"kagetora_{part.lower()}.gltf", part.lower())
    print("ok: geometry strips give the RW triangle counts, normals unit length, skin matrices match HAnim bone lengths (body, heads, weapon)")
