/* A SIGNAL ON trap whose label is missing turns itself off and raises 16.1
   at the raising clause, which a SYNTAX or ANY trap in the same activation
   takes (upstream bug 1877). */
signal on novalue
signal on syntax
a = novalue
a = 'next clause'
syntax:
say 'syntax' a rc condition('C') sigl condition('O')~position condition('O')~message
say 'novalue trap' condition('S')
signal on novalue name no_such_label
signal on any name any_handler
b = undefined_variable
say 'not reached'
any_handler:
say 'any' condition('C') rc sigl condition('O')~message
call inner
signal on syntax name no_such_label_either
say 1 / 0

inner:
  signal on notready name missing_notready_label
  signal on syntax name inner_syntax
  signal on notready name missing_notready_label
  x = charin('/')
  say 'inner not reached'
inner_syntax:
  say 'inner syntax' rc sigl condition('O')~message
  return
