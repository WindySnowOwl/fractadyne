import re,sys,statistics
from decimal import Decimal, getcontext
getcontext().prec=400
W,H=Decimal(1467),Decimal(1102)
def load(f):
    rows=[]
    for l in open(f,encoding="utf-8",errors="replace"):
        if "eval t=" not in l: continue
        rows.append((Decimal(re.search(r"l2=([\d.\-]+)",l).group(1)),
                     Decimal(re.search(r"cx=(\S+)",l).group(1)),
                     Decimal(re.search(r"cy=(\S+)",l).group(1)),
                     tuple(float(x) for x in re.search(r"aim=\(([\d.e\-]+), ([\d.e\-]+)\)",l).groups()),
                     "retarget=true" in l))
    return rows
def upp(l2): return Decimal(4)/H/(Decimal(2)**l2)
for f in sys.argv[1:]:
    rows=load(f); foe=[]; aimstep=[]
    for k in range(len(rows)-1):
        l20,cx0,cy0,a0,_=rows[k]; l21,cx1,cy1,a1,_=rows[k+1]
        u1=upp(l21); g=float(upp(l20)/u1)
        if abs(g-1)<1e-9: continue
        bx=float((cx0-cx1)/u1)/float(W); by=float((cy1-cy0)/u1)/float(H)
        fx=0.5-bx/(g-1); fy=0.5-by/(g-1)
        foe.append(max(abs(fx-0.5),abs(fy-0.5))/0.5)
        aimstep.append(((a1[0]-a0[0])**2+(a1[1]-a0[1])**2)**0.5)
    foe.sort()
    print(f"{f}")
    print(f"   focus of expansion off-centre : median {statistics.median(foe):.2f}  p90 {foe[int(.9*len(foe))]:.2f}  worst {max(foe):.2f}   (1.0 = frame edge)")
    print(f"   aim movement per look         : median {statistics.median(aimstep)*100:.2f}%  p90 {sorted(aimstep)[int(.9*len(aimstep))]*100:.2f}%  worst {max(aimstep)*100:.2f}% of the screen")
    print(f"   looks moving the aim >2%      : {sum(1 for s in aimstep if s>0.02)}/{len(aimstep)}")
