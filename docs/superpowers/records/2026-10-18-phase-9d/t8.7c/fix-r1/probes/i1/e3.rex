/* DEFAULTNAME raising, rendered by a trace line */
o = .bad~new
trace i
x = o
trace off
say 'after trace'
::class bad
::method defaultname
  raise syntax 98.900 array('boom')
