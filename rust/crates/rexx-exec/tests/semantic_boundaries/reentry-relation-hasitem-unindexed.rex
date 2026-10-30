r = .relation~new
r~put('tail', 'key')
r~put('head', 'key')
q = .wanted~new(r)
say 'answer' r~hasItem(q) 'items' r~items
say 'calls' q~calls
exit
::class wanted
::attribute calls
::method init
  expose table calls
  use arg table
  calls = 0
::method '=='
  expose table calls
  use arg held
  calls += 1
  say 'item' held
  if calls = 1 then do
    table~empty
    table~put('new-tail', 'key')
    table~put('new-head', 'key')
    return 0
  end
  return held == 'new-tail'
