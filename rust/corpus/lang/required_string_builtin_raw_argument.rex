/* VALUE's new value is the one argument position in the whole builtin set that
 * the oracle fetches raw. `BUILTIN(VALUE)` reads it with `optional_argument`,
 * which is `stack->peek` and converts nothing, where its name and its selector
 * go through the converting accessors -- so the object is *stored*, and the
 * required-string protocol runs later, wherever something renders it.
 *
 * Both halves are here because either alone passes a build that converts at
 * the call: the ordering shows the conversion happening after the assignment,
 * and the class shows an object came back out of the variable rather than a
 * string.
 *
 * The last rows are the same position under a live NOSTRING trap: a converted
 * position raises there and this one does not. They read the stored value with
 * ~isA so that nothing on those lines renders an object and raises for an
 * unrelated reason.
 *
 * **They say nothing about how the latch got armed**, and an earlier version of
 * this comment claimed they did. The makeString directives below arm it at
 * install, before the first clause runs, so the SIGNAL ON here changes nothing
 * about that. Ruled by inverting the interpreter's own trap-arming site: this
 * program still matches the oracle with it disabled.
 * required_string_nostring_no_directive.rex declares nothing and is what
 * witnesses that site.
 *
 * Phase 5a Task 14, fix round 1; the arming claim corrected in fix round 2.
 */

say 'before'
r = value('a', .K)
say 'after'
say 'class' a~class
say 'stored' a
say 'again' a

/* The name position is a converting one, which is what says the exemption is
   one position of one builtin and not the whole call. */
say 'name' value(.name)

signal on nostring name noStr
say 'trap armed'
r2 = value('b', .Array)
say 'no raise at the call'
say 'b is a class' b~isA(.Class)
say 'b is a string' b~isA(.String)
signal off nostring
say 'done'
exit 0

noStr:
say 'NOSTRING raised where the oracle stores the object'
exit 1

::class K
::method makeString class
  say 'K asked'
  return 'converted'

::class name
::method makeString class
  return 'zz'
