# The four measurement variants, kept because the measurement rests on them

Each patch applies to **`d6aec7d38`**, the store fusion, which is reverted from
the tree by `85992fd09` and stays in history as the instrument it was. The
figures in `2026-09-22-store-fusion-report.md` come from binaries built at that
commit with these applied.

| patch | what it builds | what it isolates |
|---|---|---|
| `noop.patch` | the four arms present, the fusion gated behind an environment variable, the driver identical to the instruction | **the arms alone.** `base` to this is the cost of adding arms with nothing fused; this to `d6aec7d38` is the fusion alone |
| `onearm.patch` | one arm instead of four | the first point of the arm ladder |
| `twoarm.patch` | two arms instead of four | the second point, which showed the cost is not monotone |
| `loadonly.patch` | the `LoadStore` shape only | the per-shape split of the price |

**Why these are committed and the rest of the run is not.** A `base` to `head`
A/B cannot price a fusion that also adds arms to the driver, and the proof is
that `emptyloop` moves -1.03% while removing three ops in the whole run. The
separation needs the gated build, so the gated build is part of the evidence
rather than a step along the way. The report's central claim -- that a four-op
difference between the two-arm and four-arm builds moves `varlookup` by
342,005,773 instructions -- cannot be re-derived without `twoarm.patch`.

They were in a session scratch directory on a tmpfs when this was written, which
is where another record on this branch was nearly lost the same week.
