call try '::options digits -1'
call try '::method'
call try '::class c inherit'
call try '::requires'
call try '::class c junk'
exit

try:
  signal on syntax name trap
  p = .package~new('x', .array~of(arg(1)))
  say arg(1) '-> ok'
  return
trap:
  c = condition('o')
  items = ''
  do a over c~additional
    items = items '['a']'
  end
  say arg(1) '->' c~code 'items' c~additional~items items
  return
