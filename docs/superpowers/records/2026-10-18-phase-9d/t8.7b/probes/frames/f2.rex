a = .array~of(.k~new, .k~new)
a~stableSort
t = .table~new
t~put('v', .k~new)
t[.k~new] = 'w'
s = .set~new
s[.k~new] = .k~new
s~put('plain')
d = .directory~new
d~setMethod('go', 'call show "go"; return 3')
say 'explicit' d~unknown('GO', .array~new)
.k~new~setup
signal on syntax
.array~of(.k~new, .k~new, .k~new)~sort
exit
syntax:
  c = condition('o')
  say 'syntax' c~code
  do f over c~stackframes
    say '  cond' f~type '|' f~name '|' f~line
  end
  call deep
  exit
deep:
  .k~new~deeprun
  exit
::routine show public
  use arg label
  do f over .context~stackframes
    if f~type == 'ROUTINE' & f~name == 'SHOW' then iterate
    say '  ' label f~type '|' f~name '|' f~line '|' f~target~class~id '|' f~arguments~items '|' (f~executable == .nil)
  end
::class k
::method deeprun
  self~run('call show "deeprun"; raise syntax 98.900 array("deep")')
::method setup
  .k~setFlag
::method setFlag class
  expose fl
  fl = 1
::method flag class
  expose fl
  return fl == 1
::method compareTo
  call show 'sort'
  if .k~flag then raise syntax 93.900 array('cmp')
  return 0
::method hashCode
  call show 'hash'
  return 'a'
::method '=='
  call show 'eq'
  return 1
