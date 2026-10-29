.myarr~define('HAS', .array~method('HASITEM'))
.myarr~define('HASITEM')
a = .myarr~of(.k~new)
say a~has(.k~new)
.myarr2~define('ORDER', .array~method('SORT'))
.myarr2~define('SORT')
b = .myarr2~of(.j~new(2), .j~new(1))
b~order
say 'ordered'
exit
::class k
::method '=='
  do f over .context~stackFrames
    if f~line == .nil then say '  ' f~type f~name f~traceline
  end
  return 0
::class j
::method init; expose v; use arg v
::method v; expose v; return v
::method compareTo; expose v; use arg o
  do f over .context~stackFrames
    if f~line == .nil then say '  ' f~type f~name f~traceline
  end
  return v - o~v
::class myarr subclass array
::class myarr2 subclass array
