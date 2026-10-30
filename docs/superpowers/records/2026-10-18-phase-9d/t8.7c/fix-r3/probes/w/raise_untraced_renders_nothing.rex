/* An untraced RAISE renders its DESCRIPTION and ADDITIONAL values for no
   trace line, so it sends nothing to them; a traced one renders each. A
   SYNTAX code is the RC value's string value. */
o = .bad~new
q = .noisy~new
call on user zz name u
signal on syntax name s1
raise syntax 93.900 additional (o)
s1: say 's1' condition('o')~code
signal on syntax name s2
raise syntax (o)
s2: say 's2' condition('o')~code condition('o')~message
signal on syntax name s5
raise syntax 93.900 description (o)
s5: say 's5' condition('o')~code
call ru 'description', o
call ru 'additional', o
say 'after o'
signal on syntax name s3
raise syntax 93.900 additional (q)
s3: say 's3' condition('o')~code
call ru 'description', q
call ru 'additional', q
say 'after q'
call ru 'trace description', q
signal on syntax name s4
call ru 'trace description', o
s4: trace off; say 's4' condition('o')~code condition('o')~message
exit
u: say 'USER'; return
ru:
  parse arg how
  select
    when how = 'description' then raise user zz description (arg(2)) return
    when how = 'additional' then raise user zz additional (arg(2)) return
    otherwise trace r; raise user zz description (arg(2)) return
  end
::class bad
::method defaultname
  raise syntax 98.900 array('boom')
::class noisy
::method defaultname
  say '    (noisy dn sent)'
  return 'NOISY'
