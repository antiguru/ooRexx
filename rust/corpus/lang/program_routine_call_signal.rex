/* The program's own Routine: SYNTAX trapped inside the re-run, and one
 * reaching the caller's trap. */
if arg(1) = 's' then signal sbody
if arg(1) = 'u' then return 1 + 'abc'
r = .context~executable
signal on syntax
say r~call('s')
say r~call('u')
say 'not reached'
exit
syntax: say 'outer syntax' rc condition('o')~traceback~items
exit
sbody:
signal on syntax name inner
x = 1 + 'a'
inner: return 'trapped inside' rc
