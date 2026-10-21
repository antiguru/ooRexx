d = .dst~new
.traceoutput~destination(d)
a = .w~new~start('run', 'A')
b = .w~new~start('run', 'B')
ra = a~result
rb = b~result
.traceoutput~destination
say 'lines' d~count 'results' ra rb
::class dst
::attribute count
::method init; expose count; count = 0
::method lineout; expose count; use arg line; count = count + 1; call syssleep 0.01; return 0
::class w
::method run
  use arg tag
  trace r
  x = 1
  y = 2
  trace off
  return tag
