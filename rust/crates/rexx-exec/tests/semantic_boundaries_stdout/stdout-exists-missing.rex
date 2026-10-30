do name over .array~of('STDOUT', 'STDOUT:')
  s = .stream~new(name)
  call report 'new', s, name
  s~open
  call report 'open', s, name
  s~close
  call report 'closed', s, name
  s~open
  call report 'reopen', s, name
  s~close
end
exit
report: procedure
  use arg phase, s, name
  found = s~query('exists')
  select
    when found == '' then form = 'empty'
    when found == name then form = 'bare'
    when found == .stream~new('./' || name)~qualify then form = 'qualified'
    otherwise form = 'unexpected:' || found
  end
  call lineout 'STDERR', name phase form s~state '['s~query('size')']['s~query('position')']['s~query('handle')']['s~query('streamtype')']'
  return
