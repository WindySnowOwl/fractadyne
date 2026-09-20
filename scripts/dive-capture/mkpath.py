import re,sys
run=sys.argv[1]; step=int(sys.argv[2])
out=[]
rows=[]
for l in open(f"{run}/stderr.txt",encoding="utf-8",errors="replace"):
    if "eval t=" not in l: continue
    rows.append((re.search(r"cx=(\S+)",l).group(1), re.search(r"cy=(\S+)",l).group(1),
                 re.search(r"l2=([\d.\-]+)",l).group(1),
                 *re.search(r"aim=\(([\d.e\-]+), ([\d.e\-]+)\)",l).groups()))
for n,r in enumerate(rows[::step]):
    out.append(f"{n:03d}\t"+"\t".join(r))
open(f"{run}/path.tsv","w",newline="\n").write("\n".join(out)+"\n")
print(run,len(out))
