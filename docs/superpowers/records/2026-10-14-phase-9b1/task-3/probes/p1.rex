say filespec('n', .k~new)
::class k
::method string
  do f over .context~stackFrames
    say f~type f~name f~line (f~arguments~items) (f~executable~class~id) (f~context~name~right(6))
  end
  return '/a/b.c'
