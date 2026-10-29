a = .array~of(.withname, .plain)
say a~makestring(, ', ')
say a~toString(, '|')
trace i
x = .withname
trace off
say .withname~class~defaultname
signal on syntax
.withname~nosuch
exit
syntax:
say condition('o')~message
.plain~nosuch
::class withname
::method defaultname class
  return 'DN'
::class plain
