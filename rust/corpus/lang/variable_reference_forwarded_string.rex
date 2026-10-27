/* In an arithmetic operator's right operand, the right operand of = and
   NUMERIC DIGITS, a variable reference's string value is its referenced
   value's makeString answer, and where that value has none, NOSTRING names
   the referenced value and its STRING answer. */
m = .knum~new; n = .nstr~new; d = .directory~new
say 'arith' (1 + (>m)) (2 * (>m)) (7 // (>m))
numeric digits (>m)
say 'digits' digits()
numeric digits 9
call try 'say 1 + r', n
call try 'say 1 + r', d
call try 'say "x" = r', n
exit
try: procedure
  use arg code, value
  r = >value
  signal on nostring
  interpret code
  return
nostring:
  say 'NOSTRING' condition('d') condition('o')~additional~class~id
  return
::class knum
::method makestring; return '5'
::class nstr
::method string; return 'nstr'
