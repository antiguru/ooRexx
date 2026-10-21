c = .counter~new
.local~c = c
hits = 0
i = 0
top:
if i > 0, c~n > b then hits = hits + 1
i = i + 1
o = .u~new
o = .nil
if i < 3000 then if i < 3000 then do; z = copies("x", 20000) || i; b = c~n; signal top; end
say 'drained at the signal' (hits > 0)
::class counter
::attribute n
::method init; expose n; n = 0
::method bump; expose n; n = n + 1
::class u
::method uninit
  .local~c~bump
