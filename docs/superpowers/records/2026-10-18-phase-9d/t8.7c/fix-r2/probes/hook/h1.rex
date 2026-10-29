/* A condition a traced rendering raises is the clause's: through PARSE's
   source and pattern, and an expression the tree evaluator runs. */
o = .bad~new
signal on syntax name s1
trace i
parse value o with a
trace off
say 'not reached 1'
s1: trace off; say 's1' condition('o')~code condition('o')~message
signal on syntax name s2
p = o
trace i
parse value 'xyz' with a (p) b
trace off
say 'not reached 2'
s2: trace off; say 's2' condition('o')~code condition('o')~message
signal on syntax name s3
trace i
interpret 'x = o'
trace off
say 'not reached 3'
s3: trace off; say 's3' condition('o')~code condition('o')~message
exit
::class bad
::method defaultname
  raise syntax 98.900 array('boom')
