do h = 0 to 5
  s = .stream~new('HANDLE:'h)
  say h s~query('streamtype') '['s~query('handle')']' '['s~query('exists')']' s~state
end
s = .stream~new('HANDLE:3'); say s~linein s~query('streamtype') s~state
