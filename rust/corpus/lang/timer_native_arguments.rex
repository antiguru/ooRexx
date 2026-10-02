/* The timer natives convert their arguments as the oracle's signatures do:
   wholenumber_t, POINTER, and the count. */
o = .sub~new
call t "o~stop1"
call t "o~stop2"
call t "o~start1"
call t "o~start2"
call t "o~start3"
call t "o~start4"
call t "o~start5"
call t "o~direct"
exit
t:
  signal on syntax name err
  interpret 'say' arg(1)
  return
err:
  c = condition('o')
  say arg(1) '->' c~code c~message
  return
::class sub subclass alarm
::method init
::method stop1;  return self~!stopTimer(5)
::method stop2;  return self~!stopTimer()
::method start1; return self~!startTimer('x', 1)
::method start2; return self~!startTimer(1)
::method start3; return self~!startTimer(1, 2, 3)
::method start4; return self~!startTimer(0, 1.5)
::method start5; return self~!startTimer(0, 1e30)
::method direct
  t0 = time('e')
  r = self~!startTimer(0, 200)
  return r (time('e') - t0 >= 0.15)
