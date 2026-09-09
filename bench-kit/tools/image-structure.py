"""Is a render a PICTURE? Measures variation ACROSS PIXELS: distinct colours and the modal
colour's share. (Channel variance is the wrong metric: a uniform (117,0,0) field has a large
channel spread and no structure at all.)"""
import sys
from collections import Counter

from PIL import Image

for path in sys.argv[1:]:
    try:
        im = Image.open(path).convert("RGB")
    except Exception as e:  # noqa: BLE001
        print(f"{path}: UNREADABLE ({e})")
        continue
    w, h = im.size
    px = list(im.getdata())
    counts = Counter(px)
    modal, modal_n = counts.most_common(1)[0]
    frac = modal_n / len(px)
    verdict = "STRUCTURE" if len(counts) > 16 and frac < 0.98 else "FLAT"
    print(f"{path}: {w}x{h} distinct={len(counts)} modal={modal} modal_frac={frac:.4f} -> {verdict}")
