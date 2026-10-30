r = .relation~new
k = .key~new(r)
r~put('old',k)
q = .key~new(r)
q~arm
a = r~allAt(q)
say 'answer' a[1] 'current' r~at(q) 'items' r~items
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
    table~put('new', .key~new(table))
  end
  return 1
