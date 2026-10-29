a = .array~of(.withname~new, .plain)
say a~makestring(, ', ')
say a~toString(, '|')
trace i
x = .withname~new
trace off
say .withname~new~defaultname
signal on syntax
.withname~new~nosuch
exit
syntax:
say condition('o')~message
.plain~nosuch
::class withname
::method defaultname
  return 'DN'
::class plain
