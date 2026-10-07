/* DO WITH's failures: no SUPPLIER method, a SUPPLIER answer that is not a
   supplier, and an AVAILABLE that is not logical; then COUNTER, WHILE and
   UNTIL beside the supplier. */
signal on syntax name s1
do with index i over 5; say i; end
s1: say 'A' condition('o')~code condition('o')~message
signal on syntax name s2
do with index i over .x~new; say i; end
s2: say 'B' condition('o')~code condition('o')~message
signal on syntax name s3
do with index i over .z~new; say i; end
s3: say 'C' condition('o')~code condition('o')~message
do counter c with item v over .directory~new; end
say c
q = .queue~of('p', 'q')
do counter c with index i item v over q while c < 1; say c i v; end
say c i v
do counter c with index i item v over q until c > 0; say c i v; end
say c i v
do with index i item v over q until i > 0; say i v; end
say i v
::class x
::method supplier
  return 'nope'
::class z
::method supplier
  return .w~new
::class w subclass supplier
::method init
  self~init:super(.array~of('a'), .array~of(1))
::method available
  return 'maybe'
