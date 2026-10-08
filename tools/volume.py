"""List and extract files from VOLUME.DAT, Acquire's FADEBABE container.

    python tools/volume.py                       list: size, path
    python tools/volume.py extract 'motion\\AT01.nmt' [more paths or prefixes...]
                                                 copy into extracted/volume/, '\\' -> '/'

Layout (big-endian), as in Scarlet's DATAkiba.cs (Akiba's Trip, also Acquire):
    header  u32 magic 0xFADEBABE, count, count again, data_offset, data_size
    entry   u32 hash, offset, size, 0, path_offset, path_length   (24 bytes, `count` of them)
offsets and path offsets are relative to data_offset. Paths use '\\'.
"""

import struct
import sys

from iso import ROOT, find_iso, iso_files

OUT = ROOT / "extracted" / "volume"


def volume_entries(iso):
    """{path: (byte offset in the ISO, size)} for every file inside VOLUME.DAT."""
    base, _ = iso_files(iso)["VOLUME.DAT"]
    with open(iso, "rb") as f:
        f.seek(base)
        magic, count, count2, data_off, _ = struct.unpack(">5I", f.read(20))
        assert magic == 0xFADEBABE and count == count2, "not a FADEBABE container"
        toc = f.read(24 * count)
        entries = {}
        for i in range(count):
            _, off, size, _, path_off, path_len = struct.unpack_from(">6I", toc, i * 24)
            f.seek(base + data_off + path_off)
            # A few paths hold Shift-JIS; latin-1 keeps every byte round-trippable.
            path = f.read(path_len).rstrip(b"\0").decode("latin-1")
            entries[path] = (base + data_off + off, size)
    return entries


def read_entry(iso, entries, path, start=0, size=None):
    off, total = entries[path]
    size = total - start if size is None else size
    with open(iso, "rb") as f:
        f.seek(off + start)
        return f.read(size)


if __name__ == "__main__":
    iso = find_iso()
    entries = volume_entries(iso)
    assert len(entries) == 6089, len(entries)  # count in this disc's header (SCES-53931)
    if sys.argv[1:2] == ["extract"]:
        wanted = [p for p in entries if any(p == a or p.startswith(a) for a in sys.argv[2:])]
        for p in wanted:
            dest = OUT / p.replace("\\", "/")
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(read_entry(iso, entries, p))
        print(f"{len(wanted)} files -> {OUT}")
    else:
        for p, (_, size) in sorted(entries.items()):
            print(f"{size:10d} {p}")
