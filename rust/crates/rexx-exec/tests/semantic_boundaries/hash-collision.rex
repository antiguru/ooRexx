r = .relation~new
r~put('first', .key~new('A'))
r~put('second', .key~new('B'))
k = .key~new('A')
say 'has' r~hasItem('first',k)
say 'remove' r~removeItem('first',k)
exit
::class key
::attribute label
::method init
  expose label
  use arg label
::method hashCode
  return 'same hash'
::method '=='
  expose label
  use arg other
  say 'equal' label other~label
  return label == other~label
