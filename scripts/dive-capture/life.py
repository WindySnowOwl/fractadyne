import re, sys, math
from mpmath import mp, mpf, mpc, log, fabs
mp.dps = 260
src = sys.argv[1]
W,H = mpf(1467), mpf(1102)
rows=[]
for l in open(src,encoding="utf-8",errors="replace"):
    if "eval t=" not in l: continue
    g=lambda p: re.search(p,l)
    l2=mpf(g(r"l2=([\d.\-]+)").group(1))
    cx=mpf(g(r"cx=(\S+)").group(1)); cy=mpf(g(r"cy=(\S+)").group(1))
    pk=g(r"pick=Some\(\(([\d.e\-]+), ([\d.e\-]+)\)\)")
    am=g(r"aim=\(([\d.e\-]+), ([\d.e\-]+)\)")
    rows.append((l2,cx,cy,(mpf(pk.group(1)),mpf(pk.group(2))) if pk else None,
                 (mpf(am.group(1)),mpf(am.group(2)))))
def de(c, maxit=200000, bail=mpf(2)**128):
    z=mpc(0,0); d=mpc(0,0)
    for n in range(maxit):
        d=2*z*d+1; z=z*z+c; az=fabs(z)
        if az>bail: return n, az*log(az)/fabs(d)
    return None,None
print(f"{src}: {len(rows)} looks")
print("  k     l2    | GOAL DE/halfheight  octaves_left | AIM DE/halfheight  octaves_left | aim-goal(screen)")
import statistics
gl=[];al=[]
for k,(l2,cx,cy,pk,am) in enumerate(rows):
    u = mpf(4)/H/(mpf(2)**l2); half = mpf(2)/(mpf(2)**l2)   # world half-height
    out=[]
    for p in (pk,am):
        if p is None: out.append(None); continue
        px = cx + (p[0]-mpf("0.5"))*W*u
        py = cy - (p[1]-mpf("0.5"))*H*u
        n,dw = de(mpc(px,py))
        out.append(None if dw is None else dw/half)
    r_g,r_a = out
    og = float(-log(r_g,2)) if r_g else float('nan')
    oa = float(-log(r_a,2)) if r_a else float('nan')
    if r_g: gl.append(og)
    if r_a: al.append(oa)
    sep = float(((pk[0]-am[0])**2+(pk[1]-am[1])**2)**mpf("0.5")) if pk else float('nan')
    print(f"{k:3d} {float(l2):7.2f} | {mp.nstr(r_g,4) if r_g else '-':>12} {og:10.1f} | {mp.nstr(r_a,4) if r_a else '-':>12} {oa:10.1f} | {sep:.4f}")
print(f"\nGOAL octaves-of-life: median {statistics.median(gl):.1f}  min {min(gl):.1f}  max {max(gl):.1f}")
print(f"AIM  octaves-of-life: median {statistics.median(al):.1f}  min {min(al):.1f}  max {max(al):.1f}")
