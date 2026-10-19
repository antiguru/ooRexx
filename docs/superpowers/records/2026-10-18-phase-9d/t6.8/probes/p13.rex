.Routine~new("", "trace i; nop")~call
.Routine~new("rr", "trace i; nop")~call
.Routine~new("rr", "nop; ::options trace i")~call
o = .k~new
o~go
::class k
::method go
self~setMethod("a", .Method~new("", "trace i; nop"), "OBJECT")
self~setMethod("b", .Method~new("bb", "nop; ::options trace i"))
self~a
self~b
