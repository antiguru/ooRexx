parse arg n
do i = 1 to n
  .routines['A'] = .routine~new('a', 'if arg(1) > 0 then return b(arg(1) - 1); return 0')
  .routines['B'] = .routine~new('b', 'return a(arg(1))')
  x = .routines['A']~call(1)
end
say x
::routine dummy
