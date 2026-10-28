/* A program's own STDIN, STDOUT and STDERR streams are unopened until an
   operation opens them, as any stream is: until then they have no stream type
   or handle and their qualified name is a path; once open they are READY and
   answer their name as written. Only reads and queries here. */
call show 'STDIN', .stream~new('STDIN')
call show 'stdin:', .stream~new('stdin:')
call show 'STDOUT', .stream~new('STDOUT')
call show 'STDERR:', .stream~new('STDERR:')
s = .stream~new('STDIN')
say 'linein:' '['s~linein']' s~state
call opened 'STDIN', s
say 'close:' s~close
call opened 'closed', s
s = .stream~new('Stdin:')
say 'open:' s~open('read')
call opened 'Stdin:', s
call opened '.stdin', .stdin
call opened '.stdout', .stdout
exit

show: procedure
  use arg name, s
  say name 'type='s~query('streamtype') 'handle=['s~query('handle')']' 'exists=['s~query('exists')']',
    'qualified='||(s~qualify == directory()'/'name) 'state='s~state
  return

opened: procedure
  use arg name, s
  say name 'type='s~query('streamtype') 'handle=['s~query('handle')']' 'exists=['s~query('exists')']',
    'qualify=['s~qualify']' 'state='s~state
  return
