"""Load every CUDA fat binary embedded in an executable onto GPU 0 through the CUDA driver API
(nvcuda.dll, shipped with the driver) and report the result code per container. A fat binary with
no code for this GPU's architecture fails with CUDA_ERROR_NO_BINARY_FOR_GPU (209); a compatible one
loads with CUDA_SUCCESS (0). This is the runtime's own verdict, independent of any parsing."""
import ctypes
import struct
import sys
from ctypes import byref, c_char_p, c_int, c_void_p

cuda = ctypes.WinDLL("nvcuda.dll")

def check(name, r):
    if r != 0:
        raise SystemExit(f"{name} failed: {errstr(r)}")

def errstr(r):
    s = c_char_p()
    cuda.cuGetErrorString(c_int(r), byref(s))
    return f"{r} ({s.value.decode() if s.value else '?'})"

check("cuInit", cuda.cuInit(0))
dev = c_int()
check("cuDeviceGet", cuda.cuDeviceGet(byref(dev), 0))
name = ctypes.create_string_buffer(256)
cuda.cuDeviceGetName(name, 256, dev)
major, minor = c_int(), c_int()
cuda.cuDeviceGetAttribute(byref(major), 75, dev)  # COMPUTE_CAPABILITY_MAJOR
cuda.cuDeviceGetAttribute(byref(minor), 76, dev)  # COMPUTE_CAPABILITY_MINOR
drv = c_int()
cuda.cuDriverGetVersion(byref(drv))
print(f"device 0: {name.value.decode()} sm_{major.value}{minor.value}, driver CUDA API {drv.value // 1000}.{(drv.value % 1000) // 10}")
ctx = c_void_p()
check("cuCtxCreate", cuda.cuCtxCreate_v2(byref(ctx), 0, dev))

MAGIC = b"\x50\xED\x55\xBA"
for path in sys.argv[1:]:
    data = open(path, "rb").read()
    print(path)
    pos = data.find(MAGIC)
    n = 0
    results = {}
    while pos != -1:
        magic, version, hsize, fsize = struct.unpack_from("<IHHQ", data, pos)
        if version == 1 and 16 <= hsize <= 64 and 0 < fsize < len(data):
            blob = data[pos:pos + hsize + fsize]
            buf = ctypes.create_string_buffer(blob, len(blob))
            mod = c_void_p()
            r = cuda.cuModuleLoadData(byref(mod), buf)
            results[r] = results.get(r, 0) + 1
            if r == 0:
                cuda.cuModuleUnload(mod)
            n += 1
        pos = data.find(MAGIC, pos + 4)
    for r, c in sorted(results.items()):
        print(f"   {c} of {n} fat binaries -> {errstr(r)}")
