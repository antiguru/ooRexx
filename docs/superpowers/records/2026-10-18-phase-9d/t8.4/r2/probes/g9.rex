say .c~k
::class c
::constant k (.c~u)
::method u class unguarded
  m = self~start('g')
  say 'waiting'
  return m~result
::method g class unguarded
  return 1/0
