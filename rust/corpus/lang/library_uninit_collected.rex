/* A ::METHOD UNINIT whose EXTERNAL names a shared library runs when the
   object is collected. RegExp_Uninit writes to no descriptor -- it deletes
   the automaton and calls DropObjectVariable("CSELF") -- so what makes the
   run visible is the drop, read back through a written method in the same
   scope pool the extension stores into, from a subclass finalizer that
   forwards to the native one. */
r = .Sub~new('a*b')
say 'live' r~probe~class~id
drop r
call gc 'force'
say 'done'

::class Re subclass Object
::method init external "LIBRARY rxregexp RegExp_Init"
::method uninit external "LIBRARY rxregexp RegExp_Uninit"
::method probe
  expose CSELF
  return CSELF

::class Sub subclass Re
::method uninit
  say 'sub before' self~probe~class~id
  forward class (super) continue
  say 'sub after' self~probe~class~id
