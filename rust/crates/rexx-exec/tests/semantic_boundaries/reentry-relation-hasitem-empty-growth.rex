r = .relation~new
k = .key~new(r)
r~put('old',k)
q = .key~new(r)
q~arm
answer = r~hasItem('new', q)
say 'calls' q~hashes q~equals
say 'answer' answer 'items' r~items
exit
::class key
::attribute hashes
::attribute equals
::method init
  expose table armed hashes equals
  use arg table
  hashes = 0
  equals = 0
  armed = 0
::method arm
  expose armed
  armed = 1
::method hashCode
  expose hashes
  hashes += 1
  return 'key'
::method '=='
  expose table armed equals
  equals += 1
  if armed then do
    armed = 0
    table~empty
    table~put('new', .key~new(table))
    do i = 1 to 40
      table~put('extra', .key~new(table))
    end
  end
  return 1
