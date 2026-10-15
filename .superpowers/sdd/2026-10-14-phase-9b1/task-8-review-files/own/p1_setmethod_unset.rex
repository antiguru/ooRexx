o = .k~new
o~put('X', .k~method('BODY')~copy)
say o~x
say o~hasMethod('X')
exit
::class k
::method put
  use arg n, m, s = "FLOAT"
  self~setMethod(n, m, s)
::method body
  self~unsetMethod('X')
  call filler
  say 'exe' .context~executable~class~id .context~executable~isGuarded
  say 'src' .context~executable~source~items
  say 'name' .context~name
  f = .context~stackFrames[1]
  say 'frame' f~name f~type f~executable~class~id
  signal on syntax
  x = 1 + 'a'
  return 'no'
syntax:
  c = condition('O')
  say 'cond' c~code c~traceback~items
  call filler
  say 'pkg' .context~executable~package~class~id
  return 'ok'
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
