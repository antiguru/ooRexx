/* ::ROUTINE dispatch, and the >I>/<I< pair no committed expectation can hold.
 *
 * WHY THIS PROGRAM IS IN THE LIVE CORPUS AND NOT IN tests/trace_oracle/.
 * The >I>/<I< lines name the package by its ABSOLUTE PATH, so a captured
 * .expected file would be true on one machine and false on the next. This
 * harness runs both interpreters on the same path in the same run, so the
 * two agree without either side being committed. Block E is the only place
 * in the suite where those two prefixes are compared against the oracle at
 * all.
 *
 * WHAT EACH BLOCK WOULD PRINT IF THE ENGINE WERE WRONG.
 *
 * A -- the resolution order, internal label then builtin then ::ROUTINE.
 *      Every name here resolves to SOMETHING, so a wrong order is a wrong
 *      answer rather than a failure. A ::ROUTINE search in front of the
 *      builtin step prints ROUTINE-LENGTH for line 1; in front of the label
 *      search it prints ROUTINE for line 2; and a quoted target that still
 *      searched labels would print LABEL for line 3.
 *
 * B -- the lookup upcases both sides where the builtin lookup does not. A
 *      case-sensitive routine lookup fails line 1 with 43.1; a
 *      case-insensitive BUILTIN lookup sends line 3 to MAX and prints 9.
 *
 * C -- a ::ROUTINE's own variable pool, against a CALLed label's shared one.
 *      A routine sharing the caller's pool prints CALLER for its first line
 *      and ROUTINE-WROTE for the caller's second.
 *
 * D -- EXIT inside a ::ROUTINE ends the routine, not the program. An engine
 *      treating it as a program exit prints nothing after line D1.
 *
 * E -- >I>/<I<, on stderr. The routine's own TRACE LABELS is the only route
 *      4c implements (::OPTIONS TRACE LABELS is the other and is a declared
 *      Phase 5 gap). rtn_traced's TRACE must stay its FIRST instruction: a
 *      clause in front of it, a label included, suppresses both lines on the
 *      oracle. rtn_quiet beside it is the neighbouring silent case, so this
 *      block also pins that the announcement is per activation and not per
 *      program.
 *
 * The program prints no path of its own on stdout: the only absolute path
 * anywhere in its output is the one the oracle itself puts in the >I>/<I<
 * lines.
 */

say '-- A'
call length 'abcd'
say result
call zorkolo
say result
call 'ZORKOLO'
say result

say '-- B'
call 'ZORK'
say result
call zork
say result
call 'max' 1, 9
say result

say '-- C'
vv = 'CALLER'
call rtn_pool
say vv
call lbl_pool
say vv

say '-- D'
call rtn_exit
say result
say 'D1'

say '-- E'
call rtn_traced
call rtn_quiet
say result
exit

zorkolo:
  return 'LABEL'

lbl_pool:
  say vv
  vv = 'LABEL-WROTE'
  return

::routine zorkolo
  return 'ROUTINE'

::routine length
  return 'ROUTINE-LENGTH'

::routine 'zork'
  return 'HIT'

::routine 'max'
  return 'ROUTINE-lower-max'

::routine rtn_pool
  say vv
  vv = 'ROUTINE-WROTE'
  return

::routine rtn_exit
  exit 'EXITED'

::routine rtn_traced
  trace l
  inner_label:
  return

::routine rtn_quiet
  return 'QUIET'
