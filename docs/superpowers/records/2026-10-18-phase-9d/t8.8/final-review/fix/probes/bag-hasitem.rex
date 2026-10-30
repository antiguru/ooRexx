b = .bag~new
b~put(.key~new(b, 'old'))
q = .key~new(b, 'query')
q~arm
say 'answer' b~hasItem(q) 'items' b~items
say 'calls' q~hashes q~equals
exit
::class key
::attribute label
::attribute hashes
::attribute equals
::method init
  expose bag label armed hashes equals
  use arg bag, label
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
  expose bag armed equals label
  use arg held
  equals += 1
  if label == 'query' then say 'equal' held~label
  if armed then do
    armed = 0
    bag~empty
    bag~put(.key~new(bag, 'new'))
  end
  return 1
