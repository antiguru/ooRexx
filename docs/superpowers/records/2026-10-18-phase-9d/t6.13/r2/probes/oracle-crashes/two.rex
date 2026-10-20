call time 'r'
ms = .array~new
do k = 1 to 4; ms~append(.T~new~start('go', k)); end
do m over ms; say m~result; end
say 'parallel' (time('e') < 1.5)
::class T
::method go
  use arg k
  i.0 = 3; i.1 = k; i.2 = k*2; i.3 = k*3
  address system 'sleep 0.5; cat; echo err'k' >&2' with input stem i. output stem o. error stem e.
  return k rc o.0 o.1 o.2 o.3 e.0 e.1 .context~rs
