call on user foo name h
x = rr2()
say 'after rr2' x
x = rr3()
say 'after rr3' x
exit
h: say 'main handler' condition('C'); return
::routine rr2
call on user foo name h2
interpret 'raise user foo return 5'; say 'no'
return 9
h2: say 'rr2 handler'; return
::routine rr3
interpret 'raise user foo return 6'; say 'no'
return 9
