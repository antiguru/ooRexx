/* A `GUARD ... WHEN` expression that is exposed but not exactly `0` or `1`
   is 34.902, GUARD's own sub-number rather than IF's 34.1 or WHEN's 34.2.
   The value is checked before anything would wait on it, so the raise is
   the oracle's answer whatever a scheduler does with a false one. Phase 5a
   Task 16. */

say .K~m

::class K

::method m class
  expose v
  v = 'x'
  guard on when v
  return 'ran'
