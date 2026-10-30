b = .bag~new
b~put(.key~new('held'))
q = .key~new('query')
say 'allowed' b~hasItem(q, .key~new('allowed'))
say 'blocked' (b~hasItem(q, .key~new('blocked')) == .nil)
say 'calls' q~hashes q~equals
exit
::class key
::attribute label
::attribute hashes
::attribute equals
::method init
  expose label hashes equals
  use arg label
  hashes = 0
  equals = 0
::method hashCode
  expose hashes
  hashes += 1
  say 'hash'
  return 'key'
::method '=='
  expose equals
  use arg held
  equals += 1
  say 'equal' held~label
  return held~label \== 'blocked'
