-- StackFrame~executable on a library routine's frame while it is live: the
-- routine asks its argument for its string value, and that method reads
-- .context~stackFrames.
say r(.k~new)
exit 0
::routine r external 'LIBRARY orxfunction TestStringArg'
::class k
::method string
  do f over .context~stackFrames
    say 'frame' f~type f~name
    e = f~executable
    say 'executable' e~class~id (e == .nil)
  end
  return 'k'
