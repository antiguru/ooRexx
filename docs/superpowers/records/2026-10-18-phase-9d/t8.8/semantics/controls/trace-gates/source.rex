o = .key~new
trace off
v = o
say 'off' o~count
trace r
v = o
trace off
say 'result' o~count
trace i
s.tail = o
trace off
say 'compound' o~count
trace i
s. = o
trace off
say 'stem' o~count
exit
::class key
::method init
  expose n
  n = 0
::method count
  expose n
  return n
::method defaultname
  expose n
  n = n + 1
  return 'NAME'
