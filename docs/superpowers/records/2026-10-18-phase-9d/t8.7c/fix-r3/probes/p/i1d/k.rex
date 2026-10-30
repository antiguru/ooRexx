o = .bad~new
r = >o
signal on syntax name s1
say 'arr' .array~of(r)~makestring
say 'not reached 1'
s1: say 's1' condition('o')~code condition('o')~message
signal on syntax name s2
say 'str' r~string
say 'not reached 2'
s2: say 's2' condition('o')~code condition('o')~message
signal on syntax name s3
trace i
z = r
trace off
say 'not reached 3'
s3: trace off; say 's3' condition('o')~code condition('o')~message
exit
::class bad
::method defaultname
  raise syntax 98.900 array('boom')
