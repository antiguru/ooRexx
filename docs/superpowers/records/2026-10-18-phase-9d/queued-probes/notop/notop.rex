t = .fwd~new(.true); f = .fwd~new(.false)
say (\.true) (\.false)
say (\t) (\f)
say (t & 1) (f & 1)
::class fwd
::method init; expose v; use arg v
::method unknown; expose v; use arg m, a; forward to (v) message (m) arguments (a)
