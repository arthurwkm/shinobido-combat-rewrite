"""Read files straight out of the ISO 9660 disc image, without extracting it.

    python tools/iso.py                 list every file: size, sector, path
    python tools/iso.py SCES_539.31 out extract one file to `out`

Other tools import `find_iso()` and `iso_files()` and read by offset, so the
3.9 GB image is never copied.
"""

import struct
import sys
from pathlib import Path

SECTOR = 2048
ROOT = Path(__file__).resolve().parent.parent


def find_iso():
    isos = sorted(ROOT.glob("*.iso"))
    if not isos:
        sys.exit(f"No .iso found in {ROOT}")
    return isos[0]


def iso_files(iso):
    """{path: (byte offset, size)} for every file on the disc; paths use '/', no ';1'."""
    files = {}
    with open(iso, "rb") as f:
        def read(lba, size):
            f.seek(lba * SECTOR)
            return f.read(size)

        pvd = read(16, SECTOR)
        assert pvd[1:6] == b"CD001", "not an ISO 9660 image"

        def walk(lba, size, prefix):
            data = read(lba, size)
            i = 0
            while i < len(data):
                n = data[i]
                if n == 0:  # records never cross a sector; skip the padding
                    i = (i // SECTOR + 1) * SECTOR
                    continue
                rec = data[i:i + n]
                i += n
                name = rec[33:33 + rec[32]]
                if name in (b"\0", b"\1"):
                    continue
                elba, esize = struct.unpack_from("<I", rec, 2)[0], struct.unpack_from("<I", rec, 10)[0]
                path = prefix + name.decode("ascii").split(";")[0]
                if rec[25] & 2:
                    walk(elba, esize, path + "/")
                else:
                    files[path] = (elba * SECTOR, esize)

        root = pvd[156:190]
        walk(struct.unpack_from("<I", root, 2)[0], struct.unpack_from("<I", root, 10)[0], "")
    return files


def read_file(iso, files, path, start=0, size=None):
    off, total = files[path]
    size = total - start if size is None else size
    with open(iso, "rb") as f:
        f.seek(off + start)
        return f.read(size)


if __name__ == "__main__":
    iso = find_iso()
    files = iso_files(iso)
    if len(sys.argv) == 3:
        Path(sys.argv[2]).write_bytes(read_file(iso, files, sys.argv[1]))
    else:
        for path, (off, size) in sorted(files.items(), key=lambda kv: kv[1][0]):
            print(f"{size:12d} {off // SECTOR:8d} {path}")
