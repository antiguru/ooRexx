# Vendoring ootest into the tree -- DEFERRED, not done

**Status: not doing this now.** Moritz, 2026-08-28: "I feel it's too early to vendor. Let's skip."
Nothing below was carried out; `ootest/` stays a git-ignored svn working copy and the 5a
global-constraints rule to check it with `svn info` stands unchanged. The findings are kept because
they cost the investigation and hold whenever this is revisited.

Moritz, 2026-08-28, choosing between four readings of "adopt ootest": bring the working copy in as
tracked files at a recorded revision, instead of a git-ignored external svn checkout that every task
must re-verify. Fixes the fragility and prepares the later tree restructure. Neutral on gate speed.

## What is there

`ootest/` is `https://svn.code.sf.net/p/oorexx/code-0/test/trunk` (`^/test/trunk`) at **r13178**,
last changed r13169. 532 versioned files, 9.5 MB excluding `.svn/`. `svn status` reports **no local
modifications**, so the working copy is faithful to that revision. Licence is CPLv1.0, the project's
own. 409 `.testGroup` files, 354 of them under `base/`.

## Two hazards found before committing anything

1. **14 unversioned paths are test residue, not upstream content.** They are artifacts the C++
   oracle's own ooTest runs left behind, and a blind `git add ootest/` would commit them as though
   they came from r13178:

   ```
   ootest/address-with.tmp                       ootest/ooRexx/base/keyword/search_order
   ootest/initdata.txt                           ootest/ooRexx/base/keyword/search_order.cls
   ootest/ooRexx/base/bif/lineout                ootest/ooRexx/base/keyword/search_order.other
   ootest/ooRexx/base/class/tmpTest_ExternalCode_Compiled.rex
   ootest/ooRexx/base/class/tmpTest_ExternalCode_CompiledAndEncoded.rex
   ootest/ooRexx/base/class/tmpTest_ExternalCode_Source.rex
   ootest/ooRexx/base/rexxutil/platform/windows/test_sysini.ini
   ootest/ooRexx/base/rexxutil/sysfiletree                ootest/ooRexx/base/rexxutil/test_sysfile
   ootest/ooRexx/base/rexxutil/test_sysfile_dir           ootest/ooRexx/base/rexxutil/test_sysfile_readonly
   ```

   Vendor the versioned set only. The list is derived by `svn status ootest | grep '^?'`, and the
   exclusion belongs in `.gitignore` beside the vendored tree so a later run of the suite does not
   dirty the checkout.

2. **Several `.testGroup` files are deliberately binary, and 12 contain CR bytes.** `file` reports
   `DELWORD.testGroup`, `INSERT.testGroup`, `CONCATENATION.testGroup`, `MutableBuffer/delword` and
   others as binary: they test operations over strings like `'0A'x`. The extractors read these byte
   for byte, so any end-of-line normalisation silently moves every expectation derived from them.
   The tree has no `.gitattributes` today and `core.autocrlf` is unset, so nothing converts on this
   machine -- but nothing stops a different checkout from doing so either. Vendoring adds
   `ootest/** -text` so the bytes are pinned rather than left to a client's configuration.

## What the vendoring changes about the rules

`docs/superpowers/records/2026-08-17-phase-5a/global-constraints.md` tells every task to check
`ootest/` with `svn info` before trusting it, because it is machine state no file in the checkout
records. Once the content is tracked, that is no longer true for `ootest/`, and the constraint has to
say so rather than being left to contradict the tree. `oodocs/` is unchanged and keeps its rule.

## Steps

1. Wait for Phase 5b Task 0 to commit; it has five modified test files in flight.
2. `.gitattributes`: `ootest/** -text`.
3. `.gitignore`: drop the `ootest/` line, keep `ootest/.svn/` ignored, and add the 14 residue paths.
4. Add the 532 versioned files. Never `git add -A`; name `ootest` explicitly.
5. Record provenance in-tree beside the vendored root: URL, revision 13178, last-changed 13169, the
   date, and the `svn checkout`/`svn update` command that refreshes it.
6. Update the 5a global-constraints paragraph for `ootest/` only.

## Verification

* `svn status ootest`, filtered of `?` lines, stays empty after the commit -- what is tracked is
  r13178's content and nothing else.
* The tracked set equals the versioned set: `git ls-files ootest | wc -l` against
  `svn list -R ootest | grep -v '/$' | wc -l`, both 532.
* The gate is unaffected, because nothing moves: every harness reads `ootest/` by the same path.
  `assertions`, `bif_assertions` and `ir_dual` report the same headline counts before and after.
* A byte check on the binary groups: hash `CONCATENATION.testGroup`, `DELWORD.testGroup` and
  `INSERT.testGroup` before and after `git add`/checkout round trip, and confirm they are unchanged.
  That is the assertion behind hazard 2, and it fails loudly if `-text` is ever dropped.
