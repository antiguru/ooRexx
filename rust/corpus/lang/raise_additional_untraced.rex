/* Untraced ADDITIONAL renders nothing for a trace line. A SYNTAX code uses
   the RC value's string value. */
o = .bad~new
q = .noisy~new
call on user zz name u
signal on syntax name s1
raise syntax 93.900 additional (o)
s1: say 's1' condition('o')~code
signal on syntax name s2
raise syntax (o)
s2: say 's2' condition('o')~code condition('o')~message
call ru 'additional', o
say 'after o'
signal on syntax name s3
raise syntax 93.900 additional (q)
s3: say 's3' condition('o')~code
call ru 'additional', q
say 'after q'
exit
u: say 'USER'; return
ru:
  parse arg how
  raise user zz additional (arg(2)) return
::class bad
::method defaultname
  raise syntax 98.900 array('boom')
::class noisy
::method defaultname
  say '    (noisy dn sent)'
  return 'NOISY'
