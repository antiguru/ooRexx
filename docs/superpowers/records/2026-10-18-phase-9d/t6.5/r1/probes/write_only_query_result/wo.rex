call on notready name caught
f = 'wo.txt'
call lineout f, 'line1'; call lineout f, 'line2'; call lineout f
'chmod a+w,a-r' f
call t "stream(f, 'c', 'open write')"
call t "chars(f)"
call t "lines(f)"
call t "lines(f, 'N')"
call t "charin(f)"
call t "linein(f)"
call t "stream(f, 'c', 'query position write')"
call t "stream(f, 'c', 'query position write line')"
call t "stream(f, 'c', 'query position read line')"
call t "lineout(f, 'x', 2)"
call t "stream(f, 'c', 'query position write')"
call t "lineout(f, 'y')"
call t "stream(f, 'c', 'query position write')"
call t "stream(f, 'c', 'seek =1 write line')"
call t "stream(f, 'c', 'seek =5 write char')"
call t "charout(f, 'z')"
call t "stream(f, 'c', 'close')"
'chmod a+r' f
say c2x(charin(f, 1, 40))
exit
t:
  hits = ''
  interpret 'r =' arg(1)
  say arg(1) '[' || r || ']' stream(f, 'd') hits
  return
caught:
  hits = 'raised [' || condition('O')~result || ']'
  return
