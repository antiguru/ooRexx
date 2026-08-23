/* .ROUTINES is the package's own routine table, reachable both by index and
 * as a message. Phase 5a Task 17.
 *
 * It is package->routines, which is both access scopes and not the public
 * ones alone, and it is keyed by the upcased name even where the directive
 * quoted a lower-case one -- which is what the third lookup on the first line
 * asks. A StringTable answers an entry name sent as a message exactly as a
 * Directory does, out of the same donated UNKNOWN.
 *
 * Measured, rc 0.
 */

say .routines~class
say .routines~r .routines["R"] .routines["r"]
say .routines["S"]
say .routines~q

::routine "r" public
  return 1

::routine s
  return 2
