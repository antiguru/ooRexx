/* The NOSTRING half of the required-string protocol with **no directive
 * anywhere in the file**, which is what makes this program's own witness the
 * trap and nothing else.
 *
 * required_string_nostring.rex covers the same condition, but it declares a
 * class with a makeString, so a build could reach every one of its rows for a
 * reason that has nothing to do with the trap. This file declares nothing, so
 * the only thing that can turn an object with no string value from a rendering
 * into a raise is the SIGNAL ON itself. Verified by inverting: with the
 * interpreter's own trap-arming site disabled, this program diverges from the
 * oracle on stdout and on exit status while every other trap-armed program in
 * the corpus still matches.
 *
 * Both spellings are here because a trap table answers NOSTRING through its own
 * name and through ANY, and the fallback between them is a separate lookup.
 *
 * The last rows are VALUE's new value, the one argument position the oracle
 * fetches raw: a converted position raises under a live trap and this one does
 * not, so it belongs with the trap rows rather than with the makeString ones in
 * required_string_builtin_raw_argument.rex.
 *
 * Phase 5a Task 14, fix round 2.
 */

say 'untrapped' .array
say 'untrapped concat' 'x' || .environment

signal on nostring name byName
say 'armed by name'
say 'not reached' .array

byName:
say 'by name' condition('C') '|' condition('D')
signal off nostring
say 'disarmed' .array

signal on any name byAny
say 'armed by any'
say 'not reached' .environment

byAny:
say 'by any' condition('C') '|' condition('D')
signal off any

/* A raw argument position under a live trap: the oracle stores the object and
   raises nothing, and a build that converted it would raise here. */
signal on nostring name unreached
r = value('stored', .array)
say 'raw position stored' stored~isA(.Class)
say 'raw position is not a string' stored~isA(.String)
signal off nostring
say 'done'
exit 0

unreached:
say 'a raw argument position was converted'
exit 1
