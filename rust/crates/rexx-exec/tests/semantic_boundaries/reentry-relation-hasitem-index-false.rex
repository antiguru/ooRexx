r = .relation~new
r~put('tail', .key~new(r, 'old-tail'))
r~put('head', .key~new(r, 'old-head'))
q = .key~new(r, 'query')
q~arm
say 'answer' r~hasItem('new-tail', q) 'items' r~items
say 'calls' q~hashes q~equals
exit
::class key
::attribute label
::attribute hashes
::attribute equals
::method init
  expose table label armed hashes equals
  use arg table, label
  armed = 0
  hashes = 0
  equals = 0
::method arm
  expose armed
  armed = 1
::method hashCode
  expose hashes
  hashes += 1
  return 'key'
::method '=='
  expose table armed equals label
  use arg held
  equals += 1
  if label == 'query' then say 'index' held~label
  if armed then do
    armed = 0
    table~empty
    table~put('new-tail', .key~new(table, 'new-tail'))
    table~put('new-head', .key~new(table, 'new-head'))
    return 0
  end
  return 1
