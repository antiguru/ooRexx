/* The bug 2003 shape: ten REPLY continuations each send an unguarded
   method whose GUARD ON WHEN is true at once, so each waits only for the
   lock and they run one after another. */
b = .Buffer~new
append = .Message~new(b, "ADD")
do 10
  append~reply
end
::class Buffer
::method init
  expose n
  n = 0
::method add unguarded
  expose n
  guard on when n = n
  say "n" n
  n = n + 1
