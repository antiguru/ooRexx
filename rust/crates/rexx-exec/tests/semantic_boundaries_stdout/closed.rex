call report 'local-open', .stdout
.stdout~close
call report 'local-closed', .stdout
call lineout 'STDERR', 'invalid' invalid(.stdout)
do name over .array~of('STDOUT', 'STDOUT:')
  s = .stream~new(name)
  call report name'-new', s
  call lineout 'STDERR', 'invalid' invalid(s)
  s~open('write')
  call report name'-open', s
  s~close
  call report name'-closed', s
end
exit
report: procedure
  use arg label, s
  call lineout 'STDERR', label '['s~query('position')']['s~query('position write char')']['s~query('position sys')']'
  return
invalid: procedure
  use arg s
  signal on syntax
  return s~query('position write read')
syntax:
  return condition('o')~code
