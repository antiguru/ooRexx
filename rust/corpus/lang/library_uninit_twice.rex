/* An explicit UNINIT send does not stop the collector sending another, so
   the extension's own guard is what a second finalization reaches: with
   CSELF dropped, CSELF arrives as a null pointer and RegExp_Uninit deletes
   nothing. */
r = .Sub~new('a*b')
r~uninit
say 'explicit' r~probe~class~id
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
  say 'sub' self~probe~class~id
  forward class (super) continue
