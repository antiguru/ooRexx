/* A comparator's compare runs under a METHOD frame of the sortWith or
   stableSortWith that called it: its arguments are the comparator, its
   target the receiver, its executable Array's method, and it has no line.
   A condition raised in compare lists the frame too. */
test = .array~of(2, 1)
c = .recorder~new
test~sortWith(c)
call show 'sortWith', c~frames, test, c
test~stableSortWith(c)
call show 'stableSortWith', c~frames, test, c
signal on syntax
test~sortWith(.failing~new)
exit
syntax:
  frames = condition('o')~stackFrames
  do f over frames
    if f~type \== 'PROGRAM' then say 'condition' f~type f~name f~arguments~items f~traceLine
  end
  exit

show: procedure
  use arg tag, frames, test, c
  f = frames[2]
  say tag f~type f~name f~line (f~target == test) f~arguments~items (f~arguments[1] == c)
  say tag (f~executable == .Array~method(f~name)) f~traceLine
  say tag 'next' frames[3]~type
  return

::class recorder
::attribute frames
::method compare
  self~frames = .context~stackFrames
  return 0

::class failing
::method compare
  return 1 + 'x'
