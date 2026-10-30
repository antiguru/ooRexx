t = .table~new
k = .key~new(t)
t~put('value',k)
k = .key~new(t)
k~arm
say 'removed' (t~remove(k) == .nil) 'items' t~items
exit
::class key
::method init
  expose table armed
  use arg table
  armed = 0
::method arm
  expose armed
  armed = 1
::method hashCode
  return 'key'
::method '=='
  expose table armed
  if armed then do
    armed = 0
    table~empty
  end
  return 1
