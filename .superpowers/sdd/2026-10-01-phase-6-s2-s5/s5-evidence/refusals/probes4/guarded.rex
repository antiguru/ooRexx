o = .k~new
a = o~start("g", "a")
b = o~start("g", "b")
c = o~start("u", "c")
d = o~start("u", "d")
a~wait; b~wait; c~wait; d~wait
say "end"
::class k
::method g guarded
  use arg n; say "enter" n; call syssleep 0.2; say "exit" n
::method u unguarded
  use arg n; call syssleep 0.1; say "u" n
