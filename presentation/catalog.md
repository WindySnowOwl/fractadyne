# Challenge coordinates — verify these yourself

Every entry has a **known, externally-checkable answer** — a hyperbolic component's exact
period + nucleus, or a yes/no set membership. Confirm each with any trusted renderer, by hand,
or with your own arbitrary-precision code. Fractadyne's `--selftest` verifies the build against
all of them automatically; the full machine-readable list is
[`validation/catalog.toml`](../validation/catalog.toml).

If any answer here is wrong, I want to know.

## Minibrot nuclei — Newton-snap from `center` should reach `nucleus` and report `period`

| feature                                 | approach center                       | period | exact nucleus                                                                        |
| --------------------------------------- | ------------------------------------- | ------ | ------------------------------------------------------------------------------------ |
| period-2 disk (c = −1)                  | −1.001, 0.001                         | 2      | −1.0, 0.0                                                                            |
| period-3 bulb nucleus                   | −0.121, 0.745                         | 3      | −0.1225611668766536, 0.7448617666197446                                              |
| period-3 antenna minibrot               | −1.7549, 0.0001                       | 3      | −1.7548776662466927, 0.0                                                             |
| period-4 window nucleus                 | −1.3107, 0.0001                       | 4      | −1.3107026413368328, 0.0                                                             |
| period-998 minibrot (Seahorse V., 2e7×) | −0.743643887037151, 0.131825904205330 | 998    | −0.74364388703715887077806454349323251348, 0.131825904205312292821097354874199108694 |

## Set membership — is the point inside the Mandelbrot set?

| point                                                                                   | inside the set?             |
| --------------------------------------------------------------------------------------- | --------------------------- |
| c = −0.5 (main cardioid)                                                                | yes                         |
| c = 1                                                                                   | no                          |
| −0.74364388703715887077806454349323251348 + 0.131825904205312292821097354874199108694 i | yes (deep minibrot nucleus) |

These are verified independently in the suite by (a) the arbitrary-precision CPU dwell oracle and
(b) a 1×1 GPU render — two code paths that share nothing.

**Want more?** Send me a coordinate whose answer you know (period/nucleus or membership) and I'll
run it through the same machinery and report back.
