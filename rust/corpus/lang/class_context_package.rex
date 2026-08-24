/* Task 21: `RexxContext~package`, the one route to the running program's own
   package object.

   The identity rows are the point. A build that minted a fresh Package
   object per send would answer `The Package class` to the first row and `a
   Package` to nothing at all, so the class row alone cannot see it; the
   identity rows can. `~name` is deliberately absent: it answers this file's
   own absolute path, which is not a fixed string.

   `~identityHash` is compared with `==` and not with `=`. The oracle's
   answer is an address-derived integer of more than nine digits, and `=`
   compares two of those at NUMERIC DIGITS, which two nearby addresses pass. */
say .context~package~class
say (.context~package~identityHash == .context~package~identityHash)
say (.context~package~identityHash == .K~package~identityHash)
say (.context~package~identityHash == .Array~package~identityHash)
say .Array~package~name

::class K public
