/* setMethod's OBJECT scope shares the class's pool where FLOAT does not, so
   a method of the class reads back what an OBJECT-scope method wrote and
   sees nothing a FLOAT-scope method wrote. A second instance is here because
   the pool is per object either way. */
o = .k~new
o~go
say 'class-sees' o~peek
say 'float-sees' o~oneoff
o~go2
say 'object-sees' o~oneoff2
say 'class-sees2' o~peek
p = .k~new
p~go2
say 'fresh-class-sees' p~peek
say 'first-still' o~peek

::class k
::method init
  expose v
  v = 'class-v'
::method peek
  expose v
  return v
::method go
  self~setMethod('ONEOFF', 'expose v; return "["v"]"')
::method go2
  self~setMethod('ONEOFF2', 'expose v; v = "obj-v"; return "["v"]"', 'OBJECT')
