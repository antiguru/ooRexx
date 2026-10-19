/* A sortWith or stableSortWith frame names the method the dispatch ran and
   the message it ran under: Array's, reached through overrides forwarding to
   super one or two deep or by forward class (.array); another class's own
   entry where it, a mixin or an object defined Array's method as its own;
   and *UNNAMED* at .nil for a run of Array's method. */
c = .recorder~new
m = .myarr~of(2, 1)
m~sortWith(c)
call show 'forwarded', c
m~stableSortWith(c)
call show 'forwarded', c
.deep2~of(2, 1)~sortWith(c)
call show 'two deep', c
.other~of(2, 1)~sortWith(c)
call show 'defined', c
.other2~define('SORTWITH', .array~method('SORTWITH'))
.other2~define('STABLESORTWITH', .array~method('STABLESORTWITH'))
.fsub2~of(2, 1)~stableSortWith(c)
call show 'super of defined', c
.mid~of(2, 1)~stableSortWith(c)
call show 'through mid', c
.fsub2~of(2, 1)~sortWith(c)
call show 'forward to .array', c
.withmix~of(2, 1)~sortWith(c)
call show 'mixin', c
y = .myarr2~of(2, 1)
y~sm('SORTWITH', .array~method('SORTWITH'), 'OBJECT')
y~sortWith(c)
call show 'object scope', c
z = .myarr2~of(2, 1)
z~sm('SORTWITH', .array~method('SORTWITH'))
z~sortWith(c)
call show 'float scope', c
z~runSort(c)
call show 'run', c
.array~of(2, 1)~sendWith('SORTWITH', .array~of(c))
call show 'sendWith', c
.other~of(2, 1)~sendWith('SORTWITH', .array~of(c))
call show 'sendWith defined', c
exit

show: procedure
  use arg tag, c
  f = c~frames[2]
  say tag f~type f~name f~traceLine
  e = f~executable
  say tag 'array' (e == .Array~method('SORTWITH')) (e == .Array~method('STABLESORTWITH')) 'scope' e~scope~string
  return

::class recorder
::attribute frames
::method compare
  self~frames = .context~stackFrames
  return 0

::class myarr subclass array
::method sortWith
  forward class (super)
::method stableSortWith
  forward class (super)

::class deep1 subclass array
::method sortWith
  forward class (super)
::class deep2 subclass deep1
::method sortWith
  forward class (super)

::class other subclass array
::method activate class
  self~define('SORTWITH', .array~method('SORTWITH'))

::class other2 subclass array
::class fsub2 subclass other2
::method stableSortWith
  forward class (super)
::method sortWith
  forward class (.array)
::class mid subclass fsub2

::class mx mixinclass object
::method activate class
  self~define('SORTWITH', .array~method('SORTWITH'))
::class withmix subclass array inherit mx

::class myarr2 subclass array
::method sm
  forward message 'SETMETHOD'
::method runSort
  use arg c
  self~run(.array~method('SORTWITH'), 'I', c)
