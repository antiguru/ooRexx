/* provide.xml `xmixin`: a mixin class adds a set of methods to another class
   through INHERIT, and is associated with a base class -- its first non-mixin
   superclass -- which bounds which classes may inherit it. */
say 'inherited' .k~m
say 'base-object' .mx~baseClass~id
say 'base-array' .am~baseClass~id
say 'superclasses' .k~superClasses~makeString('L', ' ')

::class mx mixinclass object
::method m class
  return 'from the mixin'
::class k inherit mx
::class am mixinclass array
