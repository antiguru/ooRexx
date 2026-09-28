-- PARSE assigning into message terms: the setter is sent with the piece.
o = .k~new
parse value 'a b c' with o~x o~y .
say o~x '|' o~y
d = .directory~new
parse value 'p q' with d['A'] d~b
say d['A'] d~b
parse version o~x
say o~x~word(1)
say .k~new~m
trace i
parse value 'a b c' with o~x o[1, 2] .
trace r
parse value 'p q' with o~y o~x
parse value 'r' with o~z
trace i
parse value 'r s' with o~z o~x
trace n
say o~x o~y o~z
::class k
::attribute x
::attribute y
::method m
  parse value 'm n' with self~x self~y
  return self~x self~y
::method '[]='
  use arg v, i, j
  say 'put' v i j
::method 'z='
  use arg v
  say 'z got' v
  return 'zret'
::method z
  return 'zz'
