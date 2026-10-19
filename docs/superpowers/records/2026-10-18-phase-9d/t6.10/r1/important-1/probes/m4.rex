signal on syntax
.traceOutput~destination(.picky~new)
g = .routine~new('g', .array~of("say 'said'", '::options trace r'))
g~call
exit
syntax:
x = .traceOutput~destination
say 'syntax' condition('o')~code
exit
::class picky
::method lineout
  use arg t
  if pos('>>>', t~traceLine) > 0 then raise syntax 40.1 array('PICKY')
  return 0
