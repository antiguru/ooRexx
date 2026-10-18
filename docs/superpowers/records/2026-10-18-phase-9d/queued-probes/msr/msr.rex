m = .t~new~start('boom')
signal on syntax name s
say m~result
exit
s: do l over condition('O')~traceback; say 'tb:' l; end
say 'code' condition('O')~code
::class t
::method boom
  return 1/0
