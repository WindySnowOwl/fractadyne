#!/usr/bin/env python3
"""Did the other renderer render the SAME VIEW, or just A view?

    tools/verify-views.py RESULTS_DIR [--prefix fs] [--masks DIR] [--quiet]

WHY THIS EXISTS
---------------
The structure guard in the bench lane asks one question: "is this a picture, or a flat field?"
It has never claimed to ask "is this the RIGHT picture", and on 2026-09-21 that gap cost us a
whole benchmark run.

FractalSharkCli accepts `--zoom 1e6.1249387366083` and silently TRUNCATES the fractional
exponent: that render is byte-identical to `--zoom 1e6`. Every scene in the run came out
under-zoomed by up to 10x. Every one exited 0. Every one was a real, structured, plausible
picture of the wrong place. And because an under-zoomed frame is a CHEAPER frame, the error did
not look like an error -- it looked like FractalShark being 20-50x faster than everything else,
which is exactly the shape a result takes when it is wrong in your favor.

A time is only meaningful if it is the time for the work you asked for. This checks that.

HOW
---
Palettes differ between renderers, so pixels cannot be compared and a plain grayscale
correlation is defeated by a palette inversion alone (it scores a perfect structural match as
-0.26). Two palette-independent signals are used instead:

  interior fraction  -- the share of pixels that never escaped. Read as black, which is true of
                        every renderer here except Fraktaler-3 (below), and strongly monotonic in
                        magnification, so it catches a wrong zoom directly. This is the signal
                        that settled the 0.543 case.
  edge correlation   -- correlation of gradient MAGNITUDE, which survives inversion and most
                        remapping, and catches a wrong center at the same zoom.

THE CONTROL IS NOT OPTIONAL. Each render is also scored against every OTHER scene's reference.
A metric that cannot tell the right scene from the wrong one is measuring nothing, so a match
must beat the best wrong scene, not merely score highly. That is reported on every line.

BLACK IS NOT ALWAYS INTERIOR
----------------------------
Read off an image, the interior fraction is the share of BLACK pixels, which assumes the
renderer draws only never-escaped pixels black. Fraktaler-3's default coloring does not: it
shades by distance estimate, so everything close to the boundary is black as well. On scene 59
its image is 62% black at 4K while 6.6% of its pixels never escaped, and this check called that
correct render a different magnification. No signal in such an image can stand in: the black
covers the right scene's interior and the wrong scenes' alike, and its noisy gray shading leaves
edge correlation negative at every blur tried (2026-10-31).

--interior-json supplies the never-escaped share measured by the renderer itself
(tools/exr-interior.py, from its raw iteration counts), and the magnification test uses that
instead of the image's black. For a scene it does not cover, the image's black can only
OVERSTATE the interior, so only a shortfall counts against the render.

THE REFERENCE'S INTERIOR IS A MASK, NOT ITS PICTURE
---------------------------------------------------
The same is true of the reference. Fractadyne's palette 0 draws escaped pixels dark as well: on
scene 59 its picture is 8.9% black where the scene has 5.5% interior, and on every deep minibrot
scene the four other renderers agreed with each other and not with it. A tolerance wide enough to
pass that (0.06 absolute) let a 2.14x under-zoom of scene 57 through. So the lane renders each
scene once more with an all-white palette, leaving only the interior black: fd-interior/SLUG.png.
Against that mask no correct render of any renderer fell more than 3% short, so on a scene with
1% interior or more a test may now fall at most 20% short (interior shrinks with the square of an
under-zoom: 1.12x). It may exceed the mask up to 2x, because a palette that draws escaped pixels
dark adds black (FractalShark: up to 1.44x); that catches an over-zoom of 1.41x. Each bound is
the tighter of that and the old 0.06, so it is never looser than before.

The share alone can be fooled the same way: Imagina's palette draws a black ring around scene 46
when zoomed out 2x, and the black share stayed at 7% while the minibrot shrank to a quarter. So
the check also asks WHICH pixels: at least 75% of the mask's interior pixels must be black in the
test. Palette darkness only adds black, so it cannot hide them coming out colored, which an
under-zoom or a moved center does. Correct renders: 0.85 and up; 1.26x under-zoomed: 0.68 at most.

On these scenes an interior contradiction stands against a decisive picture match. That match
says which SCENE (it beats the other scenes), not which magnification: scene 03 at 1.26x off was
decisive and called the same scene, with interior 0.28 against 0.41. A scene with under 1%
interior gives these tests nothing to measure.

THE SCALE SEARCH, FOR SCENES WITH NO INTERIOR
---------------------------------------------
Most scenes have none: Misiurewicz points, nuclei, spirals, dendrites. There, interior says
nothing about magnification, and the old check said "magnification agrees" anyway. Re-run on
the 2026-09-21 truncated-exponent run, it passed nine of the ten under-zoomed renders that way.
The scale search asks the question directly: edge correlation with the reference at
magnifications from 0.1x to 10x (twelve steps an octave), cropping the center of whichever image
is the wider view. On that run it names each truncation, near its true factor (scene 04: 3.78x
found, 3.90x true; 21 and 23: 5.04x, 5.07x), at r 0.53 to 0.94 against -0.10 to 0.49 as asked.
On correct renders of scenes with no interior (80), no other scale scoring 0.45 or more beat the
asked one by more than 0.07. It is not used on scenes with interior: a minibrot looks like itself across
scales, and correct renders of the deep minibrot scenes peaked at 0.3x to 0.9x (FractalShark's
scene 43: r 0.58 at 0.67x). Interior judges those, and more finely.

MEASURED (2026-11-01). Imagina rendered the 29 scenes it can take at 1.26x, 1.41x and 2x too
deep and too shallow: this check fails 144 of the 174, the old one 35, and every under-zoom of a
scene with interior is among them (36). Where the scale search names the factor (94) it is the
true one to about a twelfth of an octave, except once: the spiral of scene 47 at 2x too deep
matched best at 6x too shallow, and was failed all the same. The truncated run: 9 of 10
(old: 1). 139 correct renders by five renderers: none failed. What passes: an over-zoom of a
minibrot scene under 1.41x (the 2x allowance), and scenes 17 and 35 in palettes that match ours
at no scale, which are reported as unchecked.

Without the masks (results from before 2026-11-01, or a run with no Fractadyne lane) the check
falls back to the picture's black, the old 0.06 tolerance and no scale search, and says so; its
verdicts there are the old check's exactly (93 scenes, 8 runs).

Exit 0 if every pair agrees, 1 if any disagrees, 2 on a usage error.
"""
import os
import sys
import glob
import json
import argparse

try:
    import numpy as np
    from PIL import Image
except ImportError:
    sys.stderr.write("verify-views: needs numpy and Pillow (pip install pillow numpy)\n")
    sys.exit(2)

# Downsample before measuring: this compares FRAMING, not resampling detail, and a 4K pair costs
# nothing to load at this size.
RES = (640, 360)
# Interior is "did not escape" = black in every palette these renderers ship. The threshold is on
# the RGB sum, so it stays black-ish rather than exactly zero after a Lanczos resize.
INTERIOR_SUM = 40
# Interior against the reference's MASK (see the docstring). Below INTERIOR_SIGNAL the scene has
# too little interior to measure magnification by. A test may fall INTERIOR_SHORT short of the
# mask (correct renders: at most 3%) and exceed it INTERIOR_EXCESS times (correct renders: 1.44x).
MASK_DIR = "fd-interior"
INTERIOR_SIGNAL = 0.01
INTERIOR_SHORT = 0.20
INTERIOR_EXCESS = 2.0
# Interior RECALL: the share of the mask's interior pixels that are black in the test. Correct
# renders: 0.85 and up (59 renders); a 1.26x under-zoom: 0.68 at most.
INTERIOR_RECALL = 0.75
# The old absolute tolerance, for results with no masks, and for the one thing interior still says
# on a scene that has none: a frame that is largely black is somewhere else.
INTERIOR_TOL = 0.06
# The scale search. Edge features at SCALE_RES from images first reduced to SCALE_WORK, so a 10x
# crop still has 192 source pixels across; blurred so that a twelfth of an octave of misalignment
# does not erase a match. A render is at a different magnification when its best match is at
# least SCALE_MIN_OCTAVES away, scores SCALE_R_MIN there, and beats the asked magnification by
# SCALE_GAIN (correct renders: at most 0.07; the truncated run: 0.32 and up).
SCALE_WORK = (1920, 1080)
SCALE_RES = (320, 180)
SCALE_STEPS = 12
SCALE_SPAN = 40
SCALE_MIN_OCTAVES = 0.25
SCALE_R_MIN = 0.45
SCALE_GAIN = 0.25
EDGE_MIN = 0.35
EDGE_MARGIN = 0.10
# A DECISIVE edge match settles it on its own, and the interior test is not allowed to veto it.
# Scene 17 is why: FractalShark draws the same dendrite on a near-black background, so 16.7% of
# its frame reads as "interior" where ours reads 0%, and the interior test called a correct render
# wrong. Edge correlation there was 0.779 against 0.028 for the best wrong scene - not a close
# call. Interior fraction earns its place on the scenes where edge correlation is ambiguous, which
# is exactly where a wrong zoom lands; in the truncated-exponent run no scene reached even 0.28.
EDGE_DECISIVE = 0.50
EDGE_DECISIVE_MARGIN = 0.15
# A mirrored reference has to beat the upright one by this much before the render is
# called mirrored; near-zero correlations on both sides must not flip a coin.
ORIENT_MARGIN = 0.20
# The control-margin route to 'decisive'. Verified against both datasets: no render from
# the truncated-zoom run reaches it, and the correct-but-dark scene 17 does.
EDGE_CONTROL_MIN = 0.20
EDGE_CONTROL_FACTOR = 4.0
EDGE_CONTROL_FLOOR = 0.05


def load(path):
    a = Image.open(path).convert("RGB").resize(RES, Image.LANCZOS)
    return np.asarray(a, dtype=np.float64)


def interior_fraction(rgb):
    return float((rgb.sum(axis=2) < INTERIOR_SUM).mean())


def edge_feature(rgb):
    """Gradient magnitude of luminance, standardized. Invariant to palette inversion."""
    lum = rgb.mean(axis=2)
    gy, gx = np.gradient(lum)
    e = np.hypot(gx, gy).ravel()
    e -= e.mean()
    n = np.linalg.norm(e)
    return e / n if n else e


def orientation_features(rgb):
    """Edge features for the reference and its three mirror/rotation variants.

    A MIRRORED render is invisible to everything else here. Interior fraction is exactly
    flip-invariant, so the magnification test cannot see it; and if the two palettes differ
    enough that edge correlation is near zero anyway, the image test will not see it either.

    It is also not hypothetical. Our headless Imagina fork wrote its PPM rows top-down out of a
    buffer that is stored bottom-up, so every Imagina image in this kit was upside down, and the
    lane passed all ten because 'magnification agrees' was true of a mirror image too. Checking
    the flips costs three more correlations and names the fault outright.
    """
    a = np.asarray(Image.fromarray(rgb.astype(np.uint8)))
    return {
        "as-is": edge_feature(rgb),
        "flipped top-to-bottom": edge_feature(a[::-1, :, :].astype(np.float64)),
        "flipped left-to-right": edge_feature(a[:, ::-1, :].astype(np.float64)),
        "rotated 180": edge_feature(a[::-1, ::-1, :].astype(np.float64)),
    }


def mask_interior(path):
    """An interior mask, measured three ways: the share as a picture is measured here (after the
    downsample), the share per pixel at full size (as a renderer's raw counts are), and WHICH
    pixels are interior after the downsample."""
    im = Image.open(path).convert("RGB")
    full = float((np.asarray(im, dtype=np.int32).sum(axis=2) < INTERIOR_SUM).mean())
    small = np.asarray(im.resize(RES, Image.LANCZOS), dtype=np.float64).sum(axis=2) < INTERIOR_SUM
    return float(small.mean()), full, small


def judge_interior(ti, ri, mark, masked, recall=None):
    """('agrees' | 'contradicts' | 'none', why). 'none': the scene has too little interior to say."""
    d = ti - ri
    if not masked:
        ok = (d >= -INTERIOR_TOL) if mark == "b" else (abs(d) <= INTERIOR_TOL)
        return ("agrees", "") if ok else (
            "contradicts", "interior %.4f against %.4f in the reference's picture" % (ti, ri))
    if ri < INTERIOR_SIGNAL:
        if mark != "b" and d > INTERIOR_TOL:
            return "contradicts", "interior %.4f where the scene has %.4f" % (ti, ri)
        return "none", ""
    # WHERE before HOW MUCH: palette darkness only adds black, so it cannot hide the scene's
    # interior pixels coming out colored, which is what an under-zoom or a moved center does.
    if recall is not None and recall < INTERIOR_RECALL:
        return "contradicts", ("only %.0f%% of the scene's interior is black in it (an under-zoom "
                               "shrinks it; a moved center moves it)" % (100 * recall))
    # Each side is the TIGHTER of the relative bound and the old absolute one, so it is never
    # looser than the old check: scene 03 has 41% interior, where 20% short would be 0.08 and 2x
    # over would let a 2x over-zoom through (Imagina, 2026-11-01), both of which 0.06 catches.
    if d < -min(ri * INTERIOR_SHORT, INTERIOR_TOL):
        return "contradicts", "interior %.4f against %.4f in the reference's mask" % (ti, ri)
    # Image black at most OVERSTATES the interior, so for "b" only a shortfall counts.
    if mark != "b" and d > min(ri * (INTERIOR_EXCESS - 1.0), INTERIOR_TOL):
        return "contradicts", ("interior %.4f against %.4f in the reference's mask (an over-zoom "
                               "grows it)" % (ti, ri))
    return "agrees", ""


KERNEL = np.array([1, 4, 6, 4, 1], dtype=np.float64) / 16


def blur(a):
    """Separable binomial blur (about one pixel), edges replicated."""
    p = np.pad(a, 2, mode="edge")
    p = sum(KERNEL[i] * p[:, i:i + a.shape[1]] for i in range(5))
    return sum(KERNEL[i] * p[i:i + a.shape[0], :] for i in range(5))


def scale_feature(im, box):
    """Blurred edge magnitude of box (left, top, right, bottom) of a PIL L image, standardized."""
    a = np.asarray(im.resize(SCALE_RES, Image.LANCZOS, box=box), dtype=np.float64)
    gy, gx = np.gradient(a)
    e = blur(np.hypot(gx, gy)).ravel()
    e -= e.mean()
    n = np.linalg.norm(e)
    return e / n if n else e


def centre_box(size, k):
    w, h = size
    return ((w - w / k) / 2, (h - h / k) / 2, (w + w / k) / 2, (h + h / k) / 2)


def scale_search(ref_path, test_path):
    """Edge correlation of the test with the reference at magnifications 0.1x to 10x.

    k > 1 asks "is the test k times too SHALLOW?": its field is k times wider, so the center 1/k
    of the TEST is compared with the whole reference. k < 1 asks the opposite, cropping the
    reference. Returns (r at the asked magnification, best k, r there), or None when the two
    images' shapes differ, so that cropping one cannot frame the other.
    """
    ref = Image.open(ref_path).convert("L")
    test = Image.open(test_path).convert("L")
    if abs(ref.size[0] * test.size[1] - test.size[0] * ref.size[1]) > max(ref.size[1], test.size[1]):
        return None
    ref = ref.resize(SCALE_WORK, Image.LANCZOS)
    test = test.resize(SCALE_WORK, Image.LANCZOS)
    whole = (0, 0) + SCALE_WORK
    full_r, full_t = scale_feature(ref, whole), scale_feature(test, whole)
    r1, best = 0.0, (1.0, -2.0)
    for j in range(-SCALE_SPAN, SCALE_SPAN + 1):
        k = 2.0 ** (j / SCALE_STEPS)
        if j == 0:
            r = r1 = float(np.dot(full_r, full_t))
        elif j > 0:
            r = float(np.dot(full_r, scale_feature(test, centre_box(SCALE_WORK, k))))
        else:
            r = float(np.dot(scale_feature(ref, centre_box(SCALE_WORK, 1 / k)), full_t))
        if r > best[1]:
            best = (k, r)
    return r1, best[0], best[1]


def judge_scale(found):
    """'contradicts' when the test matches best at another magnification, decisively; 'agrees'
    when it matches best at the asked one, clearly; otherwise 'none'."""
    if found is None:
        return "none"
    r1, k, r = found
    far = abs(np.log2(k)) >= SCALE_MIN_OCTAVES - 1e-9
    if far and r >= SCALE_R_MIN and r >= r1 + SCALE_GAIN:
        return "contradicts"
    if not far and r >= SCALE_R_MIN:
        return "agrees"
    return "none"


def scale_words(k):
    return "%.2fx too %s" % (k if k >= 1 else 1 / k, "shallow" if k >= 1 else "deep")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("results_dir")
    ap.add_argument("--prefix", default="fs",
                    help="filename prefix of the renderer under test (default: fs)")
    ap.add_argument("--quiet", action="store_true")
    ap.add_argument("--json", metavar="PATH",
                    help="also write per-scene verdicts here, for the lane to consume")
    ap.add_argument("--interior-json", metavar="PATH",
                    help="never-escaped share per scene, measured by the renderer under test from "
                         "its own raw output (tools/exr-interior.py). For a renderer whose coloring "
                         "draws escaped pixels black, so its image overstates the interior")
    ap.add_argument("--masks", metavar="DIR",
                    help="the reference's interior masks, SLUG.png (default: RESULTS_DIR/%s, "
                         "which the kit's Fractadyne lane writes)" % MASK_DIR)
    args = ap.parse_args()

    d = args.results_dir
    if not os.path.isdir(d):
        sys.stderr.write("verify-views: no such directory: %s\n" % d)
        return 2

    # Fractadyne is the reference because it is the lane cross-validated pixel-for-pixel against
    # Fraktaler-3; it is the only image here already known to be the right place.
    refs = {}
    for p in sorted(glob.glob(os.path.join(d, "fd-*.png"))):
        refs[os.path.basename(p)[3:-4]] = p
    if not refs:
        sys.stderr.write("verify-views: no fd-*.png reference renders in %s\n" % d)
        return 2

    # The lane spells dots as 'p' in its output names, and may carry a -rN rep suffix.
    # Fraktaler-3 names its own output from the scene file, with the lane's tag at the END
    # ("59-iter-p951094-1e68-f3.png"). This check looked only for the prefix spelling, so it
    # found no Fraktaler-3 render to check and that lane went unchecked in every run.
    tests = {}
    for slug in refs:
        for cand in (
            os.path.join(d, "%s-%s-r1.png" % (args.prefix, slug.replace(".", "p"))),
            os.path.join(d, "%s-%s.png" % (args.prefix, slug.replace(".", "p"))),
            os.path.join(d, "%s-%s.png" % (args.prefix, slug)),
            os.path.join(d, "%s-%s.png" % (slug, args.prefix)),
        ):
            if os.path.exists(cand):
                tests[slug] = cand
                break

    if not tests:
        print("verify-views: no %s-*.png renders to check (lane skipped?)" % args.prefix)
        return 0

    raw_int = None
    if args.interior_json:
        try:
            with open(args.interior_json) as fh:
                raw_int = json.load(fh)
        except (OSError, ValueError) as e:
            sys.stderr.write("verify-views: cannot read %s: %s\n" % (args.interior_json, e))
            return 2

    R = {s: load(p) for s, p in refs.items()}
    Rint = {s: interior_fraction(v) for s, v in R.items()}
    Redge = {s: edge_feature(v) for s, v in R.items()}
    Rorient = {s: orientation_features(v) for s, v in R.items()}
    mask_dir = args.masks or os.path.join(d, MASK_DIR)
    Rmask = {}
    for s in tests:
        mp = os.path.join(mask_dir, s + ".png")
        if os.path.exists(mp):
            Rmask[s] = mask_interior(mp)

    if raw_int is not None:
        print("test int: r = never escaped, from the renderer's raw output (%s, %d of %d scenes);"
              % (os.path.basename(args.interior_json), sum(1 for s in tests if s in raw_int), len(tests)))
        print("          b = black share of its image, which this renderer's coloring overstates,")
        print("              so only a SHORTFALL counts against the render")
    if len(Rmask) == len(tests):
        print("ref int: m = the reference's interior mask (%s); magnification by interior where the"
              % (MASK_DIR if args.masks is None else mask_dir))
        print("         scene has %.0f%% or more, by the scale search where it has less" % (100 * INTERIOR_SIGNAL))
    else:
        print("ref int: m = interior mask, i = the reference picture's black, which overstates it. %d of %d"
              % (len(tests) - len(Rmask), len(tests)))
        print("         scenes have no mask in %s, so for them the check is the COARSE one:" % mask_dir)
        print("         interior within %.2f absolute, and no scale search" % INTERIOR_TOL)
    print("%-28s %9s %9s %7s %8s %9s %13s  %s"
          % ("scene", "ref int", "test int", "d_int", "edge r", "best wrong", "best scale", "verdict"))
    print("-" * 110)

    failures = []
    unconfirmed = []
    mirrors = []
    verdicts = {}
    for slug in sorted(tests):
        t = load(tests[slug])
        black, te = interior_fraction(t), edge_feature(t)
        # Where the interior comes from decides how far it can be trusted: the renderer's own
        # count of never-escaped pixels is exact; the black share of its image is exact only if
        # it draws nothing else black, and an upper bound for a renderer declared not to.
        if raw_int is not None and slug in raw_int:
            ti, isrc, mark = float(raw_int[slug]), "raw", "r"
        elif raw_int is not None:
            ti, isrc, mark = black, "image black, an upper bound", "b"
        else:
            ti, isrc, mark = black, "image black", " "
        # The reference side: its mask, measured the way the test is (a picture after the
        # downsample; raw counts per pixel), or, with no mask, its own picture's black.
        masked = slug in Rmask
        if masked:
            ri, rsrc, rmark = Rmask[slug][1 if mark == "r" else 0], "mask", "m"
        else:
            ri, rsrc, rmark = Rint[slug], "image black", "i"
        dint = ti - ri
        # A scene WITH interior, against its mask: the interior verdict is binding (below).
        binding = masked and ri >= INTERIOR_SIGNAL
        recall = None
        if binding:
            m = Rmask[slug][2]
            recall = float(((t.sum(axis=2) < INTERIOR_SUM) & m).sum() / max(1, m.sum()))
        int_state, int_why = judge_interior(ti, ri, mark, masked, recall)
        int_ok = int_state != "contradicts"
        # Scale search only where interior cannot judge magnification (see the docstring).
        found = scale_search(refs[slug], tests[slug]) if masked and ri < INTERIOR_SIGNAL else None
        scale_state = judge_scale(found)
        scale_bad = scale_state == "contradicts"
        er = float(np.dot(Redge[slug], te))
        wrong_r, wrong_s = max(
            (float(np.dot(Redge[o], te)), o) for o in R if o != slug
        ) if len(R) > 1 else (0.0, "-")

        # Decisive either by absolute strength, OR by beating the control by a wide factor.
        # The second clause is what stops the interior test vetoing a correct render whose
        # absolute correlation is modest because the palettes differ. Scene 17 is the case:
        # FractalShark draws the same dendrite on a near-black background, so 16.7% of its frame
        # reads as "interior" against our 0%, and on a shared palette its edge score fell to
        # 0.339 - under the absolute bar, but nineteen times the best WRONG scene. A score that
        # far above the control identifies the scene whatever its absolute value.
        decisive = (er >= EDGE_DECISIVE and er >= wrong_r + EDGE_DECISIVE_MARGIN) or                    (er >= EDGE_CONTROL_MIN and er >= EDGE_CONTROL_FACTOR * max(wrong_r, EDGE_CONTROL_FLOOR))
        # ...but a decisive match says WHICH SCENE, not which magnification: it beats the OTHER
        # scenes, and a render 1.26x off still beats them. Scene 03 at 1.26x and 1.41x off
        # (Imagina, 2026-11-01) was decisive and called the same scene, with interior 0.28 to
        # 0.56 against 0.41. Scene 17's excuse - black that is not interior - cannot arise against
        # a mask on a scene that HAS interior, and no correct render of one contradicted it (49).
        # So there an interior contradiction stands; on a scene with none, decisive still wins.
        decisive_mag = decisive and not (binding and not int_ok)
        ok = decisive_mag or (int_ok
                              and er >= EDGE_MIN
                              and er >= wrong_r + EDGE_MARGIN)
        # TWO DIFFERENT VERDICTS, because the two signals answer different questions and only one
        # of them is palette-proof.
        #
        # Interior fraction tracks MAGNIFICATION. A wrong zoom moves it hard - scene 03 went
        # 0.427 to 0.251 on a single truncated mantissa - and no palette choice moves it at all,
        # because "did this pixel escape" is not a coloring decision. When it disagrees, the
        # renderer is looking somewhere else and the time is void.
        #
        # Edge correlation compares PICTURES, and two renderers can draw the same iteration field
        # so differently that it collapses. FractalShark cycles its palette once per iteration
        # and has no CLI option to slow it; at 1.6e148 with 800k iterations the bands alias into
        # what looks like static when downsampled, while our own render of that scene is the one
        # we apply --normalize to, mapping the same field onto a single slow gradient. Same view,
        # opposite images, correlation -0.222. I called that a failed render and said so in a
        # report; it was neither a failure nor theirs.
        #
        # So a low correlation with a MATCHING interior is "could not confirm", not "wrong", and
        # it must not delete a time or accuse anyone.
        mag_ok = int_ok or decisive_mag

        # THE SCALE SEARCH OVERRIDES A DECISIVE MATCH. A match "as asked" can be the echo of a far
        # better one: the truncated render of scene 08 scored 0.487 as asked, nearly decisive, and
        # 0.809 at the 8.9x it was actually off by. A render that matches another magnification
        # that much better is at that magnification.
        if scale_bad:
            ok = mag_ok = False
        # What the magnification verdict rests on. "" = nothing could check it: no interior to
        # measure, and the pictures match at no scale. That is NOT agreement, and is not called it.
        mag_by = ("" if not mag_ok else
                  "interior" if int_state == "agrees" else
                  "scale search" if scale_state == "agrees" else
                  "image match" if decisive_mag else "")

        # ORIENTATION. If a mirrored or rotated reference matches decisively better than the
        # reference as it stands, the render is the right view drawn the wrong way up. That is a
        # defect, it is precise, and it is worth stating outright rather than leaving as an
        # unexplained low correlation.
        orient, orient_r = "as-is", er
        for name, feat in Rorient[slug].items():
            r = float(np.dot(feat, te))
            if r > orient_r + ORIENT_MARGIN:
                orient, orient_r = name, r
        mirrored = orient != "as-is" and orient_r >= EDGE_MIN
        if mirrored:
            ok = False
        if mirrored:
            mirrors.append((slug, orient, orient_r))
        elif not ok and not mag_ok:
            why = ("its best match is %s (r %.3f, against %.3f as asked)"
                   % (scale_words(found[1]), found[2], found[0]) if scale_bad else int_why)
            failures.append((slug, why))
        elif not ok:
            unconfirmed.append((slug, mag_by))
        verdicts[slug] = {
            "same_view": bool(ok),
            "magnification_ok": bool(mag_ok),
            # Whether anything could CHECK the magnification, and what. A false here with
            # magnification_ok true means "not contradicted", not "agrees".
            "magnification_checked": bool(mag_by) or not mag_ok,
            "magnification_by": mag_by if mag_ok else ("scale search" if scale_bad else "interior"),
            "orientation": orient,
            "orientation_r": round(orient_r, 3),
            "ref_interior": round(ri, 4),
            "ref_interior_source": rsrc,
            "ref_image_black": round(Rint[slug], 4),
            "test_interior": round(ti, 4),
            "test_interior_source": isrc,
            "test_image_black": round(black, 4),
            "d_interior": round(dint, 4),
            "interior_recall": round(recall, 3) if recall is not None else None,
            "interior_judgement": int_state,
            "scale_judgement": scale_state if found else "not run",
            "scale_k": round(found[1], 4) if found else None,
            "scale_r": round(found[2], 3) if found else None,
            "scale_r_asked": round(found[0], 3) if found else None,
            "edge_r": round(er, 3),
            "best_wrong_r": round(wrong_r, 3),
            "best_wrong_scene": wrong_s,
        }
        if found is None:
            scol = "-"
        elif abs(np.log2(found[1])) < 1e-9:
            scol = "1x %.2f" % found[2]
        else:
            scol = "%s%.2fx %.2f" % ("/" if found[1] < 1 else "", found[1] if found[1] >= 1 else 1 / found[1], found[2])
        print("%-28s %8.4f%s %8.4f%s %+7.4f %8.3f %9.3f %13s  %s"
              % (slug, ri, rmark, ti, mark, dint, er, wrong_r, scol,
                 "same scene" if ok
                 else ("** %s **" % orient.upper() if mirrored
                       else ("** WRONG ZOOM: best match %s **" % scale_words(found[1]) if scale_bad
                             else ("unconfirmed (palette?)" if mag_by
                                   else "unconfirmed, magnification unchecked" if mag_ok
                                   else "** NOT THE SCENE **")))))

    if args.json:
        with open(args.json, "w") as fh:
            json.dump(verdicts, fh, indent=1, sort_keys=True)

    print()
    if mirrors:
        print("%d of %d renders are MIRRORED or ROTATED relative to the reference:" % (len(mirrors), len(tests)))
        for slug, orient, r in mirrors:
            print("  %-28s matches the reference %s (r %.3f)" % (slug, orient, r))
        if not args.quiet:
            print()
            print("The right view, drawn the wrong way up. Nothing else here can see this:")
            print("interior fraction is exactly flip-invariant, and a large palette difference")
            print("flattens the image test. Check the row order where the pixel buffer is")
            print("serialized - a bottom-up buffer written into a top-down format does this.")
        print()
        # A mirror is a DEFECT, not an ambiguity: the view is identifiable and it is wrong.
        # Unlike an unmatched palette, there is something specific to go and fix.
        return 1

    if unconfirmed:
        agree = [u[0] for u in unconfirmed if u[1]]
        unchecked = [u[0] for u in unconfirmed if not u[1]]
        if agree:
            print("%d of %d could not be CONFIRMED, but their magnification agrees: %s."
                  % (len(agree), len(tests), ", ".join(agree)))
        if unchecked:
            print("%d of %d could not be CONFIRMED, and nothing could check their magnification: %s."
                  % (len(unchecked), len(tests), ", ".join(unchecked)))
            if not args.quiet:
                print("  These scenes have no interior to measure it by, and the pictures match at")
                print("  no scale tried. Nothing contradicts them either.")
        if not args.quiet:
            print("  Usually a palette difference, not a wrong view. Two renderers can draw the")
            print("  same iteration field so differently that the correlation collapses: a")
            print("  palette cycling once per iteration aliases into static when downsampled,")
            print("  while a normalized one maps the same field onto a single slow gradient.")
            print("  The time stands. Look at the image at 1:1 before concluding anything, and")
            print("  do not call it a failure of the other renderer on this evidence.")
        print()

    if failures:
        print("%d of %d renders do NOT depict the scene they were asked for:" % (len(failures), len(tests)))
        for slug, why in failures:
            print("  %-28s %s" % (slug, why))
        if not args.quiet:
            print()
            print("Their INTERIOR FRACTION disagrees, which tracks magnification, or their")
            print("picture matches another magnification far better than the one asked for.")
            print("Check the zoom and center actually reaching the renderer. A")
            print("truncated mantissa under-zooms, and an under-zoomed frame is CHEAPER, so this")
            print("always arrives looking like good news. A time for the wrong view is not slow")
            print("or fast, it is VOID. One exception, read off an image: a renderer whose")
            print("coloring draws ESCAPED pixels black looks too deep. Look at the image, and")
            print("give its own never-escaped counts with --interior-json.")
        return 1

    if unconfirmed:
        print("No render contradicts its scene; %d could not be positively confirmed."
              % len(unconfirmed))
        return 0

    print("All %d renders depict the same scene as the Fractadyne reference." % len(tests))
    return 0


if __name__ == "__main__":
    sys.exit(main())
