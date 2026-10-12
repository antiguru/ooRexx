/* .Package~defaultOptions(name, value): the override settings and the count.
   While the count is not zero, every compile of a package takes the override
   for each option its own ::OPTIONS did not name, and the count moves one
   down: a called file, a loaded package, Routine~new, Method~new,
   Package~new, a DEFINE from source, and the compile the D write itself
   makes. INTERPRET compiles no package. A count of 1 is what `rexx -o` sets,
   the next program alone; -1 is `rexx -od`, every one. */
parse source . . me
dir = left(me, lastpos('/', me))
cls = .Package
say 'D [' || cls~defaultOptions('D') || ']'
say 'C' cls~defaultOptions('C')
override = '::options digits 6 form engineering fuzz 1 trace e',
  'novalue syntax lostdigits syntax noprolog'
say 'set D [' || cls~defaultOptions('D', override) || ']'
say 'D [' || cls~defaultOptions('D') || ']'
say 'count 0 routine:' .routine~new('r0', 'return digits() form() fuzz() trace()')~call
say 'set C' cls~defaultOptions('C', 1)
say 'called:' called()
say 'C' cls~defaultOptions('C')
say 'called again:' called()

say 'set C' cls~defaultOptions('countOverrides', -1)
r = .routine~new('r1', 'return digits() form() fuzz() trace()')
say 'routine:' r~call 'C' cls~defaultOptions('C')
say 'routine I [' || r~package~options('I') || ']'
say 'routine [' || r~package~options || ']'
r = .routine~new('r2', ('return digits() form() fuzz() trace()', '::options digits 11 trace c'))
say 'routine naming DIGITS and TRACE:' r~call r~package~options('X') 'C' cls~defaultOptions('C')
m = .method~new('m', 'return digits()')
say 'method C' cls~defaultOptions('C')
.k~define('d', 'return digits() form()')
say 'define:' .k~new~d 'C' cls~defaultOptions('C')
pk = .Package~new('pk', 'return')
say 'package:' pk~digits pk~form pk~fuzz 'C' cls~defaultOptions('C')
req = .context~package~loadPackage(dir || 'package_default_options_req.cls')
say 'loaded:' req~digits req~form req~options('P') 'C' cls~defaultOptions('C')
interpret 'say "interpret:" digits()'
say 'C' cls~defaultOptions('C')
call try ".routine~new('nv', 'return zzunset')~call"
call try ".routine~new('ld', 'return 1234567 + 1')~call"

say 'D again [' || cls~defaultOptions('D', '::options digits 8') || ']'
say 'C after D' cls~defaultOptions('C')
say 'D now [' || cls~defaultOptions('D') || ']'
say 'set C' cls~defaultOptions('C', 0)
say 'D at count 0 [' || cls~defaultOptions('D', '::options digits 8') || ']'
say 'D now [' || cls~defaultOptions('D') || ']'
say 'C' cls~defaultOptions('C')
say 'set C' cls~defaultOptions('C', 2)
np = .context~package~loadPackage(dir || 'package_default_options_noprolog.cls')
say 'NOPROLOG package:' np~options('P') np~options('X') np~digits
say 'C' cls~defaultOptions('C', ' 0 ')

call try "cls~defaultOptions('D', 'digits 5')"
call try "cls~defaultOptions('D', '::options digits 5;')"
call try "cls~defaultOptions('D', '')"
call try "cls~defaultOptions('D', .nil)"
call try "cls~defaultOptions('C', 1.5)"
call try "cls~defaultOptions('C', 'x')"
call try "cls~defaultOptions('A', 1)"
say 'D [' || cls~defaultOptions('D') || ']'
say 'C' cls~defaultOptions('C')
exit

called:
  call (dir || 'package_default_options_called.cls')
  return result

try:
  signal on syntax name refused
  interpret 'say' arg(1)
  return
refused:
  c = condition('o')
  say '  ' arg(1) '->' c~code c~errortext c~message
  return

::class k
