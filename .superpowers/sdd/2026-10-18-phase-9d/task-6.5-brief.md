# Task 6.5

* **Task 6.5: binary and write-only streams.** Clusters: `stream-binary-records`,
  `stream-open-write-only`, `stream-command-position`. `LINEOUT TEST_BINARY` gets a
  `Deviation 46` licence row only once every assertion before line 226 passes here.
  `stream-command-position` is unprobed and may split. The instance probes it first.

## The Task 6 steps
### Task 6 (template): one 9d set cluster

The controller writes each instance as `Task 6.<n>: <cluster>`, carrying the cluster's tests, its
groups, its likely files and any Review Focus line `clusters.md` marks for it. Each instance has
these steps:

- [ ] **Step 1:** Write one corpus witness per distinct cause, and show each red against the oracle.
      For the streams cluster, one witness uses a name holding a `'80'x` byte (Review Focus 3).
      For the traceback cluster, one witness covers each of INTERPRET, REPLY and a propagated
      condition (Review Focus 4).
- [ ] **Step 2:** Fix the crate. Where the oracle is wrong, stop and report. The controller takes the
      Deviation to Moritz before any licence row is written.
- [ ] **Step 3:** The witnesses match the oracle and the cluster's tests pass. The whole suite run
      shows no other changed row. Remove the cluster's lines from `suite-9d-set.txt`. Record the
      instance's disposition of any queued item or exclusion row it closes. Run the perf check
      where it applies, then commit.

