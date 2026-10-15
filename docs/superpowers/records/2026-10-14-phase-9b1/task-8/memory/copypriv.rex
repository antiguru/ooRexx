parse arg n
do i = 1 to n; m = .k~method('FOO')~copy; m~setPrivate; end
say 'done'
::class k
::method foo
  return 1
