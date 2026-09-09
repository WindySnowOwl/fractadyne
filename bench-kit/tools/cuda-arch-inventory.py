"""Inventory the CUDA fat binaries embedded in a Windows executable: for every fatbin container,
list each entry's kind (1 = PTX, 2 = SASS/cubin, 4 = relocatable) and SM architecture. The
container header is {magic 0xBA55ED50 u32, version u16, header_size u16, fat_size u64}; each entry
header is {kind u16, unknown u16, header_size u32, size u64, compressed_size u32, unknown u32,
minor u16, major u16, arch u32, ...} (layout as documented by the cuda-fatbin community tools)."""
import struct
import sys
from collections import Counter

MAGIC = b"\x50\xED\x55\xBA"
for path in sys.argv[1:]:
    data = open(path, "rb").read()
    found = Counter()
    containers = 0
    pos = data.find(MAGIC)
    while pos != -1:
        try:
            magic, version, hsize, fsize = struct.unpack_from("<IHHQ", data, pos)
        except struct.error:
            break
        if version == 1 and 16 <= hsize <= 64 and 0 < fsize < len(data):
            containers += 1
            p = pos + hsize
            end = min(pos + hsize + fsize, len(data))
            while p + 48 <= end:
                kind, _u1, ehsize, esize, _csz, _u2, minor, major, arch = struct.unpack_from("<HHIQIIHHI", data, p)
                if ehsize < 32 or esize == 0 or kind not in (1, 2, 4):
                    break
                found[(kind, arch)] += 1
                p += ehsize + esize
        pos = data.find(MAGIC, pos + 4)
    names = {1: "PTX", 2: "SASS", 4: "reloc"}
    print(f"{path}: {containers} fat-binary containers")
    for (kind, arch), n in sorted(found.items()):
        print(f"   {names.get(kind, kind):5s} sm_{arch}: {n} entries")
    if not found:
        print("   (no entries parsed)")
