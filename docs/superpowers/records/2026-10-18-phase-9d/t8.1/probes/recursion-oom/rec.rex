d = .dst~new
.traceoutput~destination(d)
trace r
x = 1
.traceoutput~destination
say 'after'
::class dst
::method lineout; call spin
::routine spin; call spin
