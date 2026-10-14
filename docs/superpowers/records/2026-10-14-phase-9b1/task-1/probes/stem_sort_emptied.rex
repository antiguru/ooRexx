.k~new~run
::class k
::method run
  expose s.
  signal on syntax
  s.0 = 2; s.1 = 'c'; s.2 = self
  call SysStemSort 's.'
  say s.0 '['s.1'] ['s.2']'
  return
syntax:
  say 'syntax' condition('o')~code
::method string
  expose s.
  if \.local~hasEntry('EMPTIED') then do
    .local~emptied = 1
    s.~empty
  end
  return 'b'
