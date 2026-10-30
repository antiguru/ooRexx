/* A reference's required-string failure stays in its clause. Untraced
   ERROR RC and USER ADDITIONAL do not send a rendering conversion. */
b1 = .once~new
q1 = >b1
signal on syntax name s1
numeric digits (q1)
say 'returned 1'
s1: say 's1' condition('o')~code condition('o')~message
b2 = .once~new
q2 = >b2
signal on syntax name s2
signal value q2
say 'returned 2'
s2: say 's2' condition('o')~code condition('o')~message
nz = .noisy~new
bd = .bad~new
call on user zz name u
call on error name u
signal on syntax name s3
call ru 'error', >nz
say 'after nz error'
call ru 'error', >bd
say 'after bd error'
call ru 'additional', >nz
say 'after nz additional'
call ru 'additional', >bd
say 'after bd additional'
raise syntax (>bd)
say 'returned 3'
exit
s3: say 's3' condition('o')~code condition('o')~message; exit
u: say 'handler' condition('c'); return
ru:
  use arg how, v
  if how = 'additional' then raise user zz additional (v) return
  raise error (v) return
::class once
::method init; expose n; n = 0
::method defaultname; expose n
  n = n + 1
  say '    (dn' n')'
  if n = 1 then raise syntax 98.900 array('boom')
  return 'ONCE'
::class noisy
::method defaultname
  say '    (noisy dn)'
  return 'NOISY'
::class bad
::method defaultname
  say '    (bad dn)'
  raise syntax 98.900 array('boom')
