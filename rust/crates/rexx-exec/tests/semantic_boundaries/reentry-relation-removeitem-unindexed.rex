t = .relation~new
t~put('victim', .key~new(1))
w = .wanted~new(t)
say 'removed' t~removeItem(w) 'items' t~items 'victim-still-present' t~hasIndex(.key~new(1))
exit
::class key
::attribute label
::method init
  expose label
  use arg label
::method hashCode
  return 'key'
::method '=='
  expose label
  use arg other
  return label == other~label
::class wanted
::method init
  expose table armed
  use arg table
  armed = 1
::method '=='
  expose table armed
  if armed then do
    armed = 0
    table~put('extra', .key~new(2))
  end
  return 1
