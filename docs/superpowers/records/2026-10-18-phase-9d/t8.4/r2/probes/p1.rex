o = .c~new; o~go; say 'done'
::class c
::constant e (.context~executable)
::method go
  self~run(.c~e)
  say 'run instr' var('result')
