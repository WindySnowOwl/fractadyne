# Offloading to FractalShark — what is worth asking for, and what is not

Written 2026-09-21, after FractalShark 0.543 added a CLI client/server at our request.
Status: an assessment, not a plan. Nothing is built, and nothing should be until the ask below
has an answer.

## The short version

**One primitive is worth offloading: the high-precision reference orbit.** Everything else on the
menu is either something we already do well, or something that would amount to "run their
renderer instead of ours", which is not an integration.

The case rests on four things being true at once, and they are:

1. It is our dominant cost at extreme depth.
2. It is a pure function of its inputs, with no coupling to their renderer or ours.
3. It is coarse-grained — seconds of work per call — so process-boundary latency is irrelevant.
4. It is the exact thing they have engineered a large GPU win on.

It also has a real ceiling, stated in "What would stop this being worth it" below. The
recommendation is to ask, cheaply, and not to build before the answer.

## What FractalShark has that we do not

From their README and the 0.543 release:

- **A GPU high-precision reference orbit.** A persistent CUDA kernel computing the orbit in
  arbitrary precision, claimed at 16384 32-bit limbs (~158,000 decimal digits) to be
  "approximately 10× faster" than a multithreaded CPU implementation on an RTX 4090. This is the
  interesting one.
- `HpSharkFloat`, the arbitrary-precision numeric core underneath it.
- Two CUDA implementations of linear approximation (LAv2), ported from FractalZoomer.
- `HDRx32`, a 2×32-plus-exponent type giving ~48-bit mantissa without native f64.

We have our own answers to the last three. Our BLA is measured and tuned, our floatexp and df32
paths are ours, and swapping someone else's iteration-skip scheme into our shader would be a
rewrite for an unmeasured gain. The orbit is different: it is the one place where they have
attacked a cost that dominates us and we have no GPU answer at all.

## What the orbit actually costs us

`--bench-bignum` on the dev box (RTX 3080 / Ryzen), reference-orbit cost per iteration:

| precision (bits) | decimal digits | astro-float | rug/MPFR | MPFR speedup |
|---|---|---|---|---|
| 64 | 19 | 672 ns | 153 ns | 4.40× |
| 1088 | 327 | 1691 ns | 721 ns | 2.34× |
| 8256 | 2485 | 32.5 µs | 7.79 µs | 4.17× |

And from `--benchmark-std`: **88% of CPU time in floatexp frames is the reference build.**

Extrapolating the 8256-bit row, a 1e4000 view (about 13,300 bits) costs roughly 15 µs per
iteration, so a 440,000-iteration orbit is several seconds — consistent with the 5.2 s we measure
for the Misiurewicz solve at that depth. Past 1e20000 an orbit is tens of seconds to minutes.

**That is the regime where a 10× GPU orbit changes what the program can do**, rather than merely
making it quicker. Below roughly 1e1000, orbit builds are sub-second and already 2.3–5.6× better
on MPFR; there is nothing here worth an external dependency.

## Why the process boundary is the whole point

FractalShark is **GPL-3**. Fractadyne is **MIT OR Apache-2.0**. Linking their code into ours is
not available to us, and would not be even if we wanted it.

A separate executable talking over a pipe is a different matter, and it is a shape this project
already uses deliberately: we link GMP and MPFR **dynamically** precisely so the LGPL obligations
stay obligations about notices rather than about relinkable builds, and
`scripts/build-accelerated.ps1` documents that as a design decision, not an accident.

So the 0.543 client/server is not merely convenient — **it is the only architecture in which this
integration is possible at all.** We would ship no GPL code. The user installs FractalShark
themselves, points us at the executable, and we speak to it the way we already speak to an
optional MPFR. That precedent matters: `--bignum auto|astro|rug` already establishes "an optional
faster arithmetic provider, off by default, byte-identical output", and this would be a third
entry in that list.

## What we would need from them

Today the CLI returns a **PNG**. For this we need it to return **data**. The ask is small and
sharply bounded:

> An option that computes a reference orbit for a given centre, precision and iteration budget,
> and writes the samples out in a documented binary layout — rather than rendering an image.

Concretely, what would make it usable:

- **Inputs:** centre as decimal strings (as `--center-x` / `--center-y` already are), a precision
  in bits, and a maximum iteration count.
- **Output:** a header (sample count, precision, whether and where the orbit escaped) followed by
  the samples. Per sample, real and imaginary parts in a form that survives the exponent range —
  an (f64 mantissa, i32 exponent) pair each is exactly what we consume, and is what any
  perturbation renderer needs.
- **A determinism statement:** is this the orbit of that centre at that precision, and what is
  the rounding? We do not need bit-identity with our own; we need to know what we are getting.

That endpoint is generically useful to any perturbation renderer, not bespoke to us, which makes
it a better thing to ask for than something shaped only like our internals.

## What would stop this being worth it

Stated plainly, because these are the reasons it might be a bad idea:

- **CUDA/NVIDIA only.** Our renderer is wgpu and deliberately cross-vendor; the second machine we
  test on is a Radeon, where this would do nothing. It benefits a subset of users.
- **It cannot feed the validation corpus.** Our two bignum backends are **byte-identical** — all
  eight precision rows in the table above — and we gate on that. A third-party orbit almost
  certainly would not be, so it could serve the live and export paths but never the goldens or
  the corpus. That is a permanent split in the code, not a temporary one.
- **A 412 MB external dependency** and a new failure surface (process lifetime, pipe errors,
  version skew) for a win that only appears past ~1e1000.
- **We would be asking someone to maintain an API for us.** The 0.543 CLI already came from one
  of our requests. A second, larger ask should be proportionate, and should be framed as "useful
  to others too", because it is.

## Recommendation

**Ask, do not build.** The conversation is cheap, the developer has already shown he will add a
CLI surface on request, and the endpoint is independently useful. If he is interested, the first
use should be the *benchmark*, not the renderer: have the bench kit compare orbit-build cost head
to head, which is zero-risk, directly improves the comparison we already publish, and tells us
what the real speedup is on our hardware rather than the claimed one on an RTX 4090.

Only if that measurement holds up is a live integration worth designing — and then as an optional
provider beside MPFR, off by default, live and export paths only.

Draft message: `local/messages/fractalshark-orbit-ask.md` (⚠`local/` is gitignored and not backed
up by a push — copy it somewhere durable before relying on it).
