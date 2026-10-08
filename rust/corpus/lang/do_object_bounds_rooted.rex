/* A loop driven by message keeps its TO and BY alive across passes whose
   + and comparison allocate: a numeric loop whose control becomes an
   object, with a TO and BY that are not small integers, and a header whose
   BY answer is an object. */
c = 0
do n = 1 to 1234567.5 by 1.25
  c = c + 1
  if n > 3 then n = .k~new(10)
  if c > 6 then leave
end
say 'end' n c
do i = .k~new(1) to 9.5 by .k~new(2.5); say 'i' i; end
say 'end' i
::class k
::method init; expose v; use arg v
::method v; expose v; return v
::method '+'
  expose v
  junk = .array~new(50)~fill('x')
  if arg() = 0 then return self
  use arg o
  if o~isA(.k) then o = o~v
  return .k~new(v + o)
::method '>'; expose v; use arg o; junk = .array~new(50)~fill('y'); return v > o
::method '<'; expose v; use arg o; return v < o
::method string; expose v; return 'k'v
