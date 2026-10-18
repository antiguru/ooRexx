call on notready name nr
address system 'rm -f wfifo; mkfifo wfifo'
n = 'wfifo'
last = ''
r = lineout(n, 'x');           call t 'LINEOUT x', r
r = charout(n, 'y');           call t 'CHAROUT y', r
r = chars(n);                  call t 'CHARS', r
r = lineout(n, 'z', 1);        call t 'LINEOUT z 1', r
r = lineout(n);                call t 'LINEOUT close', r
r = stream(n, 'C', 'OPEN WRITE'); call t 'OPEN WRITE', r
r = stream(n, 'C', 'QUERY SIZE'); call t 'QSIZE', r
r = stream(n, 'C', 'CLOSE');   call t 'CLOSE', r
exit
t:
  say left(arg(1), 12) '=' arg(2) '| S' stream(n, 'S') '| D' stream(n, 'D') '|' last
  last = ''
  return
nr:
  o = condition('o')
  last = 'NR d=' condition('d') 'add=' o~additional~class 'res=' o~result 'inst=' o~instruction 'prop=' o~propagated
  return
