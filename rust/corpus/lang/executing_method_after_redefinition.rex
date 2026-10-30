/* An executing Method survives dictionary mutation and collection. */
do prefetch = 0 to 1
  do change over .array~of('replace', 'remove', 'hide')
    .local~change = change
    .local~prefetch = prefetch
    .k~define('FOO', .k~method('BODY')~copy~~setPrivate~~setUnguarded)
    .local~entered = .WeakReference~new(.k~method('FOO'))
    .s~new~go
  end
end
.k~define('FOO', .k~method('ALIASES')~copy~~setPrivate~~setUnguarded)
.k~define('BAR', .k~method('ALIASES')~copy)
.local~foo = .WeakReference~new(.k~method('FOO'))
.local~bar = .WeakReference~new(.k~method('BAR'))
.s~new~both
exit
::class k
::method go
  self~foo
::method both
  self~foo('FOO')
::method body protected
  if .prefetch then before = .context~executable
  select
    when .change == 'replace' then .k~define('FOO', .k~method('OTHER'))
    when .change == 'remove' then .k~define('FOO')
    otherwise .k~define('FOO', .nil)
  end
  a = .array~new(10)
  m = .context~executable
  say .prefetch .change (m == .entered~value) m~isGuarded m~isPrivate m~isProtected
  say 'same' (m == .context~executable)
::method aliases protected
  use arg name
  .k~define(name)
  if name == 'FOO' then self~bar('BAR')
  a = .array~new(10)
  m = .context~executable
  if name == 'FOO' then original = .foo~value
  else original = .bar~value
  say name (m == original) m~isGuarded m~isPrivate m~isProtected
::method other
  return 17
::class s subclass k
