say 'main'
r = .Routine~new("rn", ("say 'body'", "::class c"," ::options trace r"))
say 'after new'
r~call
m = .Method~new("mn", ("say 'mbody'", "::class c2"," ::options trace l"))
say 'after mnew'
p = .Package~new("pn", ("say 'pbody'", "::options trace r"))
p = .Package~new("pn2", ("say 'pbody'", "::class c3", "::options trace r noprolog"))
p = .Package~new("pn3", ("::class c3", "::options trace r"))
p = .Routine~new("rn2", ("say 'r2body'", " ::options trace r"))
p~call
say 'nodirectives done'
r = .Routine~new("rn3", ("say 'body'", "::requires 'req2.rex'"," ::options trace r"))
call req1.rex
say 'end'
::requires 'req2.rex'
