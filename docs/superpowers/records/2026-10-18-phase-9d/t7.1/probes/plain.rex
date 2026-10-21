call t '.Array~copy'
call t '.zz~copy'
call t '.context~copy'
call t '.RexxInfo~copy'
call t '.nil~copy'
call t '.nil~objectName = "q"'
call t '.context~stackFrames[1]~copy~class~id'
exit
t: parse arg code
  signal on syntax name bad
  interpret 'x =' code
  say 'ok' code x
  return
bad: c = condition('O'); say code '->' c~code c~message
  return
::class zz
