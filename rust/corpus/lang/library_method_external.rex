/* A ::METHOD whose EXTERNAL names a shared library binds to a procedure of
   that library, and a send runs it. Every answer here comes out of
   librxregexp.so: INIT stores the automaton as a .Pointer in the method's own
   scope pool, and each later send reads it back through CSELF. !POS is
   written by the extension through SetObjectVariable, and POS depends on
   StringData answering one address per object for the whole call. */
r = .Re~new('a*b')
say 'parse' r~doparse('aab')
say 'pos' r~where
say 'match' r~does('aab')
say 'lastpos' r~where

q = .Re~new()
say 'reparse' q~doparse('xy*z')
say 'match' q~does('xyyz')

say 'minimal' r~doparse('a*b', 'MINIMAL')
say 'find' r~find('zzaab')
say 'findpos' r~where

::class Re subclass Object
::method init external "LIBRARY rxregexp RegExp_Init"
::method uninit external "LIBRARY rxregexp RegExp_Uninit"
::method doparse external "LIBRARY rxregexp RegExp_Parse"
::method does external "LIBRARY rxregexp RegExp_Match"
::method find external "LIBRARY rxregexp RegExp_Pos"
::method where
  expose !POS
  return !POS
