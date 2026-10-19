.local~h = .helper~new
signal on syntax
g = .routine~new('g', .array~of("say .h~f 'tail'", '::options trace r'))
g~call
exit
syntax:
x = .traceOutput~destination
say 'syntax' condition('o')~code
exit
::class helper
::method f
  .traceOutput~destination("")
  return 5
