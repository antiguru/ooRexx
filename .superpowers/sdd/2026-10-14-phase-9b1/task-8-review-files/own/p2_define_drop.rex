.k~define('D', .k~method('BODY')~copy)
o = .k~new
say o~d
say .k~new~hasMethod('D') o~hasMethod('D')
exit
::class k
::method body
  .k~define('D')
  call filler
  say 'exe' .context~executable~class~id .context~executable~scope~id
  say 'src' .context~executable~source~items
  f = .context~stackFrames[1]
  say 'frame' f~name f~executable~class~id
  signal on syntax
  x = 1 + 'a'
  return 'no'
syntax:
  c = condition('O')
  say 'cond' c~code c~traceback~items
  return 'ok'
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
