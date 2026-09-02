/* Class-side method-dispatch dimension: one class-method send per pass and
 * nothing else in the loop.
 *
 * dispatch.rex is the same dimension reached through an instance. The two
 * are not interchangeable: an instance send resolves against the instance
 * behaviour and reaches an exposed variable pool, where this one resolves
 * against the class behaviour and does neither.
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
