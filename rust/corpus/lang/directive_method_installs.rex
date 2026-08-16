/* A ::METHOD directive under a ::CLASS installs, its body stored and never
 * entered. Phase 5a Task 4.
 *
 * Message dispatch (Task 5) and Rexx method-body invocation (Task 7) are
 * later work, so this program never sends BAR to anything -- it only proves
 * that a ::CLASS carrying a ::METHOD installs cleanly, matching the oracle.
 * Measured, rc 0, stdout "main ran".
 */

say 'main ran'

::class Foo
::method bar
  return 1
