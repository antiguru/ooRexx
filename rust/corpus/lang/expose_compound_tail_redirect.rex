-- ADDRESS WITH reads and writes an exposed tail through the object's stem,
-- and an OUTPUT STEM it replaces loses the exposure as EMPTY does.
o = .t~new
o~run
o~show
o~run2
o~show
::class t
::method run
  expose out.1 in.2
  in.0 = 2; in.1 = 'first'; in.2 = 'second'
  address system 'cat' with input stem in. output stem out.
  say 'run' out.0 out.1 out.2
::method run2
  expose out.2
  out.0 = 1
  address system 'echo z' with output append stem out.
  say 'run2' out.0 out.1 out.2
::method show
  expose out. in.
  say 'show' out.1 out.2 in.2 out.~items
