.myarr~define('HAS', .array~method('HASITEM'))
.myarr~define('HASITEM')
a = .myarr~of(.k~new)
say a~has(.k~new)
exit
::class k
::method '=='
  do f over .context~stackFrames
    if f~line == .nil then say '  ' f~type f~name f~traceline '|' (f~executable == .nil)
  end
  return 0
::class myarr subclass array
