/* Nil-scoped metaclass entries retain their registered Method objects. */
m = .method~new('PING', "return 'pong'")
d = .directory~new
d~ping = m
d~install = "self~setMethod('OWN', 'return 17')"
c = .class~enhanced(d, 'Remote')
drop d m
held = .WeakReference~new(c~instanceMethod('PING'))
say 'entry' c~ping (held~value~scope == .nil)
child = c~subclass('Child')
say 'inherited' child~ping (child~instanceMethod('PING') == held~value)
do i = 1 to 3
  a = .array~new(10)
  say 'held' (c~instanceMethod('PING') == held~value) (child~instanceMethod('PING') == held~value)
  methods = c~instanceMethods(.nil)
  found = 0
  do while methods~available
    if methods~index == 'PING' then do
      found = found + 1
      say 'listed' (methods~item == held~value) (methods~item~scope == .nil)
    end
    methods~next
  end
  say 'found' found
end
c~install
say 'object' c~own (c~instanceMethod('OWN')~scope == .nil)
say 'ordinary' .ordinary~ping (.ordinary~instanceMethod('PING')~scope == .ordinary)
::class ordinary
::method ping class
  return 'ordinary'
