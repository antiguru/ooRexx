frags = .array~of('x=1; y=(; z=2', 'do forever then', 'do; nop', 'select; nop; end', -
  'do i = 1; end j', 'if 1 = 1 then', 'do; x: nop; end', 'if 1 = 1 then nop; else', -
  'select; when 1 then nop; otherwise', 'do label a; end b', 'end', 'select; end', -
  'select label s; when 1 then nop; end t', 'if 1 = 1 then nop; x: else nop', -
  '::routine r', 'x: nop')
do f over frags
  call try f
end
exit

try:
  signal on syntax name trap
  interpret arg(1)
  say arg(1) '-> ok'
  return
trap:
  c = condition('o')
  items = ''
  do a over c~additional
    items = items '['a']'
  end
  say arg(1) '->' c~code 'items' c~additional~items items
  say '  ' c~message
  return
