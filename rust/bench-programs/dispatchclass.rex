/* Class-side method-dispatch dimension: one class-method send per pass and
 * nothing else in the loop.
 *
 * dispatch.rex is the same dimension reached through an instance, and it is
 * blocked on this crate: its body needs ~new. This one is reachable, so the
 * send path has an axis a guard sitting can read while that stays true. The
 * two are not interchangeable -- an instance send resolves against the
 * instance behaviour and this one against the class behaviour -- so this does
 * not replace dispatch.rex or relieve whichever task unblocks it.
 *
 * The body returns a constant rather than accumulating in the class, because
 * a class-scope instance variable would put a second unmeasured mechanism in
 * the loop.
 */
n = 4000000
total = 0
do i = 1 to n
    total = total + .Counter~bump
end
say total

::class Counter
::method bump class
  return 1
