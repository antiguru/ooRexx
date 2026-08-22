/* A refused PRIVATE send untrapped, so the traceback bytes are compared and
 * not only the condition number.
 *
 * The sender is a sibling class in the same package: a class object whose own
 * hierarchy does not contain the scope that declared the method, which is the
 * refusing side of the limb subexplicit exercises in method_access_private.
 * Two frames, the inner `return .K~m` before the outer `say .S~poke`, and the
 * report is 97.2 rather than 97.1 -- a build that treated the refusal as
 * "name not found" would print the same first line and the wrong second one.
 *
 * rc 159.
 */

say .S~poke

::CLASS K
::METHOD m CLASS PRIVATE
  return 'inner'

::CLASS S
::METHOD poke CLASS
  return .K~m
