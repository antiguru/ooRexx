/* A UNINIT a program attaches to one object is registered for finalization
   the way a class's declaration is, and hiding the name takes the
   registration back. Each object is dropped and forced on its own, because
   nothing may depend on the order two finalizers run in. */
o = .k~new
o~mk
drop o
call gc 'force'
say 'first done'
p = .j~new
p~hide
drop p
call gc 'force'
say 'second done'

::class k
::method mk
  self~setMethod('UNINIT', 'say "one-off uninit"')

::class j
::method uninit
  say 'class uninit'
::method hide
  self~setMethod('UNINIT')
