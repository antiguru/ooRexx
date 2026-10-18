n = 0
do i = .c~new to 5 for 3
  n = n + 1
end
say 'computed one:' n
n = 0
do i = .d~new to 5 for 3
  n = n + 1
end
say 'literal one:' n
::class c
::method '+'; return self
::method '>'; return 0 + 1
::class d
::method '+'; return self
::method '>'; return 1
