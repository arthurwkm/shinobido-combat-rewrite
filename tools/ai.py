"""Parse the enemy AI tables `AI\\battle\\*.bin` and check the layout (docs/formats.md "Enemy AI").

    python tools/ai.py     parses every Busi* battle table, asserts the tables fill the file exactly
"""

import struct

from iso import find_iso
from volume import read_entry, volume_entries


def battle_tables(data):
    """[table][row] = list of (action id, weight, class, -1, -1); file = u32 tables, then per table u32 rows,
    per row u32 n and n 5-word records."""
    w = struct.unpack("<%di" % (len(data) // 4), data)
    i, tables = 1, []
    for _ in range(w[0]):
        rows, i = [], i + 1
        for _ in range(w[i - 1]):
            n = w[i]
            assert 0 < n < 64, (i, n)
            rows.append([tuple(w[i + 1 + 5 * k : i + 6 + 5 * k]) for k in range(n)])
            i += 1 + 5 * n
        tables.append(rows)
    assert i == len(w), "tables do not fill the file"
    return tables


def main():
    iso = find_iso()
    ents = volume_entries(iso)
    names = sorted(p for p in ents if p.startswith("AI\\battle\\Busi"))
    total = 0
    for p in names:
        tabs = battle_tables(read_entry(iso, ents, p))
        total += sum(len(t) for t in tabs)
    print("ok: %d Busi battle files, %d rows, every file filled exactly" % (len(names), total))


if __name__ == "__main__":
    main()
