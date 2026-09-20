from PIL import Image
import numpy as np, glob, re, sys
for run in sys.argv[1:]:
    fs=sorted(glob.glob(f"{run}/frames/*.jpg"))
    if not fs: print(f"{run:12s}: none"); continue
    W,H=Image.open(fs[0]).size
    x0,x1=int(W*0.28),int(W*0.72); y0,y1=int(H*0.25),int(H*0.85)
    vals=[float(np.asarray(Image.open(f).convert("L"),dtype=np.float32)[y0:y1,x0:x1].std()) for f in fs]
    flat=sum(1 for v in vals if v<1.0)
    rungs=0; empty=0; reads=0
    for line in open(f"{run}/stderr.txt",encoding="utf-8",errors="replace"):
        if "visible-res f=" in line: rungs+=1
        if "norm reading EMPTY" in line: empty+=1
        if "norm range:" in line or "norm reading" in line: reads+=1
    print(f"{run:12s} frames {len(fs):3d} | SCREEN BLANK {flat:3d} ({100*flat/len(fs):3.0f}%) | rung changes {rungs:3d} | empty passes {100*empty/max(reads,1):3.0f}%")
    print(f"     {''.join('#' if v<1.0 else '.' for v in vals)}")
