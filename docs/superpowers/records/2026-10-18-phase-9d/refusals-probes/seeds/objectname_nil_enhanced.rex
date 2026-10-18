-- .Object's OBJECTNAME= method sent from a method run with .nil as receiver.
signal on syntax name trapped
o = .zz~new
say o~go
exit 0
trapped: say 'syntax' condition('O')~code
::class zz
::method go
  m = .Object~method('OBJECTNAME=')
  return .nil~run(m, 'I', 'zz')
